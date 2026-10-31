"""Generate -> Play regression in the disposable desktop at CDP 9241 only."""
import json
from pathlib import Path
from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[1]
with sync_playwright() as p:
    browser = p.chromium.connect_over_cdp('http://127.0.0.1:9241')
    page = browser.contexts[0].pages[0]
    page.set_default_timeout(15000)
    errors = []
    page.on('pageerror', lambda e: errors.append(str(e)))
    def call(cmd, args=None):
        return page.evaluate('async ([c,a])=>window.__TAURI_INTERNALS__.invoke(c,a)', [cmd,args or {}])
    def state(): return call('get_state')
    def play_and_check():
        play = page.get_by_role('button', name='Play', exact=True)
        assert play.is_enabled()
        play.click()
        page.wait_for_function("async()=>{const s=await window.__TAURI_INTERNALS__.invoke('playback_state');return s.playing && s.position_seconds>0.5;}")
        page.get_by_role('button', name='Stop', exact=True).click()
    call('new_song', {'title':'Empty playback check'})
    page.reload()
    page.get_by_role('button', name='Compose', exact=True).wait_for()
    assert page.get_by_role('button', name='Play', exact=True).is_disabled()
    page.get_by_role('button', name='Compose', exact=True).click()
    assert page.get_by_role('button', name='Neon Pixel Funk recipe', exact=True).count() == 0
    saved = state()['song']
    config = call('composition_defaults')
    preview = call('composition_preview', {'config':config})
    assert preview['composer_version'] == 2
    assert state()['song'] == saved
    page.locator('.composer').get_by_label('Song title', exact=True).fill('Original playback check')
    page.get_by_label('Seed', exact=True).fill('42')
    page.get_by_role('button', name='Generate song', exact=True).click()
    page.wait_for_function("async()=> (await window.__TAURI_INTERNALS__.invoke('get_state')).song.meta.title==='Original playback check'")
    generated = state()['song']
    assert generated['patterns']
    play_and_check()
    page.get_by_role('button', name='Random song', exact=True).click()
    page.wait_for_function("async()=> (await window.__TAURI_INTERNALS__.invoke('get_state')).song.meta.title!=='Original playback check'")
    play_and_check()
    call('undo')
    assert state()['song'] == generated
    call('save_project', {'path': str(ROOT/'out/restored-composer-desktop.sbproj')})
    call('open_project', {'path': str(ROOT/'out/restored-composer-desktop.sbproj')})
    assert state()['song'] == generated
    page.screenshot(path=str(ROOT/'out/restored-composer-desktop.png'))
    assert not errors, errors
    report = dict(composer_version=2, empty_play_disabled=True, named_recipe_removed=True,
        preview_non_mutating=True, generate_then_play=True, random_then_play=True,
        playback_clock_advances=True, undo=True, save_roundtrip=True, javascript_errors=errors)
    (ROOT/'out/restored-composer-desktop-check.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    print(json.dumps(report,indent=2))
    browser.close()
