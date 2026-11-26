<script lang="ts">
  import { onMount } from 'svelte';
  import { composer } from '../stores/composer.svelte';
  import { editor } from '../stores/editor.svelte';
  import type { ComposeConfig } from '../ipc/composition';
  import type { ScaleKind } from '../ipc/types';
  import { SCALE_KINDS, scaleLabel } from '../music/options';
  import { className } from '../music/pitch';

  onMount(() => { void composer.init(); });
  const ROOTS = Array.from({ length: 12 }, (_, index) => index);
  const song = $derived(editor.song);
  type Field = Event & { currentTarget: HTMLInputElement | HTMLSelectElement };
  const updateSong = (change: Record<string, unknown>): void =>
    void editor.act({ kind: 'update_song', ...change } as Parameters<typeof editor.act>[0]);
  const features: { key: keyof Pick<ComposeConfig, 'motif_development' | 'voice_leading' | 'cadences' | 'chord_colors' | 'modal_mixture' | 'secondary_dominants' | 'suspensions' | 'expressive_slides'>; name: string; help: string }[] = [
    {key:'expressive_slides',name:'Slides and glissando',help:'Pitch scoops and a rising stepped solo in the climax.'},
    { key: 'motif_development', name: 'Develop a recurring hook', help: 'Related variations, a slower bridge and a faster climax.' },
    { key: 'voice_leading', name: 'Smooth melodic and chord movement', help: 'Prefer nearby notes and connected chord inversions.' },
    { key: 'cadences', name: 'Build tension and resolve endings', help: 'Coordinated phrase endings and a final home chord.' },
    { key: 'chord_colors', name: 'Sevenths and added ninths', help: 'Richer harmony for funk and reflective passages.' },
    { key: 'modal_mixture', name: 'Borrow a minor chord', help: 'A bittersweet minor iv in major-key bridges.' },
    { key: 'secondary_dominants', name: 'Pull toward the next chord', help: 'Chromatic dominant chords in the contrast section.' },
    { key: 'suspensions', name: 'Prepared tension and release', help: 'Hold a suitable bridge note before resolving by step.' },
  ];
  const roles = ['lead', 'harmony', 'bass', 'emotional', 'kick', 'snare', 'hat', 'arp', 'cowbell', 'crash'] as const;

  const acousticKits = [
    {
      label: '🎹 Grand Piano & Quartet',
      vibe: 'title_screen' as const,
      bpm: 108,
      swing: 4,
      instruments: {
        lead: 'acoustic_grand_piano',
        harmony: 'acoustic_string_quartet',
        bass: 'acoustic_upright_bass',
        emotional: 'acoustic_solo_cello',
        kick: 'kick',
        snare: 'acoustic_brush_snare',
        hat: 'hihat_closed',
      },
    },
    {
      label: '🕯️ Felt Piano & Harp',
      vibe: 'dungeon' as const,
      bpm: 96,
      swing: 0,
      instruments: {
        lead: 'studio_felt_piano',
        harmony: 'acoustic_concert_harp',
        bass: 'acoustic_upright_bass',
        emotional: 'acoustic_solo_violin',
        kick: 'kick',
        snare: 'acoustic_brush_snare',
        hat: 'hihat_closed',
      },
    },
    {
      label: '🎷 Jazz Piano & Vibes',
      vibe: 'funk' as const,
      bpm: 116,
      swing: 16,
      instruments: {
        lead: 'acoustic_upright_piano',
        harmony: 'vintage_rhodes_mark1',
        bass: 'acoustic_upright_bass',
        emotional: 'acoustic_vibraphone',
        kick: 'kick',
        snare: 'acoustic_brush_snare',
        hat: 'hihat_closed',
      },
    },
    {
      label: '🎸 Nylon Guitar & Flute',
      vibe: 'adventure' as const,
      bpm: 112,
      swing: 6,
      instruments: {
        lead: 'acoustic_nylon_guitar',
        harmony: 'acoustic_steel_guitar',
        bass: 'acoustic_upright_bass',
        emotional: 'acoustic_concert_flute',
        kick: 'kick',
        snare: 'acoustic_brush_snare',
        hat: 'hihat_closed',
      },
    },
  ];

  function applyAcousticKit(kit: (typeof acousticKits)[number]): void {
    if (!composer.config) return;
    composer.config.vibe = kit.vibe;
    composer.config.genre_style = 'auto';
    composer.config.melody_contour = 'auto';
    composer.config.motif_form = 'auto';
    composer.config.groove_template = 'auto';
    composer.config.accompaniment_style = 'auto';
    composer.config.bpm = kit.bpm;
    composer.config.swing = kit.swing;
    composer.config.rest_density = 35;
    composer.config.syncopation = 35;
    composer.config.slide_probability = 15;
    composer.config.chord_colors = true;
    composer.config.voice_leading = true;
    composer.config.cadences = true;
    composer.config.custom_sections = [];
    composer.config.instruments = { ...composer.config.instruments, ...kit.instruments };
  }

  const electronicKits = [
    {
      label: '⚡ Euphoric 138 Trance',
      vibe: 'adventure' as const,
      genre_style: 'euphoric_trance' as const,
      melody_contour: 'pedal_pivot' as const,
      motif_form: 'aaba' as const,
      groove_template: 'four_on_floor' as const,
      accompaniment_style: 'sidechain_pad' as const,
      key: 'F#',
      scale: 'natural_minor',
      bpm: 138,
      swing: 0,
      bass_hits: 4,
      rest_density: 32,
      syncopation: 45,
      slide_probability: 20,
      expressive_slides: true,
      instruments: {
        lead: 'jp8000_festival_supersaw',
        harmony: 'rave_anthem_vocal_pad',
        bass: 'cyber_reese_neuro_bass',
        emotional: 'insomnia_rave_pizz',
        kick: 'tr909_club_kick',
        snare: 'tr909_club_clap',
        hat: 'bw_tight_pocket_hat',
      },
    },
    {
      label: '🧪 DevilFish 303 Acid',
      vibe: 'battle' as const,
      genre_style: 'acid_warehouse_303' as const,
      melody_contour: 'offbeat_syncopated' as const,
      motif_form: 'abac' as const,
      groove_template: 'acid_warehouse' as const,
      accompaniment_style: 'minimal_gap_answer' as const,
      key: 'A',
      scale: 'phrygian',
      bpm: 136,
      swing: 4,
      bass_hits: 8,
      rest_density: 42,
      syncopation: 75,
      slide_probability: 65,
      expressive_slides: true,
      instruments: {
        lead: 'acid_303_lead',
        harmony: 'alpha_juno_hoover',
        bass: 'devilfish_303_acid',
        emotional: 'padsynth_lush_choir',
        kick: 'tr909_club_kick',
        snare: 'tr909_club_clap',
        hat: 'bw_tight_pocket_hat',
      },
    },
    {
      label: '🚀 Liquid DnB 174',
      vibe: 'adventure' as const,
      genre_style: 'liquid_dnb' as const,
      melody_contour: 'anacrusis_pickup' as const,
      motif_form: 'aaba' as const,
      groove_template: 'liquid_dnb_174' as const,
      accompaniment_style: 'sidechain_pad' as const,
      key: 'D',
      scale: 'natural_minor',
      bpm: 174,
      swing: 0,
      bass_hits: 4,
      rest_density: 38,
      syncopation: 50,
      slide_probability: 35,
      expressive_slides: true,
      instruments: {
        lead: 'cz_phase_lead',
        harmony: 'padsynth_ethereal_strings',
        bass: 'cyber_reese_neuro_bass',
        emotional: 'vocal_formant_chop',
        kick: 'tr909_club_kick',
        snare: 'tr909_club_snare',
        hat: 'bw_tight_pocket_hat',
      },
    },
    {
      label: '🎧 UK Garage 2-Step',
      vibe: 'funk' as const,
      genre_style: 'uk_garage_2step' as const,
      melody_contour: 'vocal_chop_syllabic' as const,
      motif_form: 'call_response' as const,
      groove_template: 'uk_garage_2step' as const,
      accompaniment_style: 'offbeat_stabs' as const,
      key: 'Bb',
      scale: 'natural_minor',
      bpm: 134,
      swing: 18,
      bass_hits: 6,
      rest_density: 48,
      syncopation: 80,
      slide_probability: 35,
      expressive_slides: true,
      instruments: {
        lead: 'vocal_formant_chop',
        harmony: 'm1_house_piano',
        bass: 'garage_sub_sine',
        emotional: 'uk_bassline_donk',
        kick: 'tr909_club_kick',
        snare: 'tr909_club_snare',
        hat: 'bw_tight_pocket_hat',
      },
    },
    {
      label: '🤖 French Touch Disco',
      vibe: 'funk' as const,
      genre_style: 'french_touch_disco' as const,
      melody_contour: 'blues_funk_riff' as const,
      motif_form: 'abac' as const,
      groove_template: 'french_disco' as const,
      accompaniment_style: 'offbeat_stabs' as const,
      key: 'A',
      scale: 'dorian',
      bpm: 124,
      swing: 8,
      bass_hits: 8,
      rest_density: 35,
      syncopation: 60,
      slide_probability: 30,
      expressive_slides: true,
      instruments: {
        lead: 'daft_talkbox_lead',
        harmony: 'french_disco_clav',
        bass: 'pc98_opna_slap_bass',
        emotional: 'padsynth_lush_choir',
        kick: 'tr909_club_kick',
        snare: 'tr909_club_clap',
        hat: 'bw_tight_pocket_hat',
      },
    },
    {
      label: '💥 Melbourne FM Donk',
      vibe: 'battle' as const,
      genre_style: 'melbourne_bounce' as const,
      melody_contour: 'rhythmic_morse_hook' as const,
      motif_form: 'aabb' as const,
      groove_template: 'melbourne_offbeat' as const,
      accompaniment_style: 'minimal_gap_answer' as const,
      key: 'Eb',
      scale: 'minor_pentatonic',
      bpm: 140,
      swing: 0,
      bass_hits: 4,
      rest_density: 44,
      syncopation: 68,
      slide_probability: 20,
      expressive_slides: false,
      instruments: {
        lead: 'insomnia_rave_pizz',
        harmony: 'jp8000_wide_chord_saw',
        bass: 'melbourne_offbeat_bass',
        emotional: 'alpha_juno_hoover',
        kick: 'tr909_club_kick',
        snare: 'tr909_club_clap',
        hat: 'bw_tight_pocket_hat',
      },
    },
    {
      label: '🦾 Cyberpunk Midtempo',
      vibe: 'boss' as const,
      genre_style: 'midtempo_cyberpunk' as const,
      melody_contour: 'wide_interval_leap' as const,
      motif_form: 'abac' as const,
      groove_template: 'midtempo_stomp' as const,
      accompaniment_style: 'chord_pulses' as const,
      key: 'E',
      scale: 'harmonic_minor',
      bpm: 112,
      swing: 0,
      bass_hits: 6,
      rest_density: 40,
      syncopation: 45,
      slide_probability: 45,
      expressive_slides: true,
      instruments: {
        lead: 'alpha_juno_hoover',
        harmony: 'padsynth_lush_choir',
        bass: 'cyber_reese_neuro_bass',
        emotional: 'cz_phase_lead',
        kick: 'tr909_club_kick',
        snare: 'tr909_club_snare',
        hat: 'bw_tight_pocket_hat',
      },
    },
    {
      label: '🎹 90s M1 Piano House',
      vibe: 'funk' as const,
      genre_style: 'piano_house_90s' as const,
      melody_contour: 'descending_cascade' as const,
      motif_form: 'aaba' as const,
      groove_template: 'house_m1' as const,
      accompaniment_style: 'piano_house_stabs' as const,
      key: 'C',
      scale: 'natural_minor',
      bpm: 128,
      swing: 10,
      bass_hits: 4,
      rest_density: 32,
      syncopation: 58,
      slide_probability: 15,
      expressive_slides: false,
      instruments: {
        lead: 'm1_house_piano',
        harmony: 'rave_anthem_vocal_pad',
        bass: 'm1_organ_bass',
        emotional: 'vocal_formant_chop',
        kick: 'tr909_club_kick',
        snare: 'tr909_club_clap',
        hat: 'bw_tight_pocket_hat',
      },
    },
    {
      label: '🌌 Afterlife Techno',
      vibe: 'dungeon' as const,
      genre_style: 'afterlife_techno' as const,
      melody_contour: 'dual_call_response' as const,
      motif_form: 'call_response' as const,
      groove_template: 'melodic_techno' as const,
      accompaniment_style: 'rising_arp' as const,
      key: 'G',
      scale: 'harmonic_minor',
      bpm: 126,
      swing: 0,
      bass_hits: 8,
      rest_density: 44,
      syncopation: 55,
      slide_probability: 30,
      expressive_slides: true,
      instruments: {
        lead: 'afterlife_techno_pluck',
        harmony: 'padsynth_ethereal_strings',
        bass: 'cyber_reese_neuro_bass',
        emotional: 'padsynth_lush_choir',
        kick: 'tr909_club_kick',
        snare: 'tr909_club_clap',
        hat: 'bw_tight_pocket_hat',
      },
    },
    {
      label: '🎺 Balkan Rave Brass',
      vibe: 'battle' as const,
      genre_style: 'balkan_festival_brass' as const,
      melody_contour: 'exotic_ornamental' as const,
      motif_form: 'abac' as const,
      groove_template: 'balkan_brass' as const,
      accompaniment_style: 'offbeat_stabs' as const,
      key: 'D',
      scale: 'harmonic_minor',
      bpm: 140,
      swing: 6,
      bass_hits: 8,
      rest_density: 30,
      syncopation: 65,
      slide_probability: 55,
      expressive_slides: true,
      instruments: {
        lead: 'balkan_zurna_lead',
        harmony: 'arabic_oud_pluck',
        bass: 'uk_bassline_donk',
        emotional: 'cyber_shamisen_pluck',
        kick: 'tr909_club_kick',
        snare: 'tr909_club_snare',
        hat: 'bw_tight_pocket_hat',
      },
    },
  ];

  function applyElectronicKit(kit: (typeof electronicKits)[number]): void {
    if (!composer.config) return;
    composer.config.vibe = kit.vibe;
    composer.config.genre_style = kit.genre_style;
    composer.config.melody_contour = kit.melody_contour;
    composer.config.motif_form = kit.motif_form;
    composer.config.groove_template = kit.groove_template;
    composer.config.accompaniment_style = kit.accompaniment_style;
    composer.config.key = kit.key;
    composer.config.scale = kit.scale;
    composer.config.bpm = kit.bpm;
    composer.config.swing = kit.swing;
    composer.config.bass_hits = kit.bass_hits;
    composer.config.rest_density = kit.rest_density;
    composer.config.syncopation = kit.syncopation;
    composer.config.slide_probability = kit.slide_probability;
    composer.config.expressive_slides = kit.expressive_slides;
    composer.config.chord_colors = true;
    composer.config.voice_leading = true;
    composer.config.cadences = true;
    composer.config.custom_sections = [];
    composer.config.instruments = { ...composer.config.instruments, ...kit.instruments };
  }
</script>

<div class="composer">
  {#if song}
    <div class="live-song-card">
      <span class="eyebrow">CURRENT SONG · LIVE TRANSFORM</span>
      <h3>{song.meta.title}</h3>
      <p class="muted">Change Tempo, Key, Scale, or Swing on the currently open song without overwriting its notes.</p>
      <div class="fields">
        <label>Tempo (BPM)
          <input type="number" min="20" max="400" value={song.timing.bpm}
            onchange={(e: Field) => updateSong({ bpm: Number(e.currentTarget.value) })} />
        </label>
        <label>Swing %
          <input type="number" min="0" max="100" value={song.timing.swing}
            onchange={(e: Field) => updateSong({ swing: Number(e.currentTarget.value) })} />
        </label>
        <label>Key (Root)
          <select value={String(song.scale.root)}
            onchange={(e: Field) => updateSong({ scale_root: Number(e.currentTarget.value), transpose_notes: true })}>
            {#each ROOTS as root (root)}<option value={String(root)}>{className(root)}</option>{/each}
          </select>
        </label>
        <label>Scale / Mode
          <select value={song.scale.kind}
            onchange={(e: Field) => updateSong({ scale_kind: e.currentTarget.value as ScaleKind, transpose_notes: true })}>
            {#each SCALE_KINDS as kind (kind)}<option value={kind}>{scaleLabel(kind)}</option>{/each}
          </select>
        </label>
      </div>
      <div class="transpose-strip">
        <span>Transpose:</span>
        <button class="ghost" onclick={() => updateSong({ semitone_offset: -12, transpose_notes: true })}>-12</button>
        <button class="ghost" onclick={() => updateSong({ semitone_offset: -1, transpose_notes: true })}>-1 st</button>
        <button class="ghost" onclick={() => updateSong({ semitone_offset: 1, transpose_notes: true })}>+1 st</button>
        <button class="ghost" onclick={() => updateSong({ semitone_offset: 12, transpose_notes: true })}>+12</button>
      </div>
    </div>
  {/if}
  <h2>Compose with CTG</h2>
  <p class="muted">Create an original song with your chosen sounds and settings, then edit every note.</p>
  {#if composer.config}
    {@const config = composer.config}
    <div class="actions discovery">
      <button disabled={composer.busy} onclick={() => void composer.generate()}>{composer.busy ? 'Composing…' : 'Generate song'}</button>
      <button disabled={composer.busy} onclick={() => void composer.randomSong()}>Random song</button>
      <button class="ghost" disabled={composer.busy} onclick={() => void composer.randomInstruments()}>Random instruments</button>
    </div>
    <div class="acoustic-kits">
      <span class="eyebrow">ELECTRONIC, RAVE & WORLD STUDIO KITS (10 GENRES)</span>
      <div class="kit-grid">
        {#each electronicKits as kit (kit.label)}
          <button class="ghost kit-btn" disabled={composer.busy} onclick={() => applyElectronicKit(kit)}>{kit.label}</button>
        {/each}
      </div>
    </div>
    <div class="acoustic-kits">
      <span class="eyebrow">REALISTIC PIANO & ACOUSTIC STUDIO KITS</span>
      <div class="kit-grid">
        {#each acousticKits as kit (kit.label)}
          <button class="ghost kit-btn" disabled={composer.busy} onclick={() => applyAcousticKit(kit)}>{kit.label}</button>
        {/each}
      </div>
    </div>
    <p class="muted">Generate song uses your settings. Random song chooses new settings and creates the song. Then press Play. Undo restores your previous song.</p>
    <p class="muted">Random instruments chooses suitable sounds for the next generation. Open Instruments to see them, then Generate song to apply.</p>
    <div class="fields">
      <label class="wide">Song title<input bind:value={config.title} maxlength="160" /></label>
      <label>Character<select bind:value={config.vibe}>
        <option value="funk">Funk / emotional</option><option value="adventure">Adventure</option>
        <option value="battle">Battle</option><option value="dungeon">Atmospheric</option>
        <option value="title_screen">Title theme</option><option value="boss">Boss</option>
      </select></label>
      <label>Genre blueprint<select bind:value={config.genre_style}>
        <option value="auto">Auto (use Character)</option>
        <option value="initial_d_eurobeat">Initial D Eurobeat (156 BPM)</option>
        <option value="anime_opening_techno">Fast Anime OP Techno (166 BPM)</option>
        <option value="uprising_melodic_techno">Uprising Melodic Techno</option>
        <option value="hyper_eurodance_rave">Super Eurodance Rave</option>
        <option value="euphoric_trance">Euphoric 138 Trance</option>
        <option value="acid_warehouse_303">DevilFish 303 Acid</option>
        <option value="liquid_dnb">Liquid DnB 174</option>
        <option value="uk_garage_2step">UK Garage 2-Step</option>
        <option value="french_touch_disco">French Touch Disco</option>
        <option value="melbourne_bounce">Melbourne FM Donk</option>
        <option value="midtempo_cyberpunk">Cyberpunk Midtempo</option>
        <option value="piano_house_90s">90s M1 Piano House</option>
        <option value="afterlife_techno">Afterlife Techno</option>
        <option value="balkan_festival_brass">Balkan Festival Brass</option>
      </select></label>
      <label>Form<select bind:value={config.form}><option value="short">16 bars</option><option value="standard">32 bars</option><option value="extended">48 bars</option><option value="long">64 bars</option><option value="epic">96 bars</option><option value="marathon">128 bars</option></select></label>
      <label>Key<select bind:value={config.key}>{#each ['C','C#','D','Eb','E','F','F#','G','Ab','A','Bb','B'] as key (key)}<option>{key}</option>{/each}</select></label>
      <label>Scale<select bind:value={config.scale}>
        <option value="major">Major</option><option value="natural_minor">Natural minor</option><option value="harmonic_minor">Harmonic minor</option>
        <option value="melodic_minor">Melodic minor</option><option value="dorian">Dorian</option><option value="phrygian">Phrygian</option>
        <option value="lydian">Lydian</option><option value="mixolydian">Mixolydian</option>
        <option value="major_pentatonic">Major pentatonic</option><option value="minor_pentatonic">Minor pentatonic</option><option value="blues">Blues</option>
      </select></label>
      <label>Tempo<input type="number" min="40" max="240" bind:value={config.bpm} /></label>
      <label>Swing %<input type="number" min="0" max="60" bind:value={config.swing} /></label>
      <label>Bass rhythm<select bind:value={config.bass_rhythm}><option value="mixed">Mixed</option><option value="template">Groove templates</option><option value="euclidean">Euclidean</option></select></label>
      <label>Bass hits / bar<input type="number" min="0" max="16" bind:value={config.bass_hits} /></label>
      <label>Seed<input type="number" min="0" max="4294967295" step="1" bind:value={config.seed} /></label>
      <button class="ghost" onclick={() => { config.seed = Math.floor(Math.random() * 4294967296); }}>New seed</button>
    </div>
    <details open><summary>Melody, contour & groove lab</summary>
      <div class="fields">
        <label>Melody contour (10 modes)<select bind:value={config.melody_contour}>
          <option value="auto">Auto (Seed/Genre)</option>
          <option value="pedal_pivot">Pedal Pivot (Sandstorm/PPK)</option>
          <option value="descending_cascade">Descending Cascade</option>
          <option value="anacrusis_pickup">Anacrusis Pickup & Hold</option>
          <option value="offbeat_syncopated">Offbeat Syncopated (Rest Beat 1)</option>
          <option value="dual_call_response">Dual Instrument Call & Response</option>
          <option value="vocal_chop_syllabic">Vocal Chop Syllabic</option>
          <option value="exotic_ornamental">Exotic Modal Ornamental</option>
          <option value="rhythmic_morse_hook">Hypnotic Morse Hook</option>
          <option value="wide_interval_leap">Wide Interval Leap</option>
          <option value="blues_funk_riff">Blues / Funk Pocket Riff</option>
        </select></label>
        <label>Motif grammar<select bind:value={config.motif_form}>
          <option value="auto">Auto</option>
          <option value="aaba">AABA (Contrast B)</option>
          <option value="abac">ABAC (Turnaround C)</option>
          <option value="aabb">AABB (Double Statement)</option>
          <option value="call_response">Call & Response</option>
        </select></label>
        <label>Groove template<select bind:value={config.groove_template}>
          <option value="auto">Auto</option>
          <option value="four_on_floor">Four on the Floor</option>
          <option value="acid_warehouse">Acid 909 Warehouse</option>
          <option value="liquid_dnb_174">Liquid DnB 174 2-Step</option>
          <option value="uk_garage_2step">UK Garage 2-Step Shuffle</option>
          <option value="french_disco">French Touch Disco</option>
          <option value="melbourne_offbeat">Melbourne Offbeat Stomp</option>
          <option value="midtempo_stomp">Cyberpunk Midtempo Stomp</option>
          <option value="house_m1">90s Piano House</option>
          <option value="melodic_techno">Afterlife Melodic Techno</option>
          <option value="balkan_brass">Balkan Festival Groove</option>
          <option value="eurobeat_156">Initial D Eurobeat 156</option>
          <option value="anime_techno_166">Fast Anime Techno 166</option>
          <option value="syncopated">Syncopated Pocket</option>
          <option value="breakbeat">Breakbeat</option>
          <option value="tresillo">Tresillo</option>
          <option value="half_time">Half-time</option>
          <option value="driving">Driving 16ths</option>
        </select></label>
        <label>Accompaniment mode<select bind:value={config.accompaniment_style}>
          <option value="auto">Auto</option>
          <option value="sidechain_pad">Sidechained Stereo Pad</option>
          <option value="piano_house_stabs">M1 Piano House Stabs</option>
          <option value="minimal_gap_answer">Minimal Gap Answer (Anti-Crowd)</option>
          <option value="offbeat_stabs">Offbeat Rave Stabs</option>
          <option value="chord_pulses">Half-Bar Chord Pulses</option>
          <option value="broken_chords">Syncopated Dyad Comping</option>
          <option value="pendulum">Interlocking Gap Answer</option>
          <option value="rising_arp">Polyrhythmic 3-over-4 Pluck</option>
          <option value="falling_arp">Turnaround Cascade</option>
        </select></label>
        <label>Rest density: {config.rest_density}%
          <input type="range" min="0" max="80" step="5" bind:value={config.rest_density} />
        </label>
        <label>Syncopation: {config.syncopation}%
          <input type="range" min="0" max="100" step="5" bind:value={config.syncopation} />
        </label>
        <label>Slide prob: {config.slide_probability}%
          <input type="range" min="0" max="100" step="5" bind:value={config.slide_probability} />
        </label>
      </div>
    </details>
    <details><summary>Musical ideas</summary>
      <div class="features">
        {#each features as feature (feature.key)}
          <label title={feature.help}><input type="checkbox" bind:checked={config[feature.key]} />{feature.name}</label>
        {/each}
        <label title="Modulate the final chorus +2 semitones for an anime/eurobeat climax"><input type="checkbox" bind:checked={config.key_change_finale} />Key-change Finale (+2 st)</label>
        <label title="Add accelerating 8th->16th snare buildup rolls before choruses"><input type="checkbox" bind:checked={config.riser_builds} />Snare Riser Buildups</label>
        <label title="Enable Sparkle Arp, Eurobeat Cowbell & Crash Cymbal studio lanes"><input type="checkbox" bind:checked={config.studio_layers} />10-Track Studio Layers</label>
      </div>
    </details>
    <details><summary>Instruments · {editor.presets.length} available sounds</summary>
      <div class="fields">{#each roles as role (role)}
        <label>{role === 'emotional' ? 'Solo / Emotional lead' : role}<select bind:value={config.instruments[role]}>
          {#each editor.presets as preset (preset.id)}<option value={preset.id}>{preset.name}</option>{/each}
        </select></label>
      {/each}</div>
    </details>
    <p class="muted">Generate replaces the current arrangement. Undo restores it; files change only when you save.</p>
    <div class="actions">
      <button class="ghost" disabled={composer.busy} onclick={() => void composer.preview()}>Preview chords</button>
    </div>
    {#if composer.report}
      <details><summary>Last result · {composer.report.bars.length} bars · {composer.report.duration_seconds.toFixed(1)} seconds</summary>
        <p>Hook degrees: {composer.report.motif.map((n) => n + 1).join(' · ')}</p>
        <p>{composer.report.design.melody_contour.replaceAll('_', ' ')} ({composer.report.design.motif_form.toUpperCase()}) · {composer.report.design.groove.replaceAll('_', ' ')} · {composer.report.design.accompaniment.replaceAll('_', ' ')}</p>
        <p>{composer.report.design.sections.map((s) => `${s.role} ${s.bars}`).join(' → ')}</p>
        <div class="analysis">{#each composer.report.bars as bar (bar.bar)}<div><span>{bar.bar + 1}. {bar.role}</span><strong>{bar.chord}</strong></div>{/each}</div>
      </details>
    {/if}
  {/if}
  {#if composer.error}<p role="alert">{composer.error}</p>{/if}
</div>

<style>
  .composer { width: 100%; min-width: 0; }
  h2 { margin: 0; font-size: 18px; }
  p { font-size: 12px; line-height: 1.5; }
  .fields { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin: 12px 0; }
  .fields label { display: flex; flex-direction: column; gap: 4px; font-size: 12px; }
  .wide { grid-column: 1 / -1; }
  input, select { min-width: 0; width: 100%; }
  details { border-top: 1px solid var(--border); padding: 10px 0; }
  summary { cursor: pointer; font-size: 13px; }
  .features { display: grid; gap: 8px; padding-top: 12px; }
  .features label { display: flex; align-items: center; gap: 8px; font-size: 12px; }
  .features input { width: auto; }
  .actions { display: flex; justify-content: flex-end; gap: 8px; }
  .discovery { justify-content: flex-start; flex-wrap: wrap; margin-top: 12px; }
  .analysis { max-height: 200px; overflow: auto; font-size: 12px; }
  .analysis div { display: flex; justify-content: space-between; padding: 4px; }
  [role='alert'] { color: var(--danger, #e77777); }
  .live-song-card {
    padding: 12px;
    margin-bottom: 16px;
    border: 1px solid var(--accent);
    border-radius: 10px;
    background: color-mix(in srgb, var(--accent) 7%, var(--bg));
  }
  .live-song-card h3 {
    margin: 4px 0 4px;
    font-size: 14px;
    color: var(--text);
  }
  .live-song-card .fields {
    margin: 8px 0;
  }
  .transpose-strip {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--muted);
  }
  .transpose-strip button {
    flex: 1;
    font-size: 11px;
    padding: 4px 6px;
  }
  .acoustic-kits {
    margin: 12px 0;
    padding: 10px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: color-mix(in srgb, var(--accent) 4%, var(--bg));
  }
  .kit-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
    margin-top: 6px;
  }
  .kit-btn {
    font-size: 11px;
    padding: 6px 8px;
    text-align: left;
  }
</style>
