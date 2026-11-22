<script lang="ts">
  import type { ScaleKind } from '../ipc/types';
  import { SCALE_KINDS, scaleLabel } from '../music/options';
  import { className } from '../music/pitch';
  import { formatTime } from '../music/time';
  import { editor } from '../stores/editor.svelte';
  import { transport } from '../stores/transport.svelte';
  import Icon from '../ui/Icon.svelte';

  const song = $derived(editor.song);
  const ROOTS = Array.from({ length: 12 }, (_, index) => index);

  /** Bar and beat under the play head (bars counted from 0, like the arrangement). */
  const position = $derived.by(() => {
    const bars = transport.position ?? 0;
    const bar = Math.floor(bars);
    const beat = Math.floor((bars - bar) * (song?.timing.beats_per_bar ?? 4)) + 1;
    return { bar, beat };
  });

  const elapsed = $derived(formatTime(transport.seconds ?? 0, true));
  const total = $derived(formatTime(editor.state?.length_seconds ?? 0));

  function setBpm(event: Event & { currentTarget: HTMLInputElement }): void {
    const bpm = Number(event.currentTarget.value);
    if (Number.isInteger(bpm) && bpm !== song?.timing.bpm) void editor.act({ kind: 'update_song', bpm });
  }

  function setRoot(event: Event & { currentTarget: HTMLSelectElement }): void {
    const scale_root = Number(event.currentTarget.value);
    if (Number.isInteger(scale_root) && scale_root !== song?.scale.root) {
      void editor.act({ kind: 'update_song', scale_root, transpose_notes: true });
    }
  }

  function setScale(event: Event & { currentTarget: HTMLSelectElement }): void {
    const scale_kind = event.currentTarget.value as ScaleKind;
    if (scale_kind && scale_kind !== song?.scale.kind) {
      void editor.act({ kind: 'update_song', scale_kind, transpose_notes: true });
    }
  }
</script>

<div class="transport">
  <button
    class="play"
    class:playing={transport.playing}
    onclick={() => void transport.toggle()}
    disabled={transport.busy || (!transport.playing && !editor.state?.length_bars)}
    title={transport.playing ? 'Stop (Space)' : !editor.state?.length_bars ? 'Generate or open a song first' : 'Play (Space)'}
    aria-label={transport.playing ? 'Stop' : 'Play'}
  >
    <Icon name={transport.playing ? 'stop' : 'play'} size={18} />
  </button>
  <button
    class="ghost icon"
    class:on={transport.looping}
    onclick={() => void transport.toggleLoop()}
    disabled={transport.busy}
    title={transport.looping ? 'Loop on' : 'Loop off'}
    aria-pressed={transport.looping}
  >
    <Icon name="loop" />
  </button>
  <button
    class="ghost follow-pill"
    class:on={editor.followPlayback}
    onclick={() => (editor.followPlayback = !editor.followPlayback)}
    title={editor.followPlayback ? 'Follow current note: ON (auto-tracks bar, pattern, and note)' : 'Follow current note: OFF'}
    aria-pressed={editor.followPlayback}
  >
    Follow
  </button>

  <div class="readout" title="Position">
    <span class="value">{String(position.bar).padStart(2, '0')}<span class="dim">.{position.beat}</span></span>
    <span class="unit">bar</span>
  </div>
  <div class="readout time" title="Elapsed / song length">
    <span class="value">{elapsed}</span>
    <span class="dim">/ {total}</span>
  </div>
  <label class="readout" title="Tempo (BPM)">
    <input class="value" type="number" min="20" max="400" value={song?.timing.bpm ?? 120} onchange={setBpm} aria-label="Tempo BPM" />
    <span class="unit">bpm</span>
  </label>
  {#if song}
    <div class="readout key-scale" title="Change song Key & Scale (transposes song notes)">
      <select class="pill-select root" value={String(song.scale.root)} onchange={setRoot} aria-label="Song Key">
        {#each ROOTS as root (root)}<option value={String(root)}>{className(root)}</option>{/each}
      </select>
      <select class="pill-select scale" value={song.scale.kind} onchange={setScale} aria-label="Song Scale">
        {#each SCALE_KINDS as kind (kind)}<option value={kind}>{scaleLabel(kind)}</option>{/each}
      </select>
    </div>
  {/if}
</div>

<style>
  .transport {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px 3px 3px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 999px;
  }

  .play {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    padding: 0;
    border: none;
    border-radius: 50%;
    background: var(--accent);
    color: var(--bg);
  }

  .play.playing {
    background: var(--accent-2);
  }

  .icon {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    padding: 0;
    color: var(--muted);
  }

  .icon.on {
    color: var(--accent);
  }

  .follow-pill {
    height: 26px;
    padding: 0 8px;
    font: 10px var(--font-mono, ui-monospace, monospace);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border-radius: 999px;
    color: var(--muted);
    border: 1px solid transparent;
  }

  .follow-pill.on {
    color: var(--accent);
    border-color: color-mix(in srgb, var(--accent) 45%, transparent);
    background: color-mix(in srgb, var(--accent) 12%, transparent);
  }

  .readout {
    display: flex;
    align-items: baseline;
    gap: 4px;
    padding: 0 6px;
    font-family: ui-monospace, monospace;
  }

  .value {
    font-size: 15px;
    color: var(--text);
  }

  input.value {
    width: 46px;
    padding: 0;
    border: none;
    background: transparent;
    font-family: inherit;
    text-align: right;
    appearance: textfield;
  }

  input.value::-webkit-inner-spin-button {
    appearance: none;
  }

  .time {
    min-width: 108px;
    justify-content: center;
    border-left: 1px solid var(--border);
    border-right: 1px solid var(--border);
  }

  .time .dim {
    font-size: 12px;
  }

  .dim,
  .unit {
    color: var(--muted);
  }

  .unit {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .key-scale {
    border-left: 1px solid var(--border);
    padding-left: 8px;
    gap: 4px;
  }

  .pill-select {
    padding: 2px 4px;
    border: 1px solid transparent;
    border-radius: 4px;
    background: transparent;
    color: var(--text);
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 12px;
    cursor: pointer;
  }

  .pill-select:hover,
  .pill-select:focus {
    border-color: var(--border);
    background: var(--panel);
  }

  .pill-select.root {
    width: 44px;
    font-weight: 600;
    color: var(--accent);
  }

  .pill-select.scale {
    max-width: 122px;
  }
</style>
