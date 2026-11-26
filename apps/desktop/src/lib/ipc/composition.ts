// Shared composition recipe and analysis returned by the native engine.
export type Vibe = 'adventure' | 'battle' | 'dungeon' | 'title_screen' | 'boss' | 'funk';
export type GenreStyle =
  | 'auto'
  | 'euphoric_trance'
  | 'acid_warehouse_303'
  | 'liquid_dnb'
  | 'uk_garage_2step'
  | 'french_touch_disco'
  | 'melbourne_bounce'
  | 'midtempo_cyberpunk'
  | 'piano_house_90s'
  | 'afterlife_techno'
  | 'balkan_festival_brass'
  | 'initial_d_eurobeat'
  | 'anime_opening_techno'
  | 'uprising_melodic_techno'
  | 'hyper_eurodance_rave';
export type ChordProgressionStyle =
  | 'auto'
  | 'royal_road'
  | 'eurobeat_circle'
  | 'uprising_anthem'
  | 'komuro_drive'
  | 'dramatic_minor';
export type SectionKind =
  | 'intro_riff'
  | 'verse_a'
  | 'verse_b'
  | 'pre_chorus'
  | 'chorus'
  | 'solo'
  | 'breakdown'
  | 'final_chorus'
  | 'outro';
export interface SectionSpec {
  kind: SectionKind;
  bars: number;
  melody_theme: number;
  progression: ChordProgressionStyle;
  energy: number;
  transpose_semitones: number;
  seed_offset: number;
}
export type MelodyContour =
  | 'auto'
  | 'pedal_pivot'
  | 'descending_cascade'
  | 'anacrusis_pickup'
  | 'offbeat_syncopated'
  | 'dual_call_response'
  | 'vocal_chop_syllabic'
  | 'exotic_ornamental'
  | 'rhythmic_morse_hook'
  | 'wide_interval_leap'
  | 'blues_funk_riff';
export type MotifForm = 'auto' | 'aaba' | 'abac' | 'aabb' | 'call_response';
export type GrooveTemplate =
  | 'auto'
  | 'four_on_floor'
  | 'syncopated'
  | 'breakbeat'
  | 'tresillo'
  | 'half_time'
  | 'driving'
  | 'acid_warehouse'
  | 'liquid_dnb_174'
  | 'uk_garage_2step'
  | 'french_disco'
  | 'melbourne_offbeat'
  | 'midtempo_stomp'
  | 'house_m1'
  | 'melodic_techno'
  | 'balkan_brass'
  | 'eurobeat_156'
  | 'anime_techno_166';
export type AccompanimentStyle =
  | 'auto'
  | 'chord_pulses'
  | 'offbeat_stabs'
  | 'broken_chords'
  | 'pendulum'
  | 'rising_arp'
  | 'falling_arp'
  | 'sidechain_pad'
  | 'piano_house_stabs'
  | 'minimal_gap_answer';
export interface ComposeConfig {
  title: string;
  seed: number;
  vibe: Vibe;
  key: string;
  scale: string;
  bpm: number;
  form: 'short' | 'standard' | 'extended' | 'long' | 'epic' | 'marathon';
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
  genre_style: GenreStyle;
  melody_contour: MelodyContour;
  motif_form: MotifForm;
  groove_template: GrooveTemplate;
  accompaniment_style: AccompanimentStyle;
  rest_density: number;
  syncopation: number;
  slide_probability: number;
  progression_style: ChordProgressionStyle;
  key_change_finale: boolean;
  riser_builds: boolean;
  studio_layers: boolean;
  custom_sections: SectionSpec[];
  instruments: Record<
    'lead' | 'harmony' | 'bass' | 'emotional' | 'kick' | 'snare' | 'hat' | 'arp' | 'cowbell' | 'crash',
    string
  >;
}
export interface CompositionReport {
  composer_version?: number;
  config: ComposeConfig;
  motif: number[];
  design: {
    groove: string;
    accompaniment: string;
    bass_motion: string;
    melody_contour: string;
    motif_form: string;
    phrase_bars: number;
    bars_per_chord: number;
    sections: {
      role: string;
      bars: number;
      kind?: SectionKind;
      melody_theme?: number;
      energy?: number;
      transpose_semitones?: number;
    }[];
  };
  duration_seconds: number;
  bars: { bar: number; role: string; chord: string; voicing: number[]; attacks: number[] }[];
}
