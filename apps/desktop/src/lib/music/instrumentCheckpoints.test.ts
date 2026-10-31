import { describe, expect, it } from 'vitest';
import type { Instrument } from '../ipc/types';
import { InstrumentCheckpoints } from './instrumentCheckpoints';
import { defaultVoice } from './voices';

const sound = (name: string): Instrument => ({ id: 1, name, voice: defaultVoice('triangle'),
  envelope: { attack: 3, decay: 120, sustain: 65, release: 80 },
  pitch_envelope: { semitones: 0, time: 0 }, vibrato: { depth: 0, rate: 50, delay: 0 },
  unison: { voices: 1, detune: 0 }, filter: { mode: 'off', cutoff: 20000, resonance: 0, envelope_depth: 0 },
  volume: 70, pan: 0, reverb_send: 0 });

describe('instrument reset points', () => {
  it('keeps the opened sound through edits and returns independent copies', () => {
    const points = new InstrumentCheckpoints(); const original = sound('Bass');
    points.observe(1, [original]); original.envelope.attack = 80;
    points.observe(1, [original]);
    expect(points.get(1)?.envelope.attack).toBe(3);
    const reset = points.get(1); if (reset) reset.volume = 0;
    expect(points.get(1)?.volume).toBe(70);
  });
  it('uses a saved sound as the next reset point', () => {
    const points = new InstrumentCheckpoints(); points.observe(1, [sound('Original')]);
    points.save(sound('Saved')); points.observe(1, [sound('Later edit')]);
    expect(points.get(1)?.name).toBe('Saved');
  });
  it('does not carry instrument ids into a different or generated song', () => {
    const points = new InstrumentCheckpoints(); points.observe(1, [sound('First')]);
    points.observe(2, [sound('Other project')]); expect(points.get(1)?.name).toBe('Other project');
    points.replace(2, [sound('Generated')]); expect(points.get(1)?.name).toBe('Generated');
  });
});
