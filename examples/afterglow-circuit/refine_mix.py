"""Native level revision after the first stem measurement (no DSP outside CTG)."""
import json
from mcp_client import CTG, ROOT
ctg=CTG()
for instrument,volume in [(1,48),(3,58),(7,42)]:
    ctg.call('instrument_update',id=instrument,volume=volume)
ctg.call('song_save',path='Afterglow Circuit.sbproj')
report=ctg.call('export_wav',path='Afterglow Circuit.wav',sample_rate=48000)
(ROOT/'render-report.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
(ROOT/'song-view.json').write_text(json.dumps(ctg.call('song_get'),indent=2),encoding='utf-8')
print(json.dumps(report,indent=2))
