"""Exercise a disposable CTG desktop at CDP 9241, never the user's live session.
Launch with isolated WEBVIEW2_USER_DATA_FOLDER, CTG_MCP_PORT and CTG_PRESET_DIR.
"""
import json
from pathlib import Path
from playwright.sync_api import sync_playwright

ROOT=Path(__file__).resolve().parents[1]
INVOKE='async ([cmd,args]) => await window.__TAURI_INTERNALS__.invoke(cmd,args)'
catalog=json.loads((ROOT/'apps/desktop/src/lib/music/demos.json').read_text(encoding='utf-8'))
with sync_playwright() as p:
    browser=p.chromium.connect_over_cdp('http://127.0.0.1:9241')
    page=browser.contexts[0].pages[0]
    page.set_default_timeout(12000)
    page.get_by_role('button',name='Compose',exact=True).wait_for()
    def call(name,args=None):return page.evaluate(INVOKE,[name,args or {}])
    def state():return call('get_state')
    errors=[]
    page.on('pageerror',lambda error:errors.append(str(error)))
    # The composition dock must survive hardware choices in a separate popover.
    call('new_song',{'title':'QA selection regression'})
    page.get_by_role('button',name='Compose',exact=True).click()
    borrow=page.get_by_label('Borrow a minor chord',exact=True)
    borrow.check()
    page.get_by_title('Song settings',exact=True).click()
    for profile in ['nes','game_boy','c64','genesis','free']:
        page.get_by_label('Chip profile',exact=True).select_option(profile)
        page.wait_for_function("async (value)=>(await window.__TAURI_INTERNALS__.invoke('get_state')).song.chip_profile===value",arg=profile)
        assert borrow.is_visible() and borrow.is_checked()
    page.keyboard.press('Escape')
    assert borrow.is_visible()
    # Long instrument menus escape the library's scroll clipping and remain clickable.
    page.get_by_role('button',name='Studio',exact=True).click()
    page.get_by_role('button',name='Add instrument',exact=True).click()
    menu=page.get_by_role('menu',name='Add instrument',exact=True)
    item=menu.get_by_role('menuitem',name='CTG Deep Sine Sub',exact=True)
    item.scroll_into_view_if_needed()
    box=menu.bounding_box();viewport=page.evaluate('({width:innerWidth,height:innerHeight})')
    assert box and box['x']>=0 and box['x']+box['width']<=viewport['width']
    assert box['y']>=0 and box['y']+box['height']<=viewport['height']
    assert item.evaluate('(el)=>{const r=el.getBoundingClientRect();return el.contains(document.elementFromPoint(r.x+r.width/2,r.y+r.height/2));}')
    page.screenshot(path=str(ROOT/'out/menu-fixed.png'))
    item.click();page.wait_for_function("async ()=>(await window.__TAURI_INTERNALS__.invoke('get_state')).song.instruments.length===1")
    assert not menu.count()
    # Every native example parses in the packaged app, not just a CLI fixture.
    for demo in catalog:
        if demo['project']:
            loaded=call('open_demo',{'id':demo['id']})
            assert len(loaded['song']['tracks'])==demo['tracks'] and loaded['file'] is None
    before=state()
    try:call('open_demo',{'id':'zzfx-not-native'})
    except Exception:pass
    else:raise AssertionError('Audio-only project unexpectedly opened')
    assert state()==before
    call('open_demo',{'id':'afterglow-circuit'})
    page.reload();page.get_by_label('Instrument name',exact=True).wait_for()
    original=state()['song']['instruments'][0]
    page.get_by_label('Instrument name',exact=True).fill('QA reset sound')
    page.get_by_label('Instrument name',exact=True).press('Tab')
    page.wait_for_function("async ()=>(await window.__TAURI_INTERNALS__.invoke('get_state')).song.instruments[0].name==='QA reset sound'")
    page.get_by_role('button',name='Reset',exact=True).click()
    page.wait_for_function("async (name)=>(await window.__TAURI_INTERNALS__.invoke('get_state')).song.instruments[0].name===name",arg=original['name'])
    assert state()['song']['instruments'][0]==original
    page.get_by_role('button',name='Save preset',exact=True).click()
    page.wait_for_function("()=>document.body.innerText.includes('Available under Saved')")
    page.get_by_role('button',name='Play',exact=True).click()
    page.get_by_role('button',name='Stop',exact=True).wait_for()
    assert call('playback_state')['playing']
    page.get_by_role('button',name='Stop',exact=True).click()
    call('save_project',{'path':str(ROOT/'out/qa-roundtrip.sbproj')})
    exported=call('export_wav',{'path':str(ROOT/'out/qa-roundtrip.wav')})
    assert json.loads((ROOT/'out/qa-roundtrip.sbproj').read_text(encoding='utf-8'))['song']==state()['song']
    page.get_by_text('Afterglow 1 | lead',exact=True).click()
    page.screenshot(path=str(ROOT/'out/ctg-studio-final.png'))
    page.get_by_role('button',name='Demo library',exact=True).click()
    assert page.locator('.demo-grid article').count()==22
    page.get_by_label('Demo creator',exact=True).select_option('Antigravity')
    assert page.locator('.demo-grid article').count()==13
    page.get_by_label('Search demos',exact=True).fill('Singularity')
    assert page.locator('.demo-grid article').count()==2
    page.get_by_label('Search demos',exact=True).fill('')
    page.get_by_label('Demo creator',exact=True).select_option('All creators')
    page.locator('audio').first.evaluate('(a)=>a.play()')
    page.wait_for_function('()=>document.querySelector("audio").currentTime>0')
    page.locator('audio').nth(1).evaluate('(a)=>a.play()')
    assert page.locator('audio').first.evaluate('(a)=>a.paused')
    page.locator('audio').nth(1).evaluate('(a)=>a.pause()')
    page.screenshot(path=str(ROOT/'out/ctg-library-final.png'))
    page.get_by_role('button',name='Compose',exact=True).click()
    before=state()['revision']
    page.get_by_role('button',name='Random song',exact=True).click()
    page.wait_for_function("async (rev)=>(await window.__TAURI_INTERNALS__.invoke('get_state')).revision>rev",arg=before)
    page.get_by_role('button',name='Generate song',exact=True).wait_for()
    page.get_by_role('button',name='Undo',exact=True).click()
    page.get_by_role('button',name='Connect AI',exact=True).click()
    assert page.get_by_role('button',name='Copy configuration',exact=True).is_visible()
    assert not errors,errors
    report=dict(compose_stays_open_for_all_chip_profiles=True,instrument_menu_visible_and_clickable=True,
        native_demos_loaded=14,library_tracks=22,creator_and_search_filters=True,one_audio_preview_at_a_time=True,
        reset_restores_sound=True,save_preset=True,playback=True,save_roundtrip=True,export=exported,random_composition=True,javascript_errors=errors)
    (ROOT/'out/workstation-check.json').write_text(json.dumps(report,indent=2))
    print(json.dumps(report,indent=2))
