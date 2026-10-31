// Composition actions keep the UI and AI on one native implementation.
import { backend, errorMessage } from '../ipc/client';
import type { ComposeConfig, CompositionReport } from '../ipc/composition';
import { editor } from './editor.svelte';

const freshSeed = (): number => crypto.getRandomValues(new Uint32Array(1))[0] ?? 0;

class ComposerStore {
  config = $state<ComposeConfig | null>(null);
  report = $state<CompositionReport | null>(null);
  busy = $state(false);
  error = $state('');

  async init(): Promise<void> {
    if (this.config) return;
    try { this.config = await backend.compositionDefaults(); }
    catch (error) { this.error = errorMessage(error); }
  }

  neonFunk(): void {
    if(!this.config)return;
    Object.assign(this.config,{title:'Midnight Pixel Groove',vibe:'funk',key:'E',scale:'dorian',bpm:130,swing:8,neon_funk:true,expressive_slides:true,bass_hits:9,bass_rhythm:'template',seed:freshSeed(),instruments:{lead:'zzfx_neon_lead',harmony:'zzfx_neon_chords',bass:'zzfx_neon_bass',emotional:'zzfx_midnight_keys',kick:'zzfx_neon_kick',snare:'zzfx_neon_snare',hat:'zzfx_neon_hat'}});
    this.report=null;
  }

  async preview(): Promise<void> {
    if (!this.config || this.busy) return;
    this.busy = true;
    this.error = '';
    try { this.report = await backend.compositionPreview($state.snapshot(this.config)); }
    catch (error) { this.error = errorMessage(error); }
    finally { this.busy = false; }
  }

  async generate(): Promise<void> {
    if (!this.config || !editor.state || this.busy) return;
    this.busy = true;
    this.error = '';
    try {
      await this.apply($state.snapshot(this.config), editor.state.editor_id, editor.state.revision);
    } catch (error) { this.error = errorMessage(error); }
    finally { this.busy = false; }
  }

  async randomSong(): Promise<void> {
    if (!editor.state || this.busy) return;
    const { editor_id, revision } = editor.state;
    this.busy = true;
    this.error = '';
    try {
      this.config = await backend.compositionRandom(freshSeed());
      this.report = null;
      await this.apply($state.snapshot(this.config), editor_id, revision);
    } catch (error) { this.error = errorMessage(error); }
    finally { this.busy = false; }
  }

  async randomInstruments(): Promise<void> {
    if (!this.config || this.busy) return;
    this.busy = true;
    this.error = '';
    try {
      this.config = await backend.compositionRandom(freshSeed(), $state.snapshot(this.config));
      this.report = null;
      editor.notify('New instrument palette ready. Generate song applies it with the same musical seed.');
    } catch (error) { this.error = errorMessage(error); }
    finally { this.busy = false; }
  }

  private async apply(config: ComposeConfig, editorId: number, revision: number): Promise<void> {
    const result = await backend.composeSong(config, editorId, revision);
    this.report = result.report;
    await editor.refresh();
    editor.checkpointInstruments();
    editor.notify(`Generated ${result.report.bars.length} editable bars. Undo restores the previous song.`);
  }

  async develop(degrees: number, invert: boolean): Promise<void> {
    if (editor.selectedPattern === null) return;
    try {
      await backend.developPattern(editor.selectedPattern, degrees, invert);
      await editor.refresh();
    } catch (error) { editor.notify(errorMessage(error), true); }
  }
}
export const composer = new ComposerStore();
