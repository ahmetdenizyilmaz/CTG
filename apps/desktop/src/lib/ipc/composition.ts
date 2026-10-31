// Shared composition recipe and analysis returned by the native engine.
export type Vibe = 'adventure' | 'battle' | 'dungeon' | 'title_screen' | 'boss' | 'funk';
export interface ComposeConfig {
  title: string;
  seed: number;
  vibe: Vibe;
  key: string;
  scale: string;
  bpm: number;
  form: 'short' | 'standard' | 'extended';
  swing: number;
  bass_rhythm: 'mixed' | 'template' | 'euclidean';
  bass_hits: number;
  motif_development: boolean;
  voice_leading: boolean;
  cadences: boolean;
  chord_colors: boolean;
  modal_mixture: boolean;
  secondary_dominants: boolean;
  suspensions: boolean;
  neon_funk: boolean;
  expressive_slides: boolean;
  instruments: Record<'lead' | 'harmony' | 'bass' | 'emotional' | 'kick' | 'snare' | 'hat', string>;
}
export interface CompositionReport {
  config: ComposeConfig;
  motif: number[];
  design: { groove: string; accompaniment: string; bass_motion: string; phrase_bars: number; bars_per_chord: number; sections: { role: string; bars: number }[] };
  duration_seconds: number;
  bars: { bar: number; role: string; chord: string; voicing: number[]; attacks: number[] }[];
}
