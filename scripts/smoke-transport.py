"""Playback regression checks in a disposable CTG desktop (CDP 9241 only)."""
import json
from copy import deepcopy
from pathlib import Path
from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[1]
# A short real arrangement makes loop-on/off observable without a minute-long wait.
project = json.loads((ROOT / 'examples/songs/pixel-sunrise.sbproj').read_text(encoding='utf-8'))
for track in project['song']['tracks']:
    track['clips'] = [dict(track['clips'][0], bar=0)] if track['clips'] else []
project['song']['loop_range'] = None
fixture = ROOT / 'out/transport-one-bar.sbproj'
fixture.write_text(json.dumps(project), encoding='utf-8')
zzfx = deepcopy(project)
zzfx['format_version'] = 3
zzfx['song']['tracks'] = [zzfx['song']['tracks'][0]]
zzfx['song']['patterns'] = [zzfx['song']['patterns'][0]]
zzfx['song']['patterns'][0]['notes'] = [dict(pitch=60, start=0, length=16, velocity=90)]
zzfx['song']['instruments'] = [zzfx['song']['instruments'][0]]
zzfx['song']['instruments'][0]['voice'] = dict(type='zzfx', parameters=[.5,0,261.625565,.004,1.8,.18,5,.85,0,0,0,0,0,0,0,0,.028,.88,.03,0,0])
zzfx['song']['instruments'][0]['reverb_send'] = 0
sample_fixture = ROOT / 'out/transport-zzfx-held.sbproj'
sample_fixture.write_text(json.dumps(zzfx), encoding='utf-8')

with sync_playwright() as p:
    browser = p.chromium.connect_over_cdp('http://127.0.0.1:9241')
    page = browser.contexts[0].pages[0]
    page.set_default_timeout(15000)
    errors = []
    page.on('pageerror', lambda error: errors.append(str(error)))
    def call(command, args=None):
        return page.evaluate('([c,a]) => window.__TAURI_INTERNALS__.invoke(c,a)', [command,args or {}])
    def wait_advancing():
        before = call('playback_state')['position_seconds']
        page.wait_for_function("async(before)=>{const s=await window.__TAURI_INTERNALS__.invoke('playback_state');return s.playing && s.position_seconds!==null && Math.abs(s.position_seconds-(before??0))>0.05;}", arg=before)
    def wait_playing():
        page.wait_for_function("async()=>{const s=await window.__TAURI_INTERNALS__.invoke('playback_state');return s.playing && s.position_seconds>0.12;}")
        wait_advancing()
    call('open_project', {'path':str(fixture)})
    page.reload()
    play = page.get_by_role('button', name='Play', exact=True)
    stop = page.get_by_role('button', name='Stop', exact=True)
    play.wait_for()
    # Exercise real UI -> native audio rather than merely returning optimistic state.
    for _ in range(20):
        play.click()
        wait_playing()
        stop.click()
        page.wait_for_function("async()=>!(await window.__TAURI_INTERNALS__.invoke('playback_state')).playing")
    # A held ZzFX voice must not survive with an index into a replaced native-only bank.
    for _ in range(5):
        call('open_project', {'path':str(sample_fixture)})
        play.click()
        wait_playing()
        call('open_project', {'path':str(fixture)})
        page.wait_for_timeout(250)
        wait_advancing()
        stop.click()
    # Loop enabled mid-song must prevent the upcoming natural stop.
    page.get_by_title('Loop on', exact=True).click()
    play.click()
    wait_playing()
    page.get_by_title('Loop off', exact=True).click()
    page.wait_for_timeout(4500)
    assert call('playback_state')['playing']
    # Loop disabled mid-song must reach a natural end, with tails allowed to ring out.
    page.get_by_title('Loop on', exact=True).click()
    page.wait_for_function("async()=>!(await window.__TAURI_INTERNALS__.invoke('playback_state')).playing")
    play.wait_for()
    # Holding Space should start once, not alternate play/stop on every repeat.
    page.locator('h1').click()
    page.keyboard.down('Space')
    wait_playing()
    for _ in range(5):
        page.keyboard.down('Space')
    page.keyboard.up('Space')
    assert call('playback_state')['playing']
    stop.click()
    # A fresh song generated while playing still accepts a clean Stop -> Play.
    page.get_by_role('button', name='Compose', exact=True).click()
    page.get_by_role('button', name='Random song', exact=True).click()
    page.wait_for_function("async()=> (await window.__TAURI_INTERNALS__.invoke('get_state')).length_bars>1")
    play.click()
    wait_playing()
    page.get_by_role('button', name='Random song', exact=True).click()
    page.wait_for_timeout(1000)
    stop.click()
    play.click()
    wait_playing()
    stop.click()
    assert not errors, errors
    log = ROOT / 'out/qa-playback-stderr.txt'
    assert not log.exists() or 'panicked' not in log.read_text(encoding='utf-8',errors='replace'), 'Audio thread panicked'
    report = dict(repeated_ui_starts=20, live_zzfx_bank_replacements=5, advancing_clock_verified=True, live_loop_enable=True, live_loop_disable=True,
                  held_space=True, generate_while_playing=True, javascript_errors=errors,
                  environment=call('get_environment'))
    (ROOT / 'out/transport-desktop-check.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    page.screenshot(path=str(ROOT / 'out/transport-desktop.png'))
    print(json.dumps(report,indent=2))
    browser.close()
