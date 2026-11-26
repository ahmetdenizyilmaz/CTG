<script lang="ts">
  import { onMount } from 'svelte';
  import { composer } from '../stores/composer.svelte';
  import { editor } from '../stores/editor.svelte';
  import { transport } from '../stores/transport.svelte';
  import { workspace } from '../stores/workspace.svelte';
  import type {
    ChordProgressionStyle,
    GenreStyle,
    GrooveTemplate,
    SectionKind,
    SectionSpec,
  } from '../ipc/composition';

  onMount(() => {
    void composer.init();
  });

  const SECTION_META: Record<SectionKind, { label: string; color: string; short: string }> = {
    intro_riff: { label: 'Intro Riff', color: '#f97316', short: 'INTRO' },
    verse_a: { label: 'Verse A', color: '#38bdf8', short: 'VERSE A' },
    verse_b: { label: 'Verse B Continuation', color: '#06b6d4', short: 'VERSE B' },
    pre_chorus: { label: 'Pre-Chorus Ascent', color: '#a855f7', short: 'BUILD' },
    chorus: { label: 'Chorus (Sabi)', color: '#ec4899', short: 'CHORUS' },
    solo: { label: 'Guitar / Synth Solo', color: '#eab308', short: 'SOLO' },
    breakdown: { label: 'Emotional Breakdown', color: '#6366f1', short: 'BREAK' },
    final_chorus: { label: 'Final Chorus Drop', color: '#ef4444', short: 'FINAL' },
    outro: { label: 'Outro', color: '#64748b', short: 'OUTRO' },
  };

  const THEME_LABELS = [
    'Theme 0 · Explosive Riff / Brass Hook',
    'Theme 1 · Verse A Singing Melody',
    'Theme 2 · Verse B Higher Continuation',
    'Theme 3 · Pre-Chorus Uprising Ascent',
    'Theme 4 · Soaring Anime / Eurobeat Sabi',
    'Theme 5 · Blazing Touge Solo Run',
  ];

  const PROGRESSION_LABELS: { id: ChordProgressionStyle; label: string }[] = [
    { id: 'auto', label: 'Smart Section Auto' },
    { id: 'royal_road', label: 'Royal Road (Bb-C-Am-Dm / IV-V-iii-vi)' },
    { id: 'eurobeat_circle', label: 'Initial D Circle of 5ths' },
    { id: 'uprising_anthem', label: 'Uprising Minor Anthem (i-VI-III-VII)' },
    { id: 'komuro_drive', label: 'Komuro Drive (VI-VII-i)' },
    { id: 'dramatic_minor', label: 'Pre-Chorus Ascent -> V7' },
  ];

  interface SemiProBlueprint {
    name: string;
    subtitle: string;
    badge: string;
    title: string;
    genre_style: GenreStyle;
    groove_template: GrooveTemplate;
    progression_style: ChordProgressionStyle;
    bpm: number;
    key: string;
    scale: string;
    swing: number;
    key_change_finale: boolean;
    riser_builds: boolean;
    studio_layers: boolean;
    instruments: Record<
      'lead' | 'harmony' | 'bass' | 'emotional' | 'kick' | 'snare' | 'hat' | 'arp' | 'cowbell' | 'crash',
      string
    >;
    sections: SectionSpec[];
  }

  const std7Sections = (prog: ChordProgressionStyle, keyShift: number): SectionSpec[] => [
    { kind: 'intro_riff', bars: 4, melody_theme: 0, progression: 'eurobeat_circle', energy: 5, transpose_semitones: 0, seed_offset: 0 },
    { kind: 'verse_a', bars: 8, melody_theme: 1, progression: prog, energy: 3, transpose_semitones: 0, seed_offset: 0 },
    { kind: 'verse_b', bars: 8, melody_theme: 2, progression: 'komuro_drive', energy: 4, transpose_semitones: 0, seed_offset: 0 },
    { kind: 'pre_chorus', bars: 4, melody_theme: 3, progression: 'dramatic_minor', energy: 4, transpose_semitones: 0, seed_offset: 0 },
    { kind: 'chorus', bars: 8, melody_theme: 4, progression: 'royal_road', energy: 5, transpose_semitones: 0, seed_offset: 0 },
    { kind: 'solo', bars: 8, melody_theme: 5, progression: 'eurobeat_circle', energy: 5, transpose_semitones: 0, seed_offset: 0 },
    { kind: 'final_chorus', bars: 8, melody_theme: 4, progression: 'royal_road', energy: 5, transpose_semitones: keyShift, seed_offset: 0 },
  ];

  const BLUEPRINTS: SemiProBlueprint[] = [
    {
      name: 'Akina Touge Eurobeat',
      subtitle: 'Initial D 156 BPM · Synth Brass Riff, Octave Bass, Cowbell & Guitar Solo',
      badge: 'INITIAL D',
      title: 'Akina Midnight Drift (Eurobeat)',
      genre_style: 'initial_d_eurobeat',
      groove_template: 'eurobeat_156',
      progression_style: 'eurobeat_circle',
      bpm: 156,
      key: 'D',
      scale: 'natural_minor',
      swing: 0,
      key_change_finale: true,
      riser_builds: true,
      studio_layers: true,
      instruments: {
        lead: 'eurobeat_synth_brass',
        harmony: 'anime_grand_chords',
        bass: 'eurobeat_octave_bass',
        emotional: 'anime_rock_guitar',
        kick: 'tr909_club_kick',
        snare: 'tr909_club_snare',
        hat: 'bw_tight_pocket_hat',
        arp: 'anime_sparkle_arp',
        cowbell: 'eurobeat_cowbell',
        crash: 'club_crash_cymbal',
      },
      sections: std7Sections('eurobeat_circle', 2),
    },
    {
      name: 'Fast Anime OP Techno',
      subtitle: '166 BPM · Uprising Royal Road Sabi, Sparkle Arp & Snare Builds',
      badge: 'ANIME OP',
      title: 'Celestial Overdrive (Anime OP)',
      genre_style: 'anime_opening_techno',
      groove_template: 'anime_techno_166',
      progression_style: 'royal_road',
      bpm: 166,
      key: 'F#',
      scale: 'natural_minor',
      swing: 0,
      key_change_finale: true,
      riser_builds: true,
      studio_layers: true,
      instruments: {
        lead: 'anime_sabi_supersaw',
        harmony: 'anime_grand_chords',
        bass: 'uprising_techno_bass',
        emotional: 'touge_power_guitar',
        kick: 'tr909_club_kick',
        snare: 'tr909_club_clap',
        hat: 'bw_tight_pocket_hat',
        arp: 'anime_sparkle_arp',
        cowbell: 'eurobeat_cowbell',
        crash: 'club_crash_cymbal',
      },
      sections: std7Sections('royal_road', 2),
    },
    {
      name: 'Uprising Melodic Techno',
      subtitle: '144 BPM · Cascading Synth Climb, Sidechained Strings & Key-Change Finale',
      badge: 'UPRISING',
      title: 'Skyline Ascent (Uprising Techno)',
      genre_style: 'uprising_melodic_techno',
      groove_template: 'melodic_techno',
      progression_style: 'uprising_anthem',
      bpm: 144,
      key: 'A',
      scale: 'natural_minor',
      swing: 0,
      key_change_finale: true,
      riser_builds: true,
      studio_layers: true,
      instruments: {
        lead: 'uprising_trance_lead',
        harmony: 'padsynth_ethereal_strings',
        bass: 'uprising_techno_bass',
        emotional: 'anime_rock_guitar',
        kick: 'tr909_club_kick',
        snare: 'tr909_club_clap',
        hat: 'bw_tight_pocket_hat',
        arp: 'anime_sparkle_arp',
        cowbell: 'eurobeat_cowbell',
        crash: 'club_crash_cymbal',
      },
      sections: std7Sections('uprising_anthem', 2),
    },
    {
      name: '90s Super Eurodance',
      subtitle: '148 BPM · Punchy M1 Piano Stabs, Brass Riff, Cowbell & Euphoric Sabi',
      badge: 'EURODANCE',
      title: 'Night of Neon Fire (Eurodance)',
      genre_style: 'hyper_eurodance_rave',
      groove_template: 'four_on_floor',
      progression_style: 'komuro_drive',
      bpm: 148,
      key: 'C',
      scale: 'natural_minor',
      swing: 0,
      key_change_finale: true,
      riser_builds: true,
      studio_layers: true,
      instruments: {
        lead: 'initial_d_brass_stab',
        harmony: 'm1_house_piano',
        bass: 'eurobeat_octave_bass',
        emotional: 'vocal_formant_chop',
        kick: 'tr909_club_kick',
        snare: 'tr909_club_clap',
        hat: 'bw_tight_pocket_hat',
        arp: 'anime_sparkle_arp',
        cowbell: 'eurobeat_cowbell',
        crash: 'club_crash_cymbal',
      },
      sections: std7Sections('komuro_drive', 1),
    },
    {
      name: 'Touge Hyper-Techno',
      subtitle: '152 BPM · Phrygian-Minor Aggressive Synth Brass & Power Guitar Duel',
      badge: 'TOUGE',
      title: 'Redline Phrygian Rush',
      genre_style: 'initial_d_eurobeat',
      groove_template: 'eurobeat_156',
      progression_style: 'eurobeat_circle',
      bpm: 152,
      key: 'E',
      scale: 'harmonic_minor',
      swing: 0,
      key_change_finale: true,
      riser_builds: true,
      studio_layers: true,
      instruments: {
        lead: 'eurobeat_synth_brass',
        harmony: 'jp8000_wide_chord_saw',
        bass: 'eurobeat_octave_bass',
        emotional: 'touge_power_guitar',
        kick: 'tr909_club_kick',
        snare: 'tr909_club_snare',
        hat: 'bw_tight_pocket_hat',
        arp: 'anime_sparkle_arp',
        cowbell: 'eurobeat_cowbell',
        crash: 'club_crash_cymbal',
      },
      sections: std7Sections('eurobeat_circle', 2),
    },
    {
      name: 'Anime DnB Sabi Rush',
      subtitle: '174 BPM · 2-Step Breakbeat + Royal Road + High-Altitude Supersaw & Guitar',
      badge: '174 DNB',
      title: 'Velocity Starlight (Anime DnB)',
      genre_style: 'anime_opening_techno',
      groove_template: 'liquid_dnb_174',
      progression_style: 'royal_road',
      bpm: 174,
      key: 'B',
      scale: 'natural_minor',
      swing: 0,
      key_change_finale: true,
      riser_builds: true,
      studio_layers: true,
      instruments: {
        lead: 'uprising_trance_lead',
        harmony: 'padsynth_lush_choir',
        bass: 'cyber_reese_neuro_bass',
        emotional: 'anime_rock_guitar',
        kick: 'tr909_club_kick',
        snare: 'tr909_club_snare',
        hat: 'bw_tight_pocket_hat',
        arp: 'anime_sparkle_arp',
        cowbell: 'eurobeat_cowbell',
        crash: 'club_crash_cymbal',
      },
      sections: std7Sections('royal_road', 2),
    },
  ];

  let selectedIdx = $state(0);

  function ensureEditableSections(): SectionSpec[] {
    if (!composer.config) return [];
    if (!composer.config.custom_sections.length) {
      composer.config.custom_sections = std7Sections('royal_road', composer.config.key_change_finale ? 2 : 0).map((s) => ({ ...s }));
    }
    return composer.config.custom_sections;
  }

  function applyBlueprint(bp: SemiProBlueprint, andGenerate = false): void {
    if (!composer.config) return;
    composer.config.title = bp.title;
    composer.config.genre_style = bp.genre_style;
    composer.config.groove_template = bp.groove_template;
    composer.config.progression_style = bp.progression_style;
    composer.config.bpm = bp.bpm;
    composer.config.key = bp.key;
    composer.config.scale = bp.scale;
    composer.config.swing = bp.swing;
    composer.config.key_change_finale = bp.key_change_finale;
    composer.config.riser_builds = bp.riser_builds;
    composer.config.studio_layers = bp.studio_layers;
    composer.config.cadences = true;
    composer.config.voice_leading = true;
    composer.config.chord_colors = true;
    composer.config.instruments = { ...bp.instruments };
    composer.config.custom_sections = bp.sections.map((s) => ({ ...s }));
    selectedIdx = 0;
    if (andGenerate) {
      composer.config.seed = Math.floor(Math.random() * 4294967296);
      void composer.generate();
    }
  }

  function addSection(kind: SectionKind): void {
    if (!composer.config) return;
    const list = ensureEditableSections();
    const themeMap: Record<SectionKind, number> = {
      intro_riff: 0,
      verse_a: 1,
      verse_b: 2,
      pre_chorus: 3,
      chorus: 4,
      solo: 5,
      breakdown: 2,
      final_chorus: 4,
      outro: 0,
    };
    list.push({
      kind,
      bars: kind === 'pre_chorus' || kind === 'intro_riff' || kind === 'outro' ? 4 : 8,
      melody_theme: themeMap[kind],
      progression: 'auto',
      energy: kind === 'breakdown' ? 2 : kind === 'verse_a' ? 3 : kind === 'verse_b' || kind === 'pre_chorus' ? 4 : 5,
      transpose_semitones: kind === 'final_chorus' && composer.config.key_change_finale ? 2 : 0,
      seed_offset: Math.floor(Math.random() * 999),
    });
    selectedIdx = list.length - 1;
  }

  function moveSection(index: number, delta: number): void {
    const list = composer.config?.custom_sections;
    if (!list) return;
    const target = index + delta;
    if (target < 0 || target >= list.length) return;
    const copy = [...list];
    const [item] = copy.splice(index, 1);
    if (!item) return;
    copy.splice(target, 0, item);
    if (composer.config) composer.config.custom_sections = copy;
    selectedIdx = target;
  }

  function removeSection(index: number): void {
    const list = composer.config?.custom_sections;
    if (!list || list.length <= 1) return;
    list.splice(index, 1);
    selectedIdx = Math.min(index, list.length - 1);
  }

  function rerollSectionMelody(index: number): void {
    const sec = composer.config?.custom_sections[index];
    if (!sec) return;
    sec.seed_offset = (sec.seed_offset + 1 + Math.floor(Math.random() * 1000)) >>> 0;
    void composer.generate();
  }

  function freshAllMelodies(): void {
    if (!composer.config) return;
    composer.config.seed = Math.floor(Math.random() * 4294967296);
    if (!composer.config.genre_style || composer.config.genre_style === 'auto') {
      composer.config.genre_style = 'initial_d_eurobeat';
    }
    void composer.generate();
  }

  const totalBars = $derived(
    composer.config?.custom_sections.reduce((acc, s) => acc + s.bars, 0) ?? 0,
  );
  const estSeconds = $derived(
    composer.config ? Math.round((totalBars * 4 * 60) / Math.max(40, composer.config.bpm)) : 0,
  );
  const activeSection = $derived(
    composer.config?.custom_sections[selectedIdx] ?? composer.config?.custom_sections[0] ?? null,
  );
</script>

<section class="studio-page">
  <header class="hero">
    <div>
      <span class="eyebrow">SEMI-PRO MULTI-SECTION SONG ENGINE · 253 STUDIO SOUNDS · POLYBLEP &amp; PING-PONG DELAY</span>
      <h1>Anime, Eurobeat &amp; Techno Song Studio</h1>
      <p>
        Build full uprising multi-section songs — opening synth brass riffs, Verse A → Verse B continuations,
        rising snare buildups, royal-road choruses, blazing guitar solos, and +2 semitone key-change finales.
      </p>
    </div>
    {#if composer.config}
      <div class="hero-actions">
        <button class="primary" disabled={composer.busy} onclick={() => void composer.generate()}>
          {composer.busy ? 'Composing…' : '⚡ Generate Full Song'}
        </button>
        <button disabled={composer.busy} onclick={freshAllMelodies}>
          🎲 New Melodies
        </button>
        <button class="ghost" onclick={() => void transport.toggle()}>
          {transport.playing ? '■ Stop' : '▶ Play Song'}
        </button>
        <button class="ghost" onclick={() => { workspace.view = 'studio'; }}>
          🎹 Open Piano Roll
        </button>
        <button class="ghost" onclick={() => void editor.exportWav()}>
          💾 Export WAV
        </button>
      </div>
    {/if}
  </header>

  {#if composer.config}
    {@const config = composer.config}
    <div class="section-block">
      <div class="block-head">
        <span class="eyebrow">1. SEMI-PRO GENRE BLUEPRINTS (CLICK TO LOAD &amp; COMPOSE)</span>
        <span class="muted">Includes Initial D Synth Brass, PolyBLEP Supersaw, Tube Rock Guitar, Cowbell &amp; Sparkle Arp</span>
      </div>
      <div class="bp-grid">
        {#each BLUEPRINTS as bp (bp.name)}
          <div class="bp-card" class:active={config.title === bp.title}>
            <div class="bp-top">
              <span class="badge">{bp.badge}</span>
              <span class="mono">{bp.bpm} BPM · {bp.key} {bp.scale.replaceAll('_', ' ')}</span>
            </div>
            <h3>{bp.name}</h3>
            <p>{bp.subtitle}</p>
            <div class="bp-btns">
              <button class="ghost" disabled={composer.busy} onclick={() => applyBlueprint(bp, false)}>Load Rack</button>
              <button class="primary" disabled={composer.busy} onclick={() => applyBlueprint(bp, true)}>⚡ Compose Now</button>
            </div>
          </div>
        {/each}
      </div>
    </div>

    <div class="section-block">
      <div class="block-head">
        <span class="eyebrow">2. MULTI-SECTION ARRANGEMENT &amp; MELODIC CONTINUATION BUILDER</span>
        <span class="mono">{config.custom_sections.length} sections · {totalBars || 48} bars · ~{estSeconds || 74}s</span>
      </div>
      <div class="timeline">
        {#each config.custom_sections as sec, i (`${i}-${sec.kind}`)}
          {@const meta = SECTION_META[sec.kind]}
          <button
            class="sec-pill"
            class:chosen={i === selectedIdx}
            style:--sec-color={meta.color}
            style:flex={sec.bars}
            onclick={() => { selectedIdx = i; }}
          >
            <span class="sec-kicker">{meta.short}</span>
            <strong>{sec.bars}b · T{sec.melody_theme}</strong>
            {#if sec.transpose_semitones !== 0}
              <span class="shift">{sec.transpose_semitones > 0 ? '+' : ''}{sec.transpose_semitones}st</span>
            {/if}
          </button>
        {:else}
          <div class="empty-tl">
            <span>Using default automatic multi-section structure.</span>
            <button class="ghost" onclick={() => { ensureEditableSections(); }}>Customize Sections</button>
          </div>
        {/each}
      </div>

      <div class="add-row">
        <span class="muted">Add continuation:</span>
        <button class="ghost" onclick={() => addSection('intro_riff')}>+ Intro Riff</button>
        <button class="ghost" onclick={() => addSection('verse_a')}>+ Verse A</button>
        <button class="ghost" onclick={() => addSection('verse_b')}>+ Verse B Lift</button>
        <button class="ghost" onclick={() => addSection('pre_chorus')}>+ Pre-Chorus Build</button>
        <button class="ghost" onclick={() => addSection('chorus')}>+ Chorus (Sabi)</button>
        <button class="ghost" onclick={() => addSection('solo')}>+ Guitar Solo</button>
        <button class="ghost" onclick={() => addSection('breakdown')}>+ Breakdown</button>
        <button class="ghost" onclick={() => addSection('final_chorus')}>+ Final Drop (+2st)</button>
      </div>

      {#if activeSection}
        <div class="sec-editor">
          <div class="sec-editor-head">
            <h3>Editing Section #{selectedIdx + 1}: {SECTION_META[activeSection.kind].label}</h3>
            <div class="sec-actions">
              <button class="ghost" disabled={selectedIdx === 0} onclick={() => moveSection(selectedIdx, -1)}>◀ Move</button>
              <button class="ghost" disabled={selectedIdx + 1 >= config.custom_sections.length} onclick={() => moveSection(selectedIdx, 1)}>Move ▶</button>
              <button disabled={composer.busy} onclick={() => rerollSectionMelody(selectedIdx)}>🎲 Reroll This Section Only</button>
              <button class="ghost" disabled={config.custom_sections.length <= 1} onclick={() => removeSection(selectedIdx)}>✕</button>
            </div>
          </div>
          <div class="grid-4">
            <label>Section Type
              <select bind:value={activeSection.kind}>
                {#each Object.entries(SECTION_META) as [val, m] (val)}
                  <option value={val}>{m.label}</option>
                {/each}
              </select>
            </label>
            <label>Bars
              <select bind:value={activeSection.bars}>
                {#each [2, 4, 8, 12, 16] as b (b)}<option value={b}>{b} bars</option>{/each}
              </select>
            </label>
            <label>Melody Theme Slot
              <select bind:value={activeSection.melody_theme}>
                {#each THEME_LABELS as label, idx (idx)}<option value={idx}>{label}</option>{/each}
              </select>
            </label>
            <label>Chord Progression
              <select bind:value={activeSection.progression}>
                {#each PROGRESSION_LABELS as p (p.id)}<option value={p.id}>{p.label}</option>{/each}
              </select>
            </label>
            <label>Energy Level: {activeSection.energy}/5
              <input type="range" min="1" max="5" step="1" bind:value={activeSection.energy} />
            </label>
            <label>Key Shift: {activeSection.transpose_semitones} st
              <input type="range" min="-12" max="12" step="1" bind:value={activeSection.transpose_semitones} />
            </label>
            <label>Section Seed Tweak
              <input type="number" min="0" max="4294967295" bind:value={activeSection.seed_offset} />
            </label>
          </div>
        </div>
      {/if}
    </div>

    <div class="two-col">
      <div class="section-block">
        <div class="block-head">
          <span class="eyebrow">3. GLOBAL TEMPO, KEY &amp; PRODUCTION</span>
        </div>
        <div class="grid-2">
          <label class="wide">Song Title<input bind:value={config.title} maxlength="160" /></label>
          <label>Genre Engine
            <select bind:value={config.genre_style}>
              <option value="initial_d_eurobeat">Initial D Eurobeat</option>
              <option value="anime_opening_techno">Fast Anime OP Techno</option>
              <option value="uprising_melodic_techno">Uprising Melodic Techno</option>
              <option value="hyper_eurodance_rave">Super Eurodance Rave</option>
              <option value="euphoric_trance">Euphoric 138 Trance</option>
              <option value="auto">Classic Auto</option>
            </select>
          </label>
          <label>Progression Default
            <select bind:value={config.progression_style}>
              {#each PROGRESSION_LABELS as p (p.id)}<option value={p.id}>{p.label}</option>{/each}
            </select>
          </label>
          <label>Tempo (BPM)<input type="number" min="40" max="240" bind:value={config.bpm} /></label>
          <label>Key<select bind:value={config.key}>{#each ['C','C#','D','Eb','E','F','F#','G','Ab','A','Bb','B'] as k (k)}<option>{k}</option>{/each}</select></label>
          <label>Scale<select bind:value={config.scale}>
            <option value="natural_minor">Natural Minor (Aeolian)</option>
            <option value="harmonic_minor">Harmonic Minor (Eurobeat Tension)</option>
            <option value="major">Major</option>
            <option value="dorian">Dorian</option>
            <option value="phrygian">Phrygian</option>
            <option value="melodic_minor">Melodic Minor</option>
          </select></label>
          <label>Seed<input type="number" min="0" max="4294967295" bind:value={config.seed} /></label>
        </div>
        <div class="checks">
          <label><input type="checkbox" bind:checked={config.key_change_finale} /> Uprising Key-Change Finale (+2 semitones on Final Chorus)</label>
          <label><input type="checkbox" bind:checked={config.riser_builds} /> Accelerating Pre-Chorus Snare Riser Builds</label>
          <label><input type="checkbox" bind:checked={config.studio_layers} /> 10-Track Studio Layers (Sparkle Arp + Cowbell + Crash Cymbal)</label>
        </div>
      </div>

      <div class="section-block">
        <div class="block-head">
          <span class="eyebrow">4. 10-TRACK STUDIO SOUND RACK ({editor.presets.length} PRESETS)</span>
        </div>
        <div class="grid-2">
          <label>Main Hook / Synth Brass<select bind:value={config.instruments.lead}>{#each editor.presets as p (p.id)}<option value={p.id}>{p.name}</option>{/each}</select></label>
          <label>Solo / Emotional Lead<select bind:value={config.instruments.emotional}>{#each editor.presets as p (p.id)}<option value={p.id}>{p.name}</option>{/each}</select></label>
          <label>Chords / M1 Piano<select bind:value={config.instruments.harmony}>{#each editor.presets as p (p.id)}<option value={p.id}>{p.name}</option>{/each}</select></label>
          <label>Driving Bassline<select bind:value={config.instruments.bass}>{#each editor.presets as p (p.id)}<option value={p.id}>{p.name}</option>{/each}</select></label>
          <label>Sparkle Arp Layer<select bind:value={config.instruments.arp}>{#each editor.presets as p (p.id)}<option value={p.id}>{p.name}</option>{/each}</select></label>
          <label>Offbeat Cowbell<select bind:value={config.instruments.cowbell}>{#each editor.presets as p (p.id)}<option value={p.id}>{p.name}</option>{/each}</select></label>
          <label>Club Kick<select bind:value={config.instruments.kick}>{#each editor.presets as p (p.id)}<option value={p.id}>{p.name}</option>{/each}</select></label>
          <label>Snare / Clap<select bind:value={config.instruments.snare}>{#each editor.presets as p (p.id)}<option value={p.id}>{p.name}</option>{/each}</select></label>
          <label>Hi-Hat<select bind:value={config.instruments.hat}>{#each editor.presets as p (p.id)}<option value={p.id}>{p.name}</option>{/each}</select></label>
          <label>Section Crash Cymbal<select bind:value={config.instruments.crash}>{#each editor.presets as p (p.id)}<option value={p.id}>{p.name}</option>{/each}</select></label>
        </div>
      </div>
    </div>

    {#if composer.error}
      <p class="err" role="alert">{composer.error}</p>
    {/if}
  {/if}
</section>

<style>
  .studio-page { max-width: 1280px; margin: 0 auto; padding: 28px 34px 48px; display: flex; flex-direction: column; gap: 20px; }
  .hero { display: flex; justify-content: space-between; align-items: flex-end; gap: 20px; flex-wrap: wrap; border-bottom: 1px solid var(--border); padding-bottom: 18px; }
  .hero h1 { font-size: 28px; letter-spacing: -0.04em; margin: 6px 0; }
  .hero p { max-width: 680px; color: var(--muted); font-size: 13px; line-height: 1.6; margin: 0; }
  .hero-actions { display: flex; gap: 8px; flex-wrap: wrap; }
  .section-block { background: var(--panel); border: 1px solid var(--border); border-radius: 10px; padding: 16px 18px; }
  .block-head { display: flex; justify-content: space-between; align-items: center; gap: 10px; flex-wrap: wrap; margin-bottom: 12px; }
  .bp-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 12px; }
  .bp-card { border: 1px solid var(--border); border-radius: 8px; padding: 12px; background: var(--bg); display: flex; flex-direction: column; gap: 6px; }
  .bp-card.active { border-color: var(--accent); background: color-mix(in srgb, var(--accent) 6%, var(--bg)); }
  .bp-top { display: flex; justify-content: space-between; align-items: center; font-size: 10px; }
  .badge { padding: 2px 6px; border-radius: 4px; background: color-mix(in srgb, var(--accent) 18%, transparent); color: var(--accent); font: 10px var(--font-mono); font-weight: 700; }
  .mono { font: 10px var(--font-mono); color: var(--muted); }
  .bp-card h3 { margin: 2px 0; font-size: 15px; }
  .bp-card p { margin: 0; font-size: 11px; color: var(--muted); line-height: 1.45; flex: 1; }
  .bp-btns { display: flex; gap: 6px; margin-top: 6px; }
  .bp-btns button { flex: 1; font-size: 11px; padding: 6px 8px; }
  .timeline { display: flex; gap: 6px; align-items: stretch; min-height: 62px; margin-bottom: 10px; overflow-x: auto; }
  .sec-pill { border: 1px solid var(--sec-color); border-radius: 8px; background: color-mix(in srgb, var(--sec-color) 14%, var(--bg)); color: var(--text); padding: 8px 10px; display: flex; flex-direction: column; align-items: flex-start; gap: 3px; min-width: 72px; position: relative; text-align: left; }
  .sec-pill.chosen { box-shadow: 0 0 0 2px var(--sec-color); background: color-mix(in srgb, var(--sec-color) 26%, var(--bg)); }
  .sec-kicker { font: 9px var(--font-mono); letter-spacing: 0.08em; color: var(--sec-color); font-weight: 700; }
  .sec-pill strong { font-size: 12px; }
  .shift { font: 10px var(--font-mono); color: var(--accent); }
  .empty-tl { display: flex; align-items: center; justify-content: space-between; width: 100%; padding: 12px; border: 1px dashed var(--border); border-radius: 8px; color: var(--muted); font-size: 12px; }
  .add-row { display: flex; gap: 6px; align-items: center; flex-wrap: wrap; margin-bottom: 12px; }
  .add-row button { font-size: 11px; padding: 4px 8px; }
  .sec-editor { border-top: 1px solid var(--border); padding-top: 12px; }
  .sec-editor-head { display: flex; justify-content: space-between; align-items: center; gap: 8px; flex-wrap: wrap; margin-bottom: 10px; }
  .sec-editor-head h3 { margin: 0; font-size: 14px; }
  .sec-actions { display: flex; gap: 6px; flex-wrap: wrap; }
  .grid-4 { display: grid; grid-template-columns: repeat(auto-fit, minmax(170px, 1fr)); gap: 10px; }
  .grid-2 { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
  .two-col { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
  label { display: flex; flex-direction: column; gap: 4px; font-size: 11px; color: var(--muted); }
  .wide { grid-column: 1 / -1; }
  .checks { display: grid; gap: 8px; margin-top: 14px; }
  .checks label { flex-direction: row; align-items: center; gap: 8px; color: var(--text); font-size: 12px; }
  .checks input { width: auto; }
  .err { color: var(--danger, #e77777); font-size: 12px; }
  @media (max-width: 960px) { .two-col { grid-template-columns: 1fr; } }
</style>
