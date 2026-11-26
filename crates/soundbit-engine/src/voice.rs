//! A single sounding note.

use soundbit_dsp::math::{note_frequency, semitone_ratio};
use soundbit_dsp::{Adsr, PitchMod};

use crate::patch::{Patch, StereoGain};
use crate::schedule::NoteEvent;
use crate::shaping::{MAX_UNISON, VoiceFilter};
use crate::source::Source;

/// One rendered frame of a voice: the stereo signal plus its mono reverb send.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct VoiceFrame {
    pub(crate) left: f32,
    pub(crate) right: f32,
    pub(crate) send: f32,
}

/// One playing note: its unison layers, envelope, modulation, filter and gains.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent playback and modulation flags"
)]
pub(crate) struct Voice {
    layers: [Source; MAX_UNISON],
    layer_count: usize,
    layer_ratios: [f64; MAX_UNISON],
    layer_pan_left: [f32; MAX_UNISON],
    layer_pan_right: [f32; MAX_UNISON],
    layer_gain: f32,
    hz: f64,
    sample_rate: f64,
    /// Whether the pitch moves, so the layers must be retuned every sample.
    modulated: bool,
    slide_semitones: f64,
    glissando: bool,
    sample: Option<usize>,
    sample_positions: [f64; MAX_UNISON],
    duration_samples: u64,
    pitch: PitchMod,
    filter_left: VoiceFilter,
    filter_right: VoiceFilter,
    envelope: Adsr,
    gain: StereoGain,
    send: f32,
    started_at: u64,
    release_at: u64,
    released: bool,
    active: bool,
}

impl Voice {
    pub(crate) fn is_active(&self) -> bool {
        self.active
    }

    pub(crate) fn started_at(&self) -> u64 {
        self.started_at
    }

    // RT-SAFE
    pub(crate) fn start(&mut self, patch: &Patch, event: &NoteEvent, sample_rate: f64) {
        let unison = &patch.unison;
        let base_hz = note_frequency(event.pitch)
            * semitone_ratio(f64::from(event.detune_cents) / 100.0)
            + f64::from(event.detune_milli_hz) / 1_000.0;
        self.hz = base_hz.max(1.0);
        self.sample_rate = sample_rate;
        self.layer_count = unison.layers.clamp(1, MAX_UNISON);
        self.layer_ratios = unison.ratios;
        self.layer_pan_left = unison.pan_left;
        self.layer_pan_right = unison.pan_right;
        self.layer_gain = unison.gain;
        for ((layer, ratio), phase) in self
            .layers
            .iter_mut()
            .zip(unison.ratios)
            .zip(unison.phases)
            .take(self.layer_count)
        {
            layer.start_at(&patch.source, self.hz * ratio, sample_rate, phase);
        }
        self.sample = event.sample;
        self.sample_positions = [0.0; MAX_UNISON];
        self.glissando = event.glissando;
        self.slide_semitones = f64::from(event.slide_semitones);
        self.duration_samples = event.off.saturating_sub(event.on).max(1);
        self.modulated = !patch.pitch.is_static() || event.slide_semitones != 0;
        self.pitch.start(patch.pitch, sample_rate);
        self.filter_left.start(patch.filter, sample_rate);
        self.filter_right.start(patch.filter, sample_rate);
        self.envelope.set_params(patch.envelope);
        self.envelope.note_on();
        self.gain = patch.gain.scaled(event.velocity);
        self.send = patch.reverb_send * event.velocity;
        self.started_at = event.on;
        self.release_at = event.off;
        self.released = false;
        self.active = true;
    }

    // RT-SAFE
    /// Sample indices belong to one schedule. Retire them before that bank is replaced.
    pub(crate) fn retire_sample(&mut self) {
        if self.sample.is_some() {
            self.active = false;
        }
    }

    // RT-SAFE
    /// Makes the note release no later than `now`.
    pub(crate) fn release_by(&mut self, now: u64) {
        self.release_at = self.release_at.min(now);
    }

    // RT-SAFE
    /// Renders one frame at sample position `now`.
    pub(crate) fn render(&mut self, now: u64, samples: &[Vec<f32>]) -> VoiceFrame {
        if !self.active {
            return VoiceFrame::default();
        }
        if !self.released && now >= self.release_at {
            self.released = true;
            self.envelope.note_off();
            for layer in self.layers.iter_mut().take(self.layer_count) {
                layer.release();
            }
        }

        if let Some(index) = self.sample {
            let Some(sample) = samples.get(index) else {
                // A missing sample must never unwind through a platform audio callback.
                self.active = false;
                return VoiceFrame::default();
            };
            return self.render_sample(now, sample);
        }
        if self.modulated {
            self.retune(now);
        }
        let (raw_left, raw_right) = self.layers_stereo();
        let envelope = self.envelope.next_level();
        let filt_left = self.filter_left.process(raw_left, envelope);
        let filt_right = self.filter_right.process(raw_right, envelope);
        let first = &self.layers[0];
        let (level, finished) = if first.has_own_envelope() {
            (1.0, first.is_silent())
        } else {
            (envelope, self.envelope.is_idle())
        };
        if finished {
            self.active = false;
        }
        let out_l = filt_left * level;
        let out_r = filt_right * level;
        let out_mono = 0.5 * (out_l + out_r);
        VoiceFrame {
            left: out_l * self.gain.left,
            right: out_r * self.gain.right,
            send: out_mono * self.send,
        }
    }

    // RT-SAFE
    #[allow(
        clippy::cast_precision_loss,
        reason = "sample counts fit comfortably in f64 mantissa for audio durations"
    )]
    fn retune(&mut self, now: u64) {
        let mut semitone_offset = self.pitch.next_offset();
        if self.slide_semitones != 0.0 {
            let progress = (now.saturating_sub(self.started_at) as f64
                / self.duration_samples as f64)
                .clamp(0.0, 1.0);
            let slide = self.slide_semitones * progress;
            semitone_offset += if self.glissando { slide.trunc() } else { slide };
        }
        let hz = (self.hz * semitone_ratio(semitone_offset)).max(1.0);
        for (layer, ratio) in self
            .layers
            .iter_mut()
            .zip(self.layer_ratios)
            .take(self.layer_count)
        {
            layer.set_frequency(hz * ratio, self.sample_rate);
        }
    }

    // RT-SAFE
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "bounded sample-buffer cursor"
    )]
    fn render_sample(&mut self, now: u64, samples: &[f32]) -> VoiceFrame {
        if now >= self.release_at {
            self.active = false;
            return VoiceFrame::default();
        }
        let progress = (now.saturating_sub(self.started_at) as f64 / self.duration_samples as f64)
            .clamp(0., 1.);
        let slide = self.slide_semitones * progress;
        let offset = self.pitch.next_offset() + if self.glissando { slide.trunc() } else { slide };
        let rate = 44100. / self.sample_rate * semitone_ratio(offset);
        let mut left = 0.0_f32;
        let mut right = 0.0_f32;
        let mut sounding = false;
        if self.layer_count <= 1 {
            let position = &mut self.sample_positions[0];
            let index = *position as usize;
            if index < samples.len() {
                sounding = true;
                let fraction = (*position - index as f64) as f32;
                let sample = samples[index]
                    + (samples.get(index + 1).copied().unwrap_or(0.) - samples[index]) * fraction;
                left = sample;
                right = sample;
            }
            *position += rate * self.layer_ratios[0];
        } else {
            for (((position, ratio), &pan_l), &pan_r) in self
                .sample_positions
                .iter_mut()
                .zip(self.layer_ratios)
                .zip(&self.layer_pan_left)
                .zip(&self.layer_pan_right)
                .take(self.layer_count)
            {
                let index = *position as usize;
                if index < samples.len() {
                    sounding = true;
                    let fraction = (*position - index as f64) as f32;
                    let sample = samples[index]
                        + (samples.get(index + 1).copied().unwrap_or(0.) - samples[index])
                            * fraction;
                    left += sample * pan_l;
                    right += sample * pan_r;
                }
                *position += rate * ratio;
            }
        }
        if !sounding {
            self.active = false;
            return VoiceFrame::default();
        }
        // ZzFXM de-clicks the end of each note over 99 samples at 44.1 kHz.
        let fade =
            ((self.release_at - now) as f64 / (99. * self.sample_rate / 44100.)).min(1.) as f32;
        let scale = self.layer_gain * fade;
        let envelope = self.envelope.next_level();
        let out_l = self.filter_left.process(left * scale, envelope);
        let out_r = if self.layer_count <= 1 {
            out_l
        } else {
            self.filter_right.process(right * scale, envelope)
        };
        let out_mono = 0.5 * (out_l + out_r);
        VoiceFrame {
            left: out_l * self.gain.left,
            right: out_r * self.gain.right,
            send: out_mono * self.send,
        }
    }

    // RT-SAFE
    fn layers_stereo(&mut self) -> (f32, f32) {
        if self.layer_count == 1 {
            let s = self.layers[0].next_sample();
            return (s, s);
        }
        let mut left = 0.0_f32;
        let mut right = 0.0_f32;
        for ((layer, &pan_l), &pan_r) in self
            .layers
            .iter_mut()
            .zip(&self.layer_pan_left)
            .zip(&self.layer_pan_right)
            .take(self.layer_count)
        {
            let s = layer.next_sample();
            left += s * pan_l;
            right += s * pan_r;
        }
        (left * self.layer_gain, right * self.layer_gain)
    }
}
