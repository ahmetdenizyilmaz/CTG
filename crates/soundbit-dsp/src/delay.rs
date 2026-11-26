//! Tempo-synced zero-allocation stereo ping-pong delay.

/// Delay times and feedback levels for [`StereoDelay`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DelayParams {
    /// Left delay tap length in samples.
    pub left_samples: usize,
    /// Right delay tap length in samples.
    pub right_samples: usize,
    /// Ping-pong cross-feedback gain in `[0, 0.85]`.
    pub feedback: f32,
    /// Wet mix gain returning to the stereo bus.
    pub wet: f32,
}

impl Default for DelayParams {
    fn default() -> Self {
        Self {
            left_samples: 14_400,
            right_samples: 19_200,
            feedback: 0.34,
            wet: 0.28,
        }
    }
}

/// Real-time safe stereo ping-pong delay operating on a caller-owned buffer.
#[derive(Debug, Clone, PartialEq)]
pub struct StereoDelay {
    channel_len: usize,
    params: DelayParams,
    write_l: usize,
    write_r: usize,
    lp_l: f32,
    lp_r: f32,
    hp_in_l: f32,
    hp_out_l: f32,
    hp_in_r: f32,
    hp_out_r: f32,
    quiet_samples: u32,
    quiet_after: u32,
}

const QUIET_LEVEL: f32 = 1e-5;

impl StereoDelay {
    /// Total buffer length (left + right, 1.0 second per channel) at `sample_rate`.
    #[must_use]
    pub fn buffer_len(sample_rate: u32) -> usize {
        (sample_rate.max(1) as usize) * 2
    }

    /// Creates a silent stereo ping-pong delay line sized for `sample_rate`.
    #[must_use]
    pub fn new(sample_rate: u32, params: DelayParams) -> Self {
        let mut delay = Self {
            channel_len: sample_rate.max(1) as usize,
            params: DelayParams::default(),
            write_l: 0,
            write_r: 0,
            lp_l: 0.0,
            lp_r: 0.0,
            hp_in_l: 0.0,
            hp_out_l: 0.0,
            hp_in_r: 0.0,
            hp_out_r: 0.0,
            quiet_samples: sample_rate / 10,
            quiet_after: sample_rate / 10,
        };
        delay.set_params(params);
        delay
    }

    /// Updates delay tap times and feedback safely without reallocating.
    pub fn set_params(&mut self, params: DelayParams) {
        let max_delay = self.channel_len.saturating_sub(1).max(1);
        self.params = DelayParams {
            left_samples: params.left_samples.clamp(1, max_delay),
            right_samples: params.right_samples.clamp(1, max_delay),
            feedback: params.feedback.clamp(0.0, 0.85),
            wet: params.wet.clamp(0.0, 1.0),
        };
    }

    /// Whether the delay echoes have finished ringing down to silence.
    #[must_use]
    pub fn is_quiet(&self) -> bool {
        self.quiet_samples >= self.quiet_after
    }

    /// Processes one stereo frame (`in_l`, `in_r`) and returns the wet ping-pong stereo frame.
    pub fn process(&mut self, buffer: &mut [f32], in_l: f32, in_r: f32) -> (f32, f32) {
        let ch_len = self.channel_len;
        if ch_len == 0 || buffer.len() < ch_len * 2 {
            return (0.0, 0.0);
        }
        let read_l = (self.write_l + ch_len - self.params.left_samples) % ch_len;
        let read_r = (self.write_r + ch_len - self.params.right_samples) % ch_len;

        let delayed_l = buffer.get(read_l).copied().unwrap_or(0.0);
        let delayed_r = buffer.get(ch_len + read_r).copied().unwrap_or(0.0);

        // Warm analog/tape feedback tone shaping: gentle lowpass + ~220 Hz highpass to keep mix crisp
        self.lp_l = flush(self.lp_l * 0.62 + delayed_l * 0.38);
        self.lp_r = flush(self.lp_r * 0.62 + delayed_r * 0.38);

        let hp_l = flush(0.972 * (self.hp_out_l + self.lp_l - self.hp_in_l));
        self.hp_in_l = self.lp_l;
        self.hp_out_l = hp_l;

        let hp_r = flush(0.972 * (self.hp_out_r + self.lp_r - self.hp_in_r));
        self.hp_in_r = self.lp_r;
        self.hp_out_r = hp_r;

        // Cross-coupled ping-pong feedback
        let fb = self.params.feedback;
        if let Some(slot_l) = buffer.get_mut(self.write_l) {
            *slot_l = flush((in_l + hp_r * fb).clamp(-2.0, 2.0));
        }
        if let Some(slot_r) = buffer.get_mut(ch_len + self.write_r) {
            *slot_r = flush((in_r + hp_l * fb).clamp(-2.0, 2.0));
        }

        self.write_l += 1;
        if self.write_l >= ch_len {
            self.write_l = 0;
        }
        self.write_r += 1;
        if self.write_r >= ch_len {
            self.write_r = 0;
        }

        let wet_l = delayed_l * self.params.wet;
        let wet_r = delayed_r * self.params.wet;
        if wet_l.abs() < QUIET_LEVEL && wet_r.abs() < QUIET_LEVEL && in_l == 0.0 && in_r == 0.0 {
            self.quiet_samples = self.quiet_samples.saturating_add(1);
        } else {
            self.quiet_samples = 0;
        }
        (wet_l, wet_r)
    }
}

fn flush(value: f32) -> f32 {
    if value.abs() < 1e-25 { 0.0 } else { value }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;

    #[test]
    fn ping_pong_alternates_channels() {
        let sr = 1_000_u32;
        let mut buf = vec![0.0; StereoDelay::buffer_len(sr)];
        let mut delay = StereoDelay::new(
            sr,
            DelayParams {
                left_samples: 10,
                right_samples: 15,
                feedback: 0.5,
                wet: 1.0,
            },
        );
        let _ = delay.process(&mut buf, 1.0, 0.0);
        let mut saw_left = false;
        let mut saw_right_pong = false;
        for step in 1..40 {
            let (l, r) = delay.process(&mut buf, 0.0, 0.0);
            if step == 10 && l.abs() > 0.5 {
                saw_left = true;
            }
            if step == 25 && r.abs() > 0.05 {
                saw_right_pong = true;
            }
        }
        assert!(saw_left, "left tap should sound at step 10");
        assert!(saw_right_pong, "right channel should receive ping-pong echo");
    }

    #[test]
    fn impulse_decays_to_quiet() {
        let sr = 4_000_u32;
        let mut buf = vec![0.0; StereoDelay::buffer_len(sr)];
        let mut delay = StereoDelay::new(
            sr,
            DelayParams {
                left_samples: 80,
                right_samples: 120,
                feedback: 0.35,
                wet: 0.5,
            },
        );
        let _ = delay.process(&mut buf, 0.9, 0.9);
        for _ in 0..sr * 8 {
            let _ = delay.process(&mut buf, 0.0, 0.0);
            if delay.is_quiet() {
                break;
            }
        }
        assert!(delay.is_quiet());
    }
}
