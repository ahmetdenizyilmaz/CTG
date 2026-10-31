"""ZzFX and expression QA against the disposable desktop at CDP 9241 only."""
import json
import sys
from pathlib import Path
from playwright.sync_api import sync_playwright

ROOT=Path(__file__).resolve().parents[1]
INVOKE='async ([cmd,args]) => await window.__TAURI_INTERNALS__.invoke(cmd,args)'
with sync_playwright() as p:
    browser=p.chromium.connect_over_cdp('http://127.0.0.1:9241')
    page=browser.contexts[0].pages[0]
    page.set_default_timeout(15000)
    if '--local-frontend' in sys.argv:
        bundle=next((ROOT/'apps/desktop/dist/assets').glob('index-*.js'))
        page.route('**/assets/index-*.js',lambda route:route.fulfill(path=str(bundle),content_type='text/javascript'))
    errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
    def call(cmd,args=None):return page.evaluate(INVOKE,[cmd,args or {}])
    def state():return call('get_state')
    original=ROOT/'examples/library/zzfx-neon-pixel-funk/original.zzfx'
    loaded=call('open_project',{'path':str(original)})
    assert loaded['file'] is None and loaded['song']['meta']['title']=='Neon Pixel Funk'
    page.get_by_role('button',name='Studio',exact=True).click();page.reload()
    page.get_by_label('Instrument name',exact=True).wait_for()
    assert len(call('list_presets'))>=168
    original_gain=state()['song']['instruments'][0]['voice']['parameters'][0]
    page.get_by_label('Gain',exact=True).fill('0.33');page.get_by_label('Gain',exact=True).press('Tab')
    page.wait_for_function("async()=> (await window.__TAURI_INTERNALS__.invoke('get_state')).song.instruments[0].voice.parameters[0]===0.33")
    page.get_by_role('button',name='Reset',exact=True).click()
    page.wait_for_function("async(v)=>(await window.__TAURI_INTERNALS__.invoke('get_state')).song.instruments[0].voice.parameters[0]===v",arg=original_gain)
    pattern=next(v for v in state()['song']['patterns'] if v['notes'])
    page.get_by_text(pattern['name'],exact=True).first.click()
    note=pattern['notes'][0]
    canvas=page.locator('.roll canvas')
    point=canvas.evaluate('''(el,n)=>{const width=el.clientWidth;const cell=(width-52)/32;
        const x=52+(n.start+.5)*cell,y=(96-n.pitch+.5)*18;
        el.parentElement.scrollTop=Math.max(0,y-90);const r=el.getBoundingClientRect();
        return {x:r.left+x,y:r.top+y};}''',note)
    page.mouse.click(point['x'],point['y'])
    page.get_by_label('Note slide',exact=True).fill('12');page.get_by_label('Note slide',exact=True).press('Tab')
    page.get_by_label('Stepped glissando',exact=True).check()
    page.get_by_label('ZzFX note effect',exact=True).select_option('SU')
    page.get_by_label('ZzFX effect amount',exact=True).fill('64');page.get_by_label('ZzFX effect amount',exact=True).press('Tab')
    page.get_by_label('Note tail steps',exact=True).fill('2');page.get_by_label('Note tail steps',exact=True).press('Tab')
    page.wait_for_function("async(id)=>(await window.__TAURI_INTERNALS__.invoke('get_state')).song.patterns.find(p=>p.id===id).notes[0].tail_steps===2",arg=pattern['id'])
    edited=next(p for p in state()['song']['patterns'] if p['id']==pattern['id'])['notes'][0]
    assert edited['slide_semitones']==12 and edited['glissando'] and edited['zzfx_effect']=={'code':'SU','value':64}
    page.screenshot(path=str(ROOT/'out/zzfx-note-expression.png'))
    call('save_project',{'path':str(ROOT/'out/zzfx-expression-roundtrip.sbproj')})
    reopened=call('open_project',{'path':str(ROOT/'out/zzfx-expression-roundtrip.sbproj')})
    assert next(p for p in reopened['song']['patterns'] if p['id']==pattern['id'])['notes'][0]==edited
    page.get_by_role('button',name='Compose',exact=True).click()
    page.get_by_role('button',name='Neon Pixel Funk recipe',exact=True).click()
    assert page.locator('.composer select').nth(2).input_value()=='E'
    assert page.locator('.composer select').nth(3).input_value()=='dorian'
    page.get_by_role('button',name='Generate song',exact=True).click()
    page.wait_for_function("async()=> (await window.__TAURI_INTERNALS__.invoke('get_state')).song.meta.title==='Midnight Pixel Groove'")
    generated=state()['song'];assert all(i['voice']['type']=='zzfx' for i in generated['instruments'])
    assert any(n.get('glissando') for p in generated['patterns'] for n in p['notes'])
    page.get_by_role('button',name='Play',exact=True).click()
    page.get_by_role('button',name='Stop',exact=True).wait_for();assert call('playback_state')['playing']
    page.get_by_role('button',name='Stop',exact=True).click()
    assert not errors,errors
    report=dict(import_editable=True,zzfx_sound_edit_and_reset=True,note_expression_ui=True,
        expression_saved_and_reopened=True,neon_recipe=True,zzfx_playback=True,javascript_errors=errors)
    (ROOT/'out/zzfx-desktop-check.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    print(json.dumps(report,indent=2))
