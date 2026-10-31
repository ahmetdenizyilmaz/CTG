<script lang="ts">
  import { editor } from '../stores/editor.svelte';
  import { workspace } from '../stores/workspace.svelte';
  import demos from '../music/demos.json';
  import { transport } from '../stores/transport.svelte';
  let search = $state('');
  let category = $state('All sessions');
  let creator = $state('All creators');
  const creators = [...new Set(demos.map(d => d.author))];
  const visible = $derived(demos.filter(d => (category === 'All sessions' || d.category === category) && (creator === 'All creators' || d.author === creator) && `${d.title} ${d.author} ${d.detail}`.toLowerCase().includes(search.toLowerCase())));
  function preview(event: Event): void {
    const current = event.currentTarget;
    document.querySelectorAll('audio').forEach(audio => { if(audio !== current) audio.pause(); });
    if(transport.playing) void transport.stop();
  }
  let busy = $state(false);
  async function load(id:string):Promise<void> {
    busy=true;
    if(await editor.openDemo(id)) workspace.view='studio';
    busy=false;
  }
</script>
<section class="demo-page">
  <span class="eyebrow">THE LISTENING ROOM</span><h1>Small chips. Big ideas.</h1>
  <p class="intro">Your sessions, together. Listen to the original mixes or open a native project and explore every note.</p>
  <div class="filters"><input type="search" aria-label="Search demos" placeholder="Search songs, creators, moods..." bind:value={search}/><select aria-label="Demo collection" bind:value={category}>{#each ['All sessions','Featured','Earlier sessions','ZzFX archive'] as group (group)}<option>{group}</option>{/each}</select><select aria-label="Demo creator" bind:value={creator}><option>All creators</option>{#each creators as author (author)}<option>{author}</option>{/each}</select><span>{visible.length} songs</span></div>
  <div class="demo-grid">{#each visible as demo,index (demo.id)}
    <article style:--demo-color={demo.color} class:featured={demo.id === 'afterglow-circuit'}>
      <div class="cover" aria-hidden="true"><span class="number">{String(index+1).padStart(2,'0')}</span><div class="wave">{#each [20,38,65,42,86,56,100,72,42,68,90,50,30,58,38,18] as height,i (i)}<i style:height={`${height}%`}></i>{/each}</div><span class="cover-title">{demo.title}</span></div>
      <div class="demo-body"><span class="eyebrow">{demo.author} / {demo.project ? 'EDITABLE PROJECT' : 'AUDIO ONLY'}</span><h2>{demo.title}</h2><p>{demo.detail}</p><div class="metadata">{#if demo.bpm}<span>{demo.bpm} BPM</span>{/if}<span>{demo.time}</span>{#if demo.tracks}<span>{demo.tracks} tracks</span>{/if}</div><audio controls preload="none" src={`/demos/${demo.audio}`} aria-label={`Listen to ${demo.title}`} onplay={preview}></audio><div class="demo-bottom"><small>{demo.category}</small>{#if demo.project}<button class="primary" disabled={busy} onclick={() => void load(demo.id)}>Open project</button>{:else}<span class="archive-note">ZzFX audio reference</span>{/if}</div></div>
    </article>
  {:else}<p class="muted">No songs match these filters.</p>{/each}</div>
  <p class="footnote">Native projects open as editable copies. Original mastered previews may have different finishing or tail lengths. ZzFX archive entries are listening references, not editable CTG songs.</p>
</section>
<style>
  .filters{display:flex;gap:10px;align-items:center;margin-bottom:24px;flex-wrap:wrap}.filters input{flex:1;min-width:200px;padding:10px 12px}.filters select{padding:10px 24px 10px 10px;font-size:11px}.filters>span{font:10px var(--font-mono);color:var(--muted)}audio{width:100%;height:34px;margin:0 0 16px}.archive-note{font-size:10px;color:var(--muted)}

  .demo-page{max-width:1220px;margin:auto;padding:44px 48px}.demo-page>h1{font-size:42px;letter-spacing:-.055em;font-weight:600;margin:10px 0 12px}.intro{color:var(--muted);font-size:15px;line-height:1.8;max-width:570px;margin-bottom:34px}.demo-grid{display:grid;grid-template-columns:1fr 1fr;gap:20px}article{display:grid;grid-template-columns:150px 1fr;border:1px solid var(--border);border-radius:12px;overflow:hidden;background:var(--panel)}.featured{grid-column:1/-1;grid-template-columns:260px 1fr}.cover{min-height:220px;position:relative;padding:20px;background:linear-gradient(140deg,color-mix(in srgb,var(--demo-color) 20%,var(--bg)),var(--bg));color:var(--demo-color);display:flex;flex-direction:column;justify-content:space-between}.number{font:11px var(--font-mono);opacity:.65}.wave{display:flex;align-items:center;gap:4px;height:90px}.wave i{flex:1;background:var(--demo-color);border-radius:2px;opacity:.7}.cover-title{font:12px var(--font-mono);line-height:1.5}.demo-body{padding:24px;min-width:0}.demo-body h2{text-transform:none;letter-spacing:-.03em;color:var(--text);font-size:23px;margin:10px 0}.demo-body p{color:var(--muted);line-height:1.8;font-size:12px}.metadata{display:flex;gap:20px;font:11px var(--font-mono);margin:18px 0;color:var(--demo-color)}.demo-bottom{display:flex;gap:12px;align-items:center;justify-content:space-between;flex-wrap:wrap}.demo-bottom small{font-size:10px;line-height:1.7}.footnote{font-size:11px;color:var(--muted);margin-top:24px}.featured .demo-body{padding:30px}.featured h2{font-size:30px}@media(max-width:1200px){.demo-grid{grid-template-columns:1fr}.featured{grid-column:auto}.demo-page{padding:30px}.featured,article{grid-template-columns:200px 1fr}}
</style>
