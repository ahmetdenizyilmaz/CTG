"""Inspect reference MIDI rhythm/interval statistics; never imports notes into CTG.

Install mido into out/reference-analysis/deps before running this optional study.
Upper staff, highest simultaneous onset is a proxy, not full voice separation.
"""
import hashlib
import json
from collections import Counter
from pathlib import Path
import sys
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'out/reference-analysis'
OUT.mkdir(parents=True, exist_ok=True)
sys.path.insert(0, str(OUT / 'deps'))
import mido

SOURCES = {
    'Maple Leaf Rag': 'https://www.mutopiaproject.org/ftp/JoplinS/maple/maple.mid',
    'Bach Invention 1': 'https://www.mutopiaproject.org/ftp/BachJS/BWV772/bach-invention-01/bach-invention-01.mid',
}
results = []
for title, url in SOURCES.items():
    path = OUT / (title + '.mid')
    if not path.exists():
        path.write_bytes(urllib.request.urlopen(url, timeout=30).read())
    midi = mido.MidiFile(path)
    tracks = []
    for index, track in enumerate(midi.tracks):
        tick, attacks = 0, {}
        for msg in track:
            tick += msg.time
            if msg.type == 'note_on' and msg.velocity and msg.channel != 9:
                attacks[tick] = max(attacks.get(tick, 0), msg.note)
        if attacks:
            tracks.append((sum(attacks.values()) / len(attacks), index, sorted(attacks.items())))
    _, index, attacks = max(tracks)
    intervals = [b[1] - a[1] for a, b in zip(attacks, attacks[1:])]
    gaps = [round((b[0] - a[0]) / midi.ticks_per_beat, 4) for a, b in zip(attacks, attacks[1:])]
    cells = Counter(tuple(intervals[i:i+4]) for i in range(len(intervals)-3))
    results.append(dict(title=title, source=url, sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
        method='Highest mean-pitch track; highest pitch at simultaneous onsets; not full voice separation',
        track=index, attacks=len(attacks),
        step_or_repeat_fraction=round(sum(abs(n) <= 2 for n in intervals)/len(intervals), 3),
        common_inter_onset_beats=Counter(gaps).most_common(5),
        recurring_four_interval_cells=sum(n > 1 for n in cells.values()),
        distinct_four_interval_cells=len(cells)))
(OUT / 'analysis.json').write_text(json.dumps(results, indent=2), encoding='utf-8')
print(json.dumps(results, indent=2))
