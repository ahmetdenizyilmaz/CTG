"""Exercise composer v3 in the isolated QA desktop (CDP 9241), never user sessions."""
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
    examples = json.loads((ROOT/'examples/composer-v3/manifest.json').read_text(encoding='utf-8'))
    for example in examples:
        call('open_project', {'path': str(ROOT/'examples/composer-v3'/example['project'])})
        page.reload()
        page.get_by_role('button', name='Play', exact=True).click()
        page.get_by_role('button', name='Stop', exact=True).wait_for()
        assert call('playback_state')['playing']
        page.get_by_role('button', name='Stop', exact=True).click()
        assert state()['song']['meta']['title'] == example['title']
    saved = state()['song']
    config = call('composition_defaults')
    preview = call('composition_preview', {'config': config})
    assert preview['composer_version'] == 3
    assert state()['song'] == saved, 'Preview must not replace music'
    page.get_by_role('button', name='Compose', exact=True).click()
    page.get_by_role('button', name='Neon Pixel Funk recipe', exact=True).click()
    page.get_by_label('Seed', exact=True).fill('73')
    page.get_by_role('button', name='Generate song', exact=True).click()
    page.wait_for_function("async()=> (await window.__TAURI_INTERNALS__.invoke('get_state')).song.meta.title==='Midnight Pixel Groove'")
    assert state()['song']['meta']['description'].startswith('CTG composer v3')
    generated = state()['song']
    page.get_by_role('button', name='Random song', exact=True).click()
    page.wait_for_function("async()=> (await window.__TAURI_INTERNALS__.invoke('get_state')).song.meta.title!=='Midnight Pixel Groove'")
    assert state()['song']['meta']['description'].startswith('CTG composer v3')
    call('undo')
    assert state()['song'] == generated
    call('save_project', {'path': str(ROOT/'out/composer-v3-desktop.sbproj')})
    call('open_project', {'path': str(ROOT/'out/composer-v3-desktop.sbproj')})
    assert state()['song'] == generated
    page.screenshot(path=str(ROOT/'out/composer-v3-desktop.png'))
    assert not errors, errors
    report = dict(composer_version=3, preview_non_mutating=True, generate_button=True,
        random_song_button=True, undo=True, save_roundtrip=True,
        example_playback=[e['title'] for e in examples], javascript_errors=errors)
    (ROOT/'out/composer-v3-desktop-check.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    print(json.dumps(report,indent=2))
    browser.close()
