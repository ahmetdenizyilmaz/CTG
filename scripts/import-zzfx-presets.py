"""Preserve upstream archetypes and the user's authored ZzFX sounds as native presets."""
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT.parent / 'SoundBit/out/zzfx-antigravity-songs'
entries=[]
def add(id,name,params,category='zzfx',description='Editable ZzFX sound; all 21 synthesis parameters are available.'):
    params=list(params)+[0]*(21-len(params))
    entries.append(dict(id=id,category=category,description=description,instrument=dict(name=name,voice=dict(type='zzfx',parameters=params),volume=80)))

source=(ROOT.parent/'zzfx-studio/apps/zzfx-studio/src/engine/instruments.ts').read_text(encoding='utf-8')
for role,body in re.findall(r'const (\w+)_ARCHETYPES: Archetype\[\] = \[(.*?)\n\];',source,re.S):
    for name,params in re.findall(r"name: '([^']+)'.*?params: (\[[^\]]+\])",body,re.S):
        params=json.loads(params.replace('FREQ_C4','261.625565').replace('FREQ_C3','130.812783'))
        add('zzfx_'+role.lower()+'_'+name.replace('-','_'),'ZzFX '+role.title()+' '+name.replace('-',' '),params)

for folder in sorted(SOURCE.iterdir()):
    if not (folder/'song.json').exists():continue
    song=json.loads((folder/'song.json').read_text(encoding='utf-8'))
    slug=re.sub(r'[^a-z0-9]+','_',song['config']['name'].lower()).strip('_')
    for i,role in enumerate(['lead','chords','bass','kit']):
        add(f'zzfx_{slug}_{role}',f"ZzFX {song['config']['name']} {role}",song['instruments'][i],description=f"Original {role} sound from {song['config']['name']}. Save preset keeps your own variations.")

neon=json.loads((SOURCE/'02 Neon Pixel Funk/song.json').read_text(encoding='utf-8'))['instruments']
for i,role in enumerate(['lead','chords','bass']):
    params=neon[i].copy()
    # Factory composition presets use standard MIDI tuning; imported projects retain their source register.
    params[2]=261.625565
    add('zzfx_neon_'+role,'ZzFX Neon Funk '+role.title(),params,description='Neon Pixel Funk palette, calibrated to standard C4 tuning for new compositions.')
keys=[.42,0,261.625565,.015,.65,.35,1,1,0,0,0,0,0,0,0,0,.08,.65,.12,0,0]
add('zzfx_midnight_keys','ZzFX Midnight Keys',keys)
for role,params in [
    ('kick',[.9,0,440,.001,.01,.12,0,1,0,0,-80,.012,0,0,0,0,0,.2,.035]),
    ('snare',[.65,0,380,.001,.02,.10,4,1,0,0,0,0,0,.55,0,0,0,.2,.025]),
    ('hat',[.35,0,988,.001,.004,.035,4,1,0,0,0,0,0,.65,0,0,0,.1,.012]),
]:add('zzfx_neon_'+role,'ZzFX Neon '+role.title(),params)

(ROOT/'assets/presets/zzfx.json').write_text(json.dumps(dict(format='soundbit-presets',format_version=1,source='ZzFX Studio archetypes (MIT); Antigravity authored sounds; CTG arrangements',presets=entries),indent=2)+'\n',encoding='utf-8')
legacy=[]
for file in sorted((ROOT/'examples/songs').glob('*.sbproj')):
    song=json.loads(file.read_text(encoding='utf-8'))['song']
    for inst in song['instruments']:
        value={k:v for k,v in inst.items() if k!='id'}
        legacy.append(dict(id='legacy_'+file.stem.replace('-','_')+'_'+str(inst['id']),category='legacy_songs',description='Original instrument from '+song['meta']['title'],instrument=value))
(ROOT/'assets/presets/legacy_songs.json').write_text(json.dumps(dict(format='soundbit-presets',format_version=1,source='Original native example instruments; preserved project authorship',presets=legacy),indent=2)+'\n',encoding='utf-8')
print({'zzfx_presets':len(entries),'legacy_presets':len(legacy),'existing_presets_retained':86})
