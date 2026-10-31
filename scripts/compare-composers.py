"""Compare structural metrics at identical settings; not a perceptual quality score."""
import argparse
import json
from pathlib import Path
import subprocess
import time

parser = argparse.ArgumentParser()
parser.add_argument('--before', required=True)
parser.add_argument('--after', required=True)
args = parser.parse_args()
ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'out/composer-comparison'
OUT.mkdir(parents=True, exist_ok=True)
results = {}
for label, exe in [('before', args.before), ('after', args.after)]:
    counts = dict(intervals=0, repeats=0, leaps_over_octave=0, lead_steps=0,
                  occupied_steps=0, pitch_signatures=set(), rhythms=set())
    started = time.perf_counter()
    for seed in range(48):
        recipe = dict(seed=seed, form='standard', vibe='funk', key='E', scale='dorian')
        config = OUT / 'recipe.json'
        config.write_text(json.dumps(recipe), encoding='utf-8')
        project = OUT / f'{label}-{seed}.sbproj'
        subprocess.run([exe, 'compose', '--config', str(config), '-o', str(project), '--force'],
                       check=True, capture_output=True)
        song = json.loads(project.read_text(encoding='utf-8'))['song']
        patterns = {p['id']: p for p in song['patterns']}
        pitches, rhythm = [], []
        for clip in song['tracks'][0]['clips']:
            notes = patterns[clip['pattern']]['notes']
            pitches.extend(n['pitch'] for n in notes)
            rhythm.append(tuple((n['start'], n['length']) for n in notes))
            counts['lead_steps'] += 16
            counts['occupied_steps'] += sum(n['length'] for n in notes)
        intervals = [b-a for a,b in zip(pitches, pitches[1:])]
        counts['intervals'] += len(intervals)
        counts['repeats'] += sum(i == 0 for i in intervals)
        counts['leaps_over_octave'] += sum(abs(i) > 12 for i in intervals)
        counts['pitch_signatures'].add(tuple(intervals))
        counts['rhythms'].add(tuple(rhythm))
    results[label] = dict(songs=48, repeated_pitch_fraction=round(counts['repeats']/counts['intervals'],3),
        leaps_over_octave=counts['leaps_over_octave'],
        lead_rest_fraction=round(1-counts['occupied_steps']/counts['lead_steps'],3),
        distinct_contours=len(counts['pitch_signatures']), distinct_rhythms=len(counts['rhythms']),
        seconds=round(time.perf_counter()-started,2))
(OUT / 'report.json').write_text(json.dumps(results, indent=2), encoding='utf-8')
print(json.dumps(results, indent=2))
