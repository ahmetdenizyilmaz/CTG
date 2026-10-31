"""Import the user's finished local sessions without changing any original file.

Run once from the development checkout beside SoundBit and CTG-Studio. Imported
projects/media and the generated catalog are committed, so consumers need neither.
"""
import os
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT.parent / 'SoundBit/out'
PUBLIC = ROOT / 'apps/desktop/public/demos'
SITE = ROOT.parent / 'CTG-site/dist'
CLI = Path(os.environ.get("CTG_CLI", ROOT.parent/"CTG-Studio/target/debug/ctg.exe"))
CATALOG = []
COLORS = ['#10d9a0', '#38bdf8', '#c4a5ff', '#f5b942', '#fb8c7c']

def slug(text):
    return re.sub(r'[^a-z0-9]+', '-', text.lower()).strip('-')

def duration(path):
    return float(subprocess.check_output(['ffprobe', '-v', 'error', '-show_entries',
        'format=duration', '-of', 'default=noprint_wrappers=1:nokey=1', str(path)]))

def add(id, title, author, detail, audio, project=None, category='Earlier sessions', bpm=None):
    folder = ROOT / 'examples/library' / id
    folder.mkdir(parents=True, exist_ok=True)
    audio_target = folder / (id + audio.suffix.lower())
    shutil.copy2(audio, audio_target)
    seconds = duration(audio_target)
    tracks = None
    project_path = None
    if project:
        project_target = folder / (id + '.sbproj')
        shutil.copy2(project, project_target)
        song = json.loads(project_target.read_text(encoding='utf-8'))['song']
        tracks = len(song['tracks']); bpm = song['timing']['bpm']
        project_path = project_target.relative_to(ROOT).as_posix()
    entry = dict(id=id, title=title, author=author, detail=detail, category=category,
        bpm=bpm, seconds=round(seconds, 2), time=f'{int(seconds)//60}:{int(seconds)%60:02}',
        tracks=tracks, project=project_path, audio=f'audio/{id}{audio.suffix.lower()}',
        color=COLORS[len(CATALOG) % len(COLORS)],
        audio_sha256=hashlib.sha256(audio.read_bytes()).hexdigest())
    CATALOG.append(entry)
    for base in [PUBLIC, SITE]:
        (base / 'audio').mkdir(parents=True, exist_ok=True)
        shutil.copy2(audio, base / entry['audio'])
        if project:
            (base / 'projects').mkdir(exist_ok=True)
            shutil.copy2(project, base / f'projects/{id}.sbproj')

add('afterglow-circuit','Afterglow Circuit','AI guest composer',
    'An emotional bowed theme becomes syncopated funk. Deliberately authored through 325 MCP calls, with no random generator.',
    ROOT/'examples/afterglow-circuit/Afterglow Circuit.wav',ROOT/'examples/afterglow-circuit/Afterglow Circuit.sbproj','Featured')
for id,title,detail in [('electric-signals','Electric Signals','Bright brass, walking bass and a broken backbeat.'),
    ('neon-satellites','Neon Satellites','A half-time title theme with space to breathe.'),
    ('neon-footsteps','Neon Footsteps','An agile adventure melody and playful octave bass.')]:
    add(id,title,'CTG composer v2',detail,ROOT/f'examples/{id}/{id}.mp3',ROOT/f'examples/{id}/{id}.sbproj','Featured')

NATIVE = {
    'medieval-battle-song': ('Iron & Banner', 'Orchestral medieval battle theme.', 'Antigravity'),
    'star-collapse-alt1-elegy': ('Starlight Elegy', 'Warm analog synthwave, composed for the star-collapse film.', 'Antigravity'),
    'star-collapse-alt2-odyssey': ('Celestial Odyssey', 'Melodic cosmic chiptune and orchestra.', 'Antigravity'),
    'star-collapse-antigravity': ('Schwarzschild Singularity', 'A cinematic star-collapse score with integrated musical sound effects.', 'Antigravity'),
    'star-collapse-original-song': ('Eventide of the Giants', 'An original song for the star-collapse sequence.', 'Antigravity'),
    'paper-satellites': ('Paper Satellites', 'One theme across emotional triplets and swung funk.', 'Ady & Claude'),
    'star-to-black-hole': ('Twenty Suns', 'A synchronized stellar soundtrack with musical effects.', 'Ady & Claude'),
    'neon-afterburn': ('Neon Afterburn', 'An earlier native synth composition from the first sessions.', 'Ady & Codex'),
    'velvet-voltage': ('Velvet Voltage', 'Emotional passages and a fast electronic groove.', 'Ady & Codex'),
    'star-collapse': ('Last Light', 'A star-to-black-hole soundtrack with integrated effects.', 'Ady & Codex'),
}
for directory,(title,detail,author) in NATIVE.items():
    folder=SOURCE/directory
    projects=[p for p in folder.glob('*.sbproj') if p.stem not in ('music','effects')]
    assert len(projects)==1,(directory,projects)
    audios=sorted(folder.glob('*.mp3'),key=lambda p:(len(p.name),p.name))
    assert audios,directory
    add(slug(title),title,author,detail,audios[0],projects[0])

zzfx=SOURCE/'zzfx-antigravity-songs'
descriptions={x['folder']:x for x in json.loads((zzfx/'manifest.json').read_text(encoding='utf-8'))}
for folder in sorted(p for p in zzfx.iterdir() if p.is_dir()):
    audios=list(folder.glob('*.mp3'))
    if not audios:continue
    assert len(audios)==1,folder
    info=descriptions.get(folder.name,{})
    title=info.get('title',audios[0].stem)
    target=ROOT/'examples/library'/('zzfx-'+slug(title))
    target.mkdir(parents=True,exist_ok=True)
    original=next(folder.glob('*.zzfx'))
    shutil.copy2(original,target/'original.zzfx')
    shutil.copy2(folder/'song.json',target/'original-song.json')
    converted=ROOT/'out'/(slug(title)+'-import.sbproj')
    # Only regenerate this script's disposable conversion output.
    if converted.exists():converted.unlink()
    subprocess.run([str(CLI),'import',str(original),'-o',str(converted)],check=True)
    add('zzfx-'+slug(title),title,'Antigravity',
        info.get('description','An earlier Antigravity ZzFX session. Original audio preserved.'),
        audios[0],project=converted,category='ZzFX archive',bpm=info.get('config',{}).get('bpm'))

for project in sorted((ROOT/'examples/songs').glob('*.sbproj')):
    song=json.loads(project.read_text(encoding='utf-8'))['song']
    preview=ROOT/'out'/(project.stem+'-preview.wav')
    mp3=preview.with_suffix('.mp3')
    subprocess.run([str(CLI),'render',str(project),'-o',str(preview)],check=True)
    subprocess.run(['ffmpeg','-v','error','-y','-i',str(preview),'-codec:a','libmp3lame','-b:a','192k',str(mp3)],check=True)
    add(project.stem,song['meta']['title'],song['meta'].get('author') or 'Earlier native sessions',
        song['meta'].get('description') or 'An original native session, with its complete instrument palette.',mp3,project)

catalog_text=json.dumps(CATALOG,indent=2,ensure_ascii=False)+'\n'
(ROOT/'apps/desktop/src/lib/music/demos.json').write_text(catalog_text,encoding='utf-8')
for folder in [PUBLIC,SITE,ROOT/'examples/library']:
    (folder/'catalog.json').write_text(catalog_text,encoding='utf-8')

# Every demo now has an editable native project, including imported ZzFX scores.
lines=['//! Embedded user demo projects. Generated by scripts/import-demo-library.py.',
       'pub(crate) fn source(id: &str) -> Option<&\'static str> {','    match id {']
for item in CATALOG:
    if item['project']:
        lines.append(f'        "{item["id"]}" => Some(include_str!("../../../../../{item["project"]}")),')
lines+=['        _ => None,','    }','}']
(ROOT/'apps/desktop/src-tauri/src/commands/demo_catalog.rs').write_text('\n'.join(lines)+'\n',encoding='utf-8')
print(json.dumps({'songs':len(CATALOG),'native_projects':sum(x['project'] is not None for x in CATALOG),
                  'audio_only':sum(x['project'] is None for x in CATALOG)},indent=2))
