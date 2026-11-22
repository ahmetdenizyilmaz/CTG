<script lang="ts">
  import type { Action, ChipProfile, ScaleKind, Song } from '../ipc/types';
  import { CHIP_PROFILES, SCALE_KINDS, scaleLabel } from '../music/options';
  import { className } from '../music/pitch';
  import { editor } from '../stores/editor.svelte';

  let { song }: { song: Song } = $props();

  type Change = Omit<Extract<Action, { kind: 'update_song' }>, 'kind'>;
  type Field = Event & { currentTarget: HTMLInputElement | HTMLSelectElement };

  const ROOTS = Array.from({ length: 12 }, (_, index) => index);
  let transposeNotes = $state(true);

  const update = (change: Change): void => void editor.act({ kind: 'update_song', ...change });
  const number = (event: Field): number => Number(event.currentTarget.value);
</script>

<div class="settings">
  <h3>Tempo</h3>
  <div class="pair">
    <label>BPM
      <input type="number" min="20" max="400" value={song.timing.bpm}
        onchange={(e: Field) => update({ bpm: number(e) })} aria-label="Tempo BPM" />
    </label>
    <label class="swing">Speed <span>{song.timing.bpm} BPM</span>
      <input type="range" min="40" max="240" value={song.timing.bpm}
        onchange={(e: Field) => update({ bpm: number(e) })} aria-label="Tempo slider" />
    </label>
  </div>

  <h3>Key &amp; Scale</h3>
  <div class="pair">
    <select value={String(song.scale.root)} onchange={(e: Field) => update({ scale_root: number(e), transpose_notes: transposeNotes })} aria-label="Root">
      {#each ROOTS as root (root)}<option value={String(root)}>{className(root)}</option>{/each}
    </select>
    <select
      value={song.scale.kind}
      onchange={(e: Field) => update({ scale_kind: e.currentTarget.value as ScaleKind, transpose_notes: transposeNotes })}
      aria-label="Scale"
    >
      {#each SCALE_KINDS as kind (kind)}<option value={kind}>{scaleLabel(kind)}</option>{/each}
    </select>
  </div>
  <label class="check">
    <input type="checkbox" bind:checked={transposeNotes} />
    Transpose &amp; remap song notes to Key / Scale
  </label>
  <div class="transpose-row">
    <button class="ghost" onclick={() => update({ semitone_offset: -12, transpose_notes: true })} title="Transpose down 1 octave">-12</button>
    <button class="ghost" onclick={() => update({ semitone_offset: -1, transpose_notes: true })} title="Transpose down 1 semitone">-1 st</button>
    <button class="ghost" onclick={() => update({ semitone_offset: 1, transpose_notes: true })} title="Transpose up 1 semitone">+1 st</button>
    <button class="ghost" onclick={() => update({ semitone_offset: 12, transpose_notes: true })} title="Transpose up 1 octave">+12</button>
  </div>

  <h3>Hardware</h3>
  <select
    value={song.chip_profile}
    onchange={(e: Field) => update({ chip_profile: e.currentTarget.value as ChipProfile })}
    aria-label="Chip profile"
  >
    {#each CHIP_PROFILES as profile (profile.value)}<option value={profile.value}>{profile.label}</option>{/each}
  </select>
  <p class="hint">Limits tracks, voices and notes at once to what the chip could play.</p>

  <h3>Grid</h3>
  <div class="pair">
    <label>Beats per bar
      <input type="number" min="1" max="16" value={song.timing.beats_per_bar}
        onchange={(e: Field) => update({ beats_per_bar: number(e) })} />
    </label>
    <label>Steps per beat
      <input type="number" min="1" max="16" value={song.timing.steps_per_beat}
        onchange={(e: Field) => update({ steps_per_beat: number(e) })} />
    </label>
  </div>
  <label class="swing">Swing <span>{song.timing.swing}%</span>
    <input type="range" min="0" max="100" value={song.timing.swing} onchange={(e: Field) => update({ swing: number(e) })} />
  </label>

  <h3>Reverb</h3>
  <div class="pair">
    <label class="swing">Room size <span>{song.reverb.size}%</span>
      <input type="range" min="0" max="100" value={song.reverb.size} onchange={(e: Field) => update({ reverb_size: number(e) })} />
    </label>
    <label class="swing">Damping <span>{song.reverb.damping}%</span>
      <input type="range" min="0" max="100" value={song.reverb.damping}
        onchange={(e: Field) => update({ reverb_damping: number(e) })} />
    </label>
  </div>
  <p class="hint">Instruments join the room through their reverb send.</p>
</div>

<style>
  .settings {
    display: flex;
    flex-direction: column;
    gap: 6px;
    width: 280px;
  }

  h3 {
    margin: 6px 0 0;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
  }

  h3:first-child {
    margin-top: 0;
  }

  .pair {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 11px;
    color: var(--muted);
  }

  .swing span {
    color: var(--text);
  }

  .hint {
    margin: 0;
    font-size: 11px;
    color: var(--muted);
  }

  .check {
    flex-direction: row;
    align-items: center;
    gap: 6px;
    color: var(--text);
    cursor: pointer;
  }

  .check input {
    width: auto;
  }

  .transpose-row {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 4px;
  }

  .transpose-row button {
    font-size: 11px;
    padding: 3px 6px;
  }
</style>
