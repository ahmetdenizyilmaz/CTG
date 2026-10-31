<script lang="ts">
  import { onMount } from 'svelte';
  import ArrangementView from './lib/components/ArrangementView.svelte';
  import InstrumentPanel from './lib/components/InstrumentPanel.svelte';
  import InstrumentEditor from './lib/components/InstrumentEditor.svelte';
  import PatternEditor from './lib/components/PatternEditor.svelte';
  import PatternPanel from './lib/components/PatternPanel.svelte';
  import StatusBar from './lib/components/StatusBar.svelte';
  import TopBar from './lib/components/TopBar.svelte';
  import ComposerPanel from './lib/components/ComposerPanel.svelte';
  import DemoLibrary from './lib/components/DemoLibrary.svelte';
  import AgentConnect from './lib/components/AgentConnect.svelte';
  import Icon from './lib/ui/Icon.svelte';
  import { editor } from './lib/stores/editor.svelte';
  import { transport } from './lib/stores/transport.svelte';
  import { workspace } from './lib/stores/workspace.svelte';
  import { theme } from './lib/theme/theme.svelte';
  onMount(() => { theme.init(); void editor.init(); });
  function onKeydown(event: KeyboardEvent): void {
    const target = event.target;
    if (target instanceof HTMLElement && (target.isContentEditable || ['INPUT','SELECT','TEXTAREA'].includes(target.tagName))) return;
    const command = event.ctrlKey || event.metaKey;
    const key = event.key.toLowerCase();
    if (event.key === ' ') { event.preventDefault(); void transport.toggle(); }
    else if (command && key === 'z') { event.preventDefault(); void (event.shiftKey ? editor.redo() : editor.undo()); }
    else if (command && key === 'y') { event.preventDefault(); void editor.redo(); }
    else if (command && key === 's') { event.preventDefault(); void editor.save(event.shiftKey); }
    else if (command && key === 'o') { event.preventDefault(); void editor.open(); }
    else if (command && key === 'n') { event.preventDefault(); void editor.newSong(); }
    else if (command && key === 'e') { event.preventDefault(); void editor.exportWav(); }
  }
</script>
<svelte:window onkeydown={onKeydown} />
<div class="app">
  <TopBar />
  <main class="workstation" class:hide-library={!workspace.library} class:hide-inspector={!workspace.inspector}>
    <nav class="rail" aria-label="Workspace">
      <button class:chosen={workspace.view === 'studio'} aria-label="Studio" onclick={() => workspace.view = 'studio'}><Icon name="sliders" size={20}/><span>Studio</span></button>
      <button class:chosen={workspace.view === 'compose'} aria-label="Compose" onclick={() => { workspace.view = 'compose'; workspace.inspector = true; }}><Icon name="plus" size={20}/><span>Compose</span></button>
      <button class:chosen={workspace.view === 'demos'} aria-label="Demo library" onclick={() => workspace.view = 'demos'}><Icon name="play" size={18}/><span>Demos</span></button>
      <button class:chosen={workspace.view === 'connect'} aria-label="Connect AI" onclick={() => workspace.view = 'connect'}><Icon name="loop" size={20}/><span>AI / MCP</span></button>
      <div class="rail-end"><span class="vertical-label">CHIPTUNE GENERATOR</span><span>0.2</span></div>
    </nav>
    {#if workspace.view === 'demos'}<div class="full-stage"><DemoLibrary /></div>
    {:else if workspace.view === 'connect'}<div class="full-stage"><AgentConnect /></div>
    {:else if editor.song}
      <aside class="library" aria-label="Sound library">
        <div class="dock-heading"><span>PROJECT LIBRARY</span><span class="tag">LOCAL</span></div>
        <InstrumentPanel /><PatternPanel />
        <div class="library-foot">{editor.presets.length} available sounds <span>+ your presets</span></div>
      </aside>
      <section class="stage" aria-label="Music workspace">
        <div class="stage-heading"><div><span class="eyebrow">SESSION / ARRANGEMENT</span><h1>{editor.song.meta.title}</h1></div>
          <div class="stage-info"><span>{editor.state?.length_bars ?? 0} bars · {editor.song.tracks.length} tracks</span><button class="ghost" aria-label="Toggle sound library" aria-pressed={workspace.library} onclick={() => workspace.library = !workspace.library}>Library</button><button class="ghost" aria-label="Toggle inspector" aria-pressed={workspace.inspector} onclick={() => workspace.inspector = !workspace.inspector}>Inspector</button></div>
        </div>
        {#if editor.song.tracks.length}<div class="editors"><ArrangementView /><PatternEditor /></div>
        {:else}<div class="empty-session"><span class="empty-symbol">♪</span><span class="eyebrow">A NEW FREQUENCY</span><h2>Your next track starts here.</h2><p>Write a melody, shape a sound, or let an AI agent join your session.</p><div class="row"><button class="primary" onclick={() => workspace.view = 'demos'}>Explore demo songs</button><button onclick={() => { workspace.view = 'compose'; workspace.inspector = true; }}>Compose a song</button></div><button class="ghost" onclick={() => void editor.act({kind:'add_track',name:'Track 1'})}>Start with an empty track</button></div>{/if}
        <div class="stage-foot"><span>SPACE <small>play / pause</small></span><span>CLICK <small>edit a note</small></span><span>CTRL Z <small>undo</small></span><span class="local-tag">● Offline synthesis</span></div>
      </section>
      <aside class="inspector" aria-label="Inspector">
        <div class="dock-heading"><span>{workspace.view === 'compose' ? 'COMPOSITION LAB' : 'SOUND INSPECTOR'}</span><span class="tag">{workspace.view === 'compose' ? 'THEORY' : 'SYNTH'}</span></div>
        <div class="inspector-content">{#if workspace.view === 'compose'}<ComposerPanel />{:else if editor.instrument}<div class="instrument-heading"><span class="eyebrow">SELECTED INSTRUMENT</span><h2>{editor.instrument.name}</h2><span class="tag">{editor.instrument.voice.type.toUpperCase()}</span></div><InstrumentEditor instrument={editor.instrument} />{:else}<p class="muted">Select or add a sound to shape its voice, envelope and effects.</p>{/if}</div>
      </aside>
    {:else}<div class="full-stage">Opening CTG…</div>{/if}
  </main>
  <StatusBar />
</div>
<style>
  .app{display:grid;grid-template-rows:54px minmax(0,1fr) 28px;height:100%}
  .workstation{display:grid;grid-template-columns:62px 218px minmax(0,1fr) 302px;min-height:0}
  .rail{display:flex;flex-direction:column;align-items:center;gap:10px;padding:16px 5px;background:var(--panel);border-right:1px solid var(--border)}
  .rail button{display:flex;flex-direction:column;align-items:center;gap:7px;width:52px;padding:12px 2px;border:0;background:transparent;color:var(--muted);border-radius:8px}.rail button span{font-size:9px}.rail button.chosen{background:color-mix(in srgb,var(--accent) 13%,transparent);color:var(--accent);box-shadow:inset 2px 0 var(--accent)}
  .rail-end{margin-top:auto;display:flex;flex-direction:column;align-items:center;gap:18px;color:var(--muted);font:10px var(--font-mono)}.vertical-label{writing-mode:vertical-rl;letter-spacing:.18em;font-size:9px}
  .library,.inspector{min-height:0;min-width:0;background:var(--panel);display:flex;flex-direction:column;border-right:1px solid var(--border);overflow:hidden}.inspector{border-right:0;border-left:1px solid var(--border)}.inspector-content{overflow-y:auto;padding:16px}
  .dock-heading{height:43px;display:flex;align-items:center;justify-content:space-between;padding:0 14px;border-bottom:1px solid var(--border);font:10px var(--font-mono);letter-spacing:.06em;flex:none;color:var(--muted)}.library-foot{margin-top:auto;padding:12px;font-size:10px;border-top:1px solid var(--border);color:var(--muted)}.library-foot span{display:block;margin-top:4px;opacity:.6}
  .stage{display:flex;flex-direction:column;min-width:0;min-height:0;background:var(--bg)}.stage-heading{display:flex;align-items:center;justify-content:space-between;padding:18px 20px;gap:12px}h1{font-size:18px;margin:6px 0 0;font-weight:600;letter-spacing:-.03em}.stage-info{display:flex;gap:8px;align-items:center;font:10px var(--font-mono);color:var(--muted);flex-wrap:wrap;justify-content:flex-end}.stage-info button{font-size:10px;padding:4px}
  .editors{flex:1;display:grid;grid-template-rows:minmax(180px,53%) minmax(180px,1fr);gap:12px;padding:0 16px;min-height:0}.stage-foot{display:flex;gap:16px;padding:11px 16px;font:9px var(--font-mono);color:var(--muted)}.stage-foot small{margin-left:4px;font-family:var(--font-ui)}.local-tag{margin-left:auto;color:var(--accent)}.instrument-heading h2{text-transform:none;font-size:17px;line-height:1.4;color:var(--text);letter-spacing:-.02em;margin:8px 0 12px}.full-stage{grid-column:2/-1;overflow-y:auto;min-width:0}
  .empty-session{flex:1;display:flex;align-items:center;justify-content:center;flex-direction:column;text-align:center;padding:36px;background-image:radial-gradient(var(--border) 1px,transparent 1px);background-size:20px 20px}.empty-symbol{font-size:56px;color:var(--accent);margin-bottom:20px}.empty-session h2{color:var(--text);font-size:28px;text-transform:none;letter-spacing:-.04em;margin:14px 0 0}.empty-session p{max-width:340px;line-height:1.8;color:var(--muted);margin-bottom:24px}.empty-session>button{margin-top:14px;font-size:11px}
  .hide-library{grid-template-columns:62px 0 minmax(0,1fr) 302px}.hide-library .library{visibility:hidden}.hide-inspector{grid-template-columns:62px 218px minmax(0,1fr) 0}.hide-inspector .inspector{visibility:hidden}.hide-library.hide-inspector{grid-template-columns:62px 0 minmax(0,1fr) 0}
  @media(max-width:1200px){.workstation{grid-template-columns:56px 180px minmax(0,1fr) 270px}.hide-library{grid-template-columns:56px 0 minmax(0,1fr) 270px}.hide-inspector{grid-template-columns:56px 180px minmax(0,1fr) 0}.hide-library.hide-inspector{grid-template-columns:56px 0 minmax(0,1fr) 0}.stage-info>span,.local-tag{display:none}.stage-heading{padding:14px}.editors{padding:0 10px}}
</style>
