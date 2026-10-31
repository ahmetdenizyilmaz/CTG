<script lang="ts">
  let { parameters, onchange }: { parameters: number[]; onchange: (parameters: number[]) => void } = $props();
  const fields: [string,number,number,number][] = [
    ['Gain',0,4,.01],['Pitch randomness',0,1,.001],['C4 frequency (Hz)',0,20000,.01],
    ['Attack (s)',0,10,.001],['Sustain (s)',0,10,.001],['Release (s)',0,10,.001],
    ['Wave shape',0,5,1],['Curve / pulse width',0,10,.01],['Pitch slide',-1000,1000,.01],
    ['Slide acceleration',-1000,1000,.01],['Pitch jump (Hz)',-20000,20000,.1],
    ['Jump time (s)',0,10,.001],['Repeat period (s)',0,10,.001],['Noise',0,50,.01],
    ['Modulation',-20000,20000,.1],['Bit crush',0,10,.01],['Delay (s)',0,10,.001],
    ['Sustain level',0,2,.01],['Decay (s)',0,10,.001],['Tremolo depth',0,1,.01],['Filter (+high / -low Hz)',-20000,20000,1],
  ];
  function change(index:number,event:Event):void {
    const input=event.currentTarget as HTMLInputElement;
    if(!input.checkValidity()) return;
    const next=[...parameters]; next[index]=Number(input.value); onchange(next);
  }
</script>
<details open><summary>ZzFX sound</summary>
  <p>0 sine · 1 triangle · 2 saw · 3 tangent · 4 noise · 5 pulse. The sound has its own envelope. Use note effects for individual bends.</p>
  <div class="parameters">{#each fields as [label,min,max,step],i (label)}
    <label>{label}<input type="number" {min} {max} {step} value={parameters[i] ?? 0} onchange={(event)=>change(i,event)} /></label>
  {/each}</div>
</details>
<style>
  .parameters{display:grid;grid-template-columns:1fr 1fr;gap:8px;padding:8px 0}label{font-size:10px;color:var(--muted);display:grid;gap:4px}input{width:100%;min-width:0}p{font-size:11px;line-height:1.6;color:var(--muted)}summary{cursor:pointer}
</style>
