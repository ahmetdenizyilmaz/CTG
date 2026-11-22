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
  const roles = ['lead', 'harmony', 'bass', 'emotional', 'kick', 'snare', 'hat'] as const;

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
    composer.config.bpm = kit.bpm;
    composer.config.swing = kit.swing;
    composer.config.chord_colors = true;
    composer.config.voice_leading = true;
    composer.config.cadences = true;
    composer.config.instruments = { ...kit.instruments };
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
      <label>Form<select bind:value={config.form}><option value="short">16 bars</option><option value="standard">32 bars</option><option value="extended">48 bars</option><option value="long">64 bars</option><option value="epic">96 bars</option><option value="marathon">128 bars</option></select></label>
      <label>Key<select bind:value={config.key}>{#each ['C','C#','D','Eb','E','F','F#','G','Ab','A','Bb','B'] as key (key)}<option>{key}</option>{/each}</select></label>
      <label>Scale<select bind:value={config.scale}>
        <option value="major">Major</option><option value="natural_minor">Natural minor</option><option value="harmonic_minor">Harmonic minor</option>
        <option value="dorian">Dorian</option><option value="mixolydian">Mixolydian</option><option value="major_pentatonic">Major pentatonic</option><option value="minor_pentatonic">Minor pentatonic</option>
      </select></label>
      <label>Tempo<input type="number" min="40" max="240" bind:value={config.bpm} /></label>
      <label>Swing %<input type="number" min="0" max="60" bind:value={config.swing} /></label>
      <label>Bass rhythm<select bind:value={config.bass_rhythm}><option value="mixed">Mixed</option><option value="template">Groove templates</option><option value="euclidean">Euclidean</option></select></label>
      <label>Bass hits / bar<input type="number" min="0" max="16" bind:value={config.bass_hits} /></label>
      <label>Seed<input type="number" min="0" max="4294967295" step="1" bind:value={config.seed} /></label>
      <button class="ghost" onclick={() => { config.seed = Math.floor(Math.random() * 4294967296); }}>New seed</button>
    </div>
    <details open><summary>Musical ideas</summary>
      <div class="features">{#each features as feature (feature.key)}
        <label title={feature.help}><input type="checkbox" bind:checked={config[feature.key]} />{feature.name}</label>
      {/each}</div>
    </details>
    <details><summary>Instruments · {editor.presets.length} available sounds</summary>
      <div class="fields">{#each roles as role (role)}
        <label>{role === 'emotional' ? 'Emotional theme' : role}<select bind:value={config.instruments[role]}>
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
        <p>{composer.report.design.groove.replaceAll('_', ' ')} groove · {composer.report.design.accompaniment.replaceAll('_', ' ')} · {composer.report.design.phrase_bars}-bar hook</p>
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
