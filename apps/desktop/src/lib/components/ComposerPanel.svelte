<script lang="ts">
  import { onMount } from 'svelte';
  import { composer } from '../stores/composer.svelte';
  import { editor } from '../stores/editor.svelte';
  import type { ComposeConfig } from '../ipc/composition';

  onMount(() => { void composer.init(); });
  const features: { key: keyof Pick<ComposeConfig, 'motif_development' | 'voice_leading' | 'cadences' | 'chord_colors' | 'modal_mixture' | 'secondary_dominants' | 'suspensions' | 'neon_funk' | 'expressive_slides'>; name: string; help: string }[] = [
    {key:'neon_funk',name:'Neon funk groove',help:'Syncopated octave bass and offbeat chord stabs.'},
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
</script>

<div class="composer">
  <h2>Compose with CTG</h2>
  <p class="muted">Shape a recurring hook, choose your sounds, then edit every note.</p>
  {#if composer.config}
    {@const config = composer.config}
    <button class="primary" disabled={composer.busy} onclick={()=>composer.neonFunk()}>Neon Pixel Funk recipe</button>
    <p class="muted">Load E Dorian, the original ZzFX sounds, octave bass and expressive slides, then Generate song for a new composition.</p>
    <div class="actions discovery">
      <button disabled={composer.busy} onclick={() => void composer.randomSong()}>Random song</button>
      <button class="ghost" disabled={composer.busy} onclick={() => void composer.randomInstruments()}>Random instruments</button>
    </div>
    <p class="muted">Random song creates a new arrangement and sounds with a recurring hook, connected chords and a resolving ending. Undo restores your song.</p>
    <p class="muted">Random instruments chooses suitable sounds for the next generation. Open Instruments to see them, then Generate song to apply.</p>
    <div class="fields">
      <label class="wide">Song title<input bind:value={config.title} maxlength="160" /></label>
      <label>Character<select bind:value={config.vibe}>
        <option value="funk">Funk / emotional</option><option value="adventure">Adventure</option>
        <option value="battle">Battle</option><option value="dungeon">Atmospheric</option>
        <option value="title_screen">Title theme</option><option value="boss">Boss</option>
      </select></label>
      <label>Form<select bind:value={config.form}><option value="short">16 bars</option><option value="standard">32 bars</option><option value="extended">48 bars</option></select></label>
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
      <button class="ghost" onclick={() => { config.seed = Math.floor(Math.random() * 4294967296); }}>New variation</button>
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
      <button disabled={composer.busy} onclick={() => void composer.generate()}>{composer.busy ? 'Composing…' : 'Generate song'}</button>
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
  .discovery { justify-content: flex-start; margin-top: 12px; }
  .analysis { max-height: 200px; overflow: auto; font-size: 12px; }
  .analysis div { display: flex; justify-content: space-between; padding: 4px; }
  [role='alert'] { color: var(--danger, #e77777); }
</style>
