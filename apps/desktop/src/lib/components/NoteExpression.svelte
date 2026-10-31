<script lang="ts">
  import type { Note, ZzfxEffectCode } from '../ipc/types';
  import { editor } from '../stores/editor.svelte';
  let { note, patternId, zzfx }: {note:Note;patternId:number;zzfx:boolean}=$props();
  const effects: [ZzfxEffectCode,string][]=[['SU','Slide up'],['SD','Slide down'],['VB','Studio wobble'],['DT','Duty'],['ST','Staccato'],['PD','Pitch drop'],['BC','Bit crush'],['TR','Tremolo']];
  // Serialize changes and merge against the acknowledged state. Rapidly changing
  // two controls must not replace the earlier change with a stale whole note.
  let pending: Promise<void> = Promise.resolve();
  function update(changes:Partial<Note>):void {
    const id=patternId,start=note.start,pitch=note.pitch,session=editor.state?.editor_id;
    pending=pending.then(async()=>{
      if(editor.state?.editor_id!==session)return;
      const pattern=editor.song?.patterns.find(p=>p.id===id);if(!pattern)return;
      const notes=pattern.notes.map(n=>n.start===start&&n.pitch===pitch?{...n,...changes}:n);
      await editor.act({kind:'set_notes',pattern:id,notes});
    });
  }
</script>
<div class="expression" aria-label="Note expression">
  <span>Step {note.start} · MIDI {note.pitch}</span>
  <label>Velocity<input aria-label="Note velocity" type="number" min="1" max="127" value={note.velocity} onchange={e=>e.currentTarget.checkValidity()&&update({velocity:Number(e.currentTarget.value)})}/></label>
  <label>Slide (semitones)<input aria-label="Note slide" type="number" min="-48" max="48" value={note.slide_semitones??0} onchange={e=>e.currentTarget.checkValidity()&&update({slide_semitones:Number(e.currentTarget.value)})}/></label>
  <label><input type="checkbox" checked={note.glissando??false} onchange={e=>update({glissando:e.currentTarget.checked})}/>Stepped glissando</label>
  <label>Hold after block<input aria-label="Note tail steps" type="number" min="0" max="65535" value={note.tail_steps??0} onchange={e=>e.currentTarget.checkValidity()&&update({tail_steps:Number(e.currentTarget.value)})}/></label>
  {#if zzfx}
    <label>Tracker effect<select aria-label="ZzFX note effect" value={note.zzfx_effect?.code??''} onchange={e=>update({zzfx_effect:e.currentTarget.value?{code:e.currentTarget.value as ZzfxEffectCode,value:note.zzfx_effect?.value??96}:null})}><option value="">None</option>{#each effects as [code,label] (code)}<option value={code}>{label}</option>{/each}</select></label>
    {#if note.zzfx_effect}<label>Amount<input aria-label="ZzFX effect amount" type="number" min="0" max="255" value={note.zzfx_effect.value} onchange={e=>e.currentTarget.checkValidity()&&note.zzfx_effect&&update({zzfx_effect:{...note.zzfx_effect,value:Number(e.currentTarget.value)}})}/></label>{/if}
  {/if}
  <button class="ghost" onclick={()=>{const p=editor.pattern;if(p)void editor.act({kind:'set_notes',pattern:p.id,notes:p.notes.filter(n=>n.start!==note.start||n.pitch!==note.pitch)});}}>Delete note</button>
</div>
<style>.expression{display:flex;gap:12px;align-items:center;flex-wrap:wrap;padding:8px 12px;background:var(--panel-raised);border-radius:6px;font-size:11px}.expression label{display:flex;gap:6px;align-items:center}.expression input[type=number]{width:64px}.expression input[type=checkbox]{min-height:0}</style>
