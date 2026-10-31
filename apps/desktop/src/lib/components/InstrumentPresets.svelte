<script lang="ts">
  import { onMount } from 'svelte';
  import type { Instrument } from '../ipc/types';
  import { backend, errorMessage } from '../ipc/client';
  import { categoryLabel, groupPresets } from '../music/presets';
  import { editor } from '../stores/editor.svelte';

  let { instrument }: { instrument: Instrument } = $props();
  let busy = $state(false);
  let selected = $state('');
  let status = $state('');
  let error = $state('');
  let libraryPath = $state('');
  onMount(() => {
    void backend.presetLibraryInfo().then(info => {
      libraryPath = info.directory;
      if (info.warnings.length) error = info.warnings.join('\n');
    }).catch(reason => { error = errorMessage(reason); });
  });

  async function action(kind: 'reset' | 'save' | 'apply'): Promise<void> {
    if (busy || !editor.state) return;
    busy = true;
    error = ''; status = '';
    const id = instrument.id;
    const editorId = editor.state.editor_id;
    try {
      if (kind === 'save') {
        const saved = await backend.saveInstrumentPreset(id);
        if (editor.state?.editor_id === editorId) editor.instrumentCheckpoints.save({ ...saved.instrument, id });
        editor.presets = await backend.listPresets();
        status = `Saved ${saved.instrument.name}. Available under Saved.`;
      } else {
        const sound = kind === 'reset' ? editor.instrumentCheckpoints.get(id) : await backend.getPreset(selected);
        if (!sound) throw new Error('No reset point is available for this instrument.');
        if (editor.state?.editor_id !== editorId) throw new Error('The project changed. Select the instrument again.');
        await backend.edit({ ...sound, kind: 'update_instrument', id });
        await editor.refresh();
        if (kind === 'apply') editor.instrumentCheckpoints.save({ ...sound, id });
        status = kind === 'reset' ? 'Instrument reset. Undo restores your edits.' : 'Preset applied. Notes kept; Undo restores the previous sound.';
      }
    } catch (reason) { error = errorMessage(reason); }
    finally { busy = false; }
  }
</script>

<div class="preset-controls">
  <div class="buttons">
    <button class="ghost" disabled={busy} onclick={() => void action('reset')} title="Restore the starting or last saved instrument settings">Reset</button>
    <button class="ghost" disabled={busy} onclick={() => void action('save')} title={libraryPath}>Save preset</button>
  </div>
  <p>Reset returns to the opened, added, applied or last saved sound.</p>
  <label>Replace sound
    <select aria-label="Replace sound" bind:value={selected} disabled={busy}>
      <option value="">Choose a preset...</option>
      {#each groupPresets(editor.presets) as group (group.category)}
        <optgroup label={categoryLabel(group.category)}>
          {#each group.presets as preset (preset.id)}<option value={preset.id}>{preset.name}</option>{/each}
        </optgroup>
      {/each}
    </select>
  </label>
  <button disabled={busy || !selected} onclick={() => void action('apply')}>Apply preset</button>
  {#if status}<p role="status">{status}</p>{/if}
  {#if error}<p role="alert">{error}</p>{/if}
</div>

<style>
  .preset-controls { display: grid; gap: 6px; }
  .buttons { display: flex; gap: 8px; }
  label { display: grid; gap: 4px; font-size: 11px; color: var(--muted); }
  select { width: 100%; min-width: 0; }
  p { margin: 0; font-size: 10px; line-height: 1.4; color: var(--muted); overflow-wrap: anywhere; }
  [role='alert'] { color: var(--danger, #e77777); }
</style>
