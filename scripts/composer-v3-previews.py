"""Render three reproducible v3 compositions without rewriting generated notes."""
import argparse
import json
from pathlib import Path
import subprocess
import wave
import array
import math

parser = argparse.ArgumentParser()
parser.add_argument('--exe', required=True)
args = parser.parse_args()
ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'examples/composer-v3'
OUT.mkdir(exist_ok=True)
neon = json.loads((ROOT / 'examples/neon-funk/recipe.json').read_text(encoding='utf-8'))
recipes = [
    dict(neon, title='Pocket Comet', seed=73, form='standard'),
    dict(title='Glass Arcade', seed=118, key='D', scale='major', vibe='adventure',
         bpm=128, swing=0, form='standard', expressive_slides=True),
    dict(title='After Midnight', seed=256, key='A', scale='dorian', vibe='funk',
         bpm=110, swing=16, form='standard', neon_funk=True, expressive_slides=True),
]
manifest = []
for recipe in recipes:
    stem = recipe['title'].lower().replace(' ', '-')
    config = OUT / (stem + '.recipe.json')
    config.write_text(json.dumps(recipe, indent=2), encoding='utf-8')
    wav = ROOT / 'out' / (stem + '-v3.wav')
    result = subprocess.run([args.exe, 'compose', '--config', str(config), '-o',
        str(OUT / (stem + '.sbproj')), '--wav', str(wav), '--report',
        str(OUT / (stem + '.report.json')), '--force'], capture_output=True, text=True, check=True)
    report = json.loads(result.stdout)
    assert report['composer_version'] == 3
    with wave.open(str(wav)) as f:
        assert f.getsampwidth() == 2
        samples = array.array('h', f.readframes(f.getnframes()))
    peak = max(abs(x) for x in samples)/32768
    rms = math.sqrt(sum((x/32768)**2 for x in samples)/len(samples))
    assert 0 < peak < 1 and rms > 0.001
    subprocess.run(['ffmpeg', '-hide_banner', '-loglevel', 'error', '-y', '-i', str(wav),
        '-af', 'loudnorm=I=-16:TP=-1.5:LRA=9', '-ar', '48000', '-codec:a', 'libmp3lame',
        '-b:a', '256k', str(OUT / (stem + '.mp3'))], check=True)
    manifest.append(dict(title=recipe['title'], seed=recipe['seed'], project=stem+'.sbproj',
        audio=stem+'.mp3', duration_seconds=report['duration_seconds'], raw_peak=round(peak,4),
        raw_rms=round(rms,4), composer_version=3))
(OUT / 'manifest.json').write_text(json.dumps(manifest, indent=2), encoding='utf-8')
print(json.dumps(manifest, indent=2))
