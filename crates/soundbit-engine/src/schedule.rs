//! Songs compiled into sample-accurate playback plans.

use soundbit_core::{InstrumentId, Song, Track};
use soundbit_dsp::{DelayParams, ReverbParams};

use crate::clock::StepClock;
use crate::error::{EngineError, check_sample_rate};
use crate::patch::Patch;

/// A note with absolute sample positions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct NoteEvent {
    /// Sample at which the note starts.
    pub(crate) on: u64,
    /// Sample at which the note is released.
    pub(crate) off: u64,
    /// Index into [`Schedule::patches`].
    pub(crate) patch: usize,
    /// MIDI pitch.
    pub(crate) pitch: u8,
    /// Velocity as a gain in `(0, 1]`.
    pub(crate) velocity: f32,
    /// Microtonal pitch offset in cents.
    pub(crate) detune_cents: i16,
    /// Fine frequency offset in millihertz.
    pub(crate) detune_milli_hz: i32,
    /// Continuous pitch glide in semitones over the note duration.
    pub(crate) slide_semitones: i8,
    pub(crate) glissando: bool,
    pub(crate) zzfx_effect: Option<soundbit_core::ZzfxEffect>,
    pub(crate) sample: Option<usize>,
}

/// Everything one track plays, in start order.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TrackPlan {
    pub(crate) gain: f32,
    pub(crate) is_kick: bool,
    pub(crate) sidechain_duck: f32,
    pub(crate) delay_send: f32,
    pub(crate) events: Vec<NoteEvent>,
}

/// Converts between bar positions and sample positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionMap {
    clock: StepClock,
    steps_per_bar: u64,
}

impl PositionMap {
    /// Sample at which `bar` begins.
    pub fn bar_start(&self, bar: u16) -> u64 {
        self.clock.sample_at(u64::from(bar) * self.steps_per_bar)
    }

    /// Position of `sample` in bars (fractional), ignoring swing.
    pub fn bars_at(&self, sample: u64) -> f64 {
        self.clock.step_at(sample) as f64 / self.steps_per_bar as f64
    }
}

/// A song compiled for playback at one sample rate.
///
/// Instruments become DSP patches and notes become events at exact sample positions. Built off
/// the audio thread; the engine only reads it.
#[derive(Debug, Clone, PartialEq)]
pub struct Schedule {
    sample_rate: u32,
    patches: Vec<Patch>,
    samples: Vec<Vec<f32>>,
    tracks: Vec<TrackPlan>,
    end: u64,
    clock: StepClock,
    steps_per_bar: u64,
    loop_region: Option<(u64, u64)>,
    reverb: ReverbParams,
    uses_reverb: bool,
    delay_params: DelayParams,
    uses_delay: bool,
}

impl Schedule {
    /// Compiles `song` for playback at `sample_rate`.
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::InvalidSampleRate`] for unsupported sample rates.
    pub fn compile(song: &Song, sample_rate: u32) -> Result<Self, EngineError> {
        check_sample_rate(sample_rate)?;
        let clock = StepClock::new(song.timing(), sample_rate);
        let steps_per_bar = u64::from(song.timing().steps_per_bar());

        let instrument_ids: Vec<InstrumentId> = song.instruments().map(|(id, _)| id).collect();
        let patches: Vec<Patch> = song
            .instruments()
            .map(|(_, instrument)| Patch::compile(instrument, sample_rate))
            .collect();
        let uses_reverb = patches.iter().any(|patch| patch.reverb_send > 0.0);
        let room = song.reverb();
        let reverb = ReverbParams {
            size: f32::from(room.size.get()) / 100.0,
            damping: f32::from(room.damping.get()) / 100.0,
        };
        let mut tracks: Vec<TrackPlan> = song
            .tracks()
            .map(|(_, track)| plan_track(song, track, &clock, &instrument_ids))
            .collect();
        let step_samples = ((f64::from(sample_rate) * 60.0)
            / (f64::from(song.timing().bpm.get()) * 4.0))
            .round() as usize;
        let max_d = sample_rate.saturating_sub(1).max(64) as usize;
        let delay_params = DelayParams {
            left_samples: (step_samples * 3).clamp(64, max_d),
            right_samples: (step_samples * 4).clamp(64, max_d),
            feedback: 0.34,
            wet: 0.28,
        };
        let uses_delay = tracks.iter().any(|track| track.delay_send > 0.0);
        let end = clock.sample_at(u64::from(song.length_bars()) * steps_per_bar);
        let samples = crate::sample_bank::compile(&mut tracks, &patches, sample_rate, end)?;
        let loop_region = song.loop_range().map(|range| {
            let bar = |bar: u16| clock.sample_at(u64::from(bar) * steps_per_bar);
            (bar(range.start()), bar(range.end()))
        });

        Ok(Self {
            sample_rate,
            patches,
            samples,
            tracks,
            end,
            clock,
            steps_per_bar,
            loop_region,
            reverb,
            uses_reverb,
            delay_params,
            uses_delay,
        })
    }

    /// Converts between bars and samples for this schedule. Cheap to copy, so it can be kept
    /// on the control side after the schedule moves to the audio thread.
    pub fn position_map(&self) -> PositionMap {
        PositionMap {
            clock: self.clock,
            steps_per_bar: self.steps_per_bar.max(1),
        }
    }

    /// The song's loop region in samples, if it has one.
    pub fn loop_region(&self) -> Option<(u64, u64)> {
        self.loop_region
    }

    /// Sample rate the schedule was compiled for.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Sample at which the arrangement ends. Releasing notes may ring on after it.
    pub fn end_sample(&self) -> u64 {
        self.end
    }

    /// The song's reverb room.
    pub(crate) fn reverb(&self) -> ReverbParams {
        self.reverb
    }

    /// Whether any instrument sends to the reverb. Songs that don't skip it entirely.
    pub(crate) fn uses_reverb(&self) -> bool {
        self.uses_reverb
    }

    pub(crate) fn delay_params(&self) -> DelayParams {
        self.delay_params
    }

    pub(crate) fn uses_delay(&self) -> bool {
        self.uses_delay
    }

    pub(crate) fn samples(&self) -> &[Vec<f32>] {
        &self.samples
    }

    pub(crate) fn patches(&self) -> &[Patch] {
        &self.patches
    }

    pub(crate) fn tracks(&self) -> &[TrackPlan] {
        &self.tracks
    }
}

fn plan_track(
    song: &Song,
    track: &Track,
    clock: &StepClock,
    instrument_ids: &[InstrumentId],
) -> TrackPlan {
    let settings = track.settings();
    if settings.muted {
        return TrackPlan {
            gain: 0.0,
            is_kick: false,
            sidechain_duck: 0.0,
            delay_send: 0.0,
            events: Vec::new(),
        };
    }

    let track_name = settings.name.trim().to_ascii_lowercase();
    let mut is_kick = track_name.contains("kick");
    let mut sidechain_duck = 0.0_f32;
    let mut delay_send = 0.0_f32;

    let steps_per_bar = u64::from(song.timing().steps_per_bar());
    let mut events = Vec::new();
    for (bar, pattern_id) in track.clips() {
        let Some(pattern) = song.pattern(pattern_id) else {
            continue;
        };
        let Some(patch) = instrument_ids
            .iter()
            .position(|&id| id == pattern.instrument)
        else {
            continue;
        };
        if let Some(inst) = song.instrument(pattern.instrument) {
            let iname = format!("{track_name} {}", inst.name.to_ascii_lowercase());
            let is_perc = iname.contains("cowbell")
                || iname.contains("crash")
                || iname.contains("cymbal")
                || iname.contains("snare")
                || iname.contains("clap")
                || iname.contains("hat");
            if iname.contains("kick") || iname.contains("taiko") {
                is_kick = true;
            } else if !is_perc
                && !iname.contains("zurna")
                && !iname.contains("eurobeat synth brass")
                && !iname.contains("initial d brass")
                && (iname.contains("pad")
                    || iname.contains("chord")
                    || iname.contains("harmony")
                    || iname.contains("string")
                    || iname.contains("choir")
                    || iname.contains("brass")
                    || iname.contains("bass")
                    || iname.contains("sub")
                    || iname.contains("rhodes")
                    || iname.contains("ep")
                    || iname.contains("organ")
                    || iname.contains("donk")
                    || iname.contains("house piano")
                    || iname.contains("padsynth")
                    || iname.contains("pizz"))
            {
                sidechain_duck = sidechain_duck.max(0.42);
            } else if !is_perc
                && (iname.contains("lead")
                    || iname.contains("hook")
                    || iname.contains("guitar")
                    || iname.contains("brass")
                    || iname.contains("trance")
                    || iname.contains("sabi")
                    || iname.contains("arp")
                    || iname.contains("pluck")
                    || iname.contains("vocal")
                    || iname.contains("talkbox")
                    || iname.contains("hoover")
                    || iname.contains("acid")
                    || iname.contains("303")
                    || iname.contains("zurna")
                    || iname.contains("shamisen")
                    || iname.contains("whistle")
                    || iname.contains("oud")
                    || iname.contains("cz")
                    || iname.contains("wavetable"))
            {
                sidechain_duck = sidechain_duck.max(0.14);
            }
            let bass_like = iname.contains("bass")
                || iname.contains("sub")
                || iname.contains("organ")
                || iname.contains("donk");
            if !is_kick
                && !is_perc
                && !bass_like
                && inst.reverb_send.get() > 0
                && (iname.contains("lead")
                    || iname.contains("brass")
                    || iname.contains("guitar")
                    || iname.contains("arp")
                    || iname.contains("trance")
                    || iname.contains("sabi")
                    || iname.contains("hook")
                    || iname.contains("solo")
                    || iname.contains("emotional")
                    || iname.contains("pluck")
                    || iname.contains("vocal")
                    || iname.contains("talkbox")
                    || iname.contains("cz"))
            {
                delay_send = delay_send.max(0.26);
            }
        }
        let bar_start = u64::from(bar) * steps_per_bar;
        for note in pattern.notes() {
            let start = bar_start + u64::from(note.start);
            events.push(NoteEvent {
                on: clock.sample_at(start),
                off: clock
                    .sample_at(start + u64::from(note.length.get()) + u64::from(note.tail_steps)),
                patch,
                pitch: note.pitch.get(),
                velocity: f32::from(note.velocity.get()) / 127.0,
                detune_cents: note.detune_cents,
                detune_milli_hz: note.detune_milli_hz,
                slide_semitones: note.slide_semitones,
                glissando: note.glissando,
                zzfx_effect: note.zzfx_effect,
                sample: None,
            });
        }
    }
    if is_kick {
        sidechain_duck = 0.0;
    }
    // Clips come in bar order and notes in start order, but a stable sort keeps the invariant
    // explicit.
    events.sort_by_key(|event| event.on);
    TrackPlan {
        gain: f32::from(settings.volume.get()) / 100.0,
        is_kick,
        sidechain_duck,
        delay_send,
        events,
    }
}
