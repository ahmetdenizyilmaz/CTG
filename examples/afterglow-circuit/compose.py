"""Afterglow Circuit: a deliberately authored score, sent to CTG through MCP.

No random module, random-song feature, composition engine, sample libraries or
external synthesis. The literal lead, harmony and bass phrases below are the
score. Repetition is intentional; sectional variations are explicitly arranged.
Run against a fresh isolated CTG MCP session rooted at this script's directory.
"""
import json
from collections import Counter
from mcp_client import CTG, ROOT

ctg = CTG()
ctg.call('reference_get')
ctg.call('preset_list')
ctg.call('song_new', title='Afterglow Circuit', bpm=124, chip_profile='free')
ctg.call('song_update', author='AI guest composer for CTG Studio',
    description='Original, deliberately authored via low-level CTG Studio MCP. No random-song or song_compose calls. A six-note longing theme becomes a syncopated funk hook; D minor / D Dorian with an F-major bridge. See compose.py and mcp-call-audit.jsonl.',
    swing=10, scale_root='D', scale_kind='natural_minor', reverb_size=47, reverb_damping=68)

# One sound per role except the melody lane, which changes timbre by section.
SOUNDS = [
 ('cello','aloboi_cello_solo','Afterglow / bowed melody',48,-7,23),
 ('lead','ctg_velvet_reed','Circuit / velvet hook',51,5,15),
 ('shine','ctg_singing_pulse','Lift / singing answer',58,9,18),
 ('ep','ctg_soft_ep','Warm electric piano',34,-24,18),
 ('pluck','ctg_wood_pluck','Dry offbeat chord guitar',27,26,9),
 ('pad','ctg_air_choir','Air behind the theme',16,0,25),
 ('bell','ctg_glass_keys','Four points of light',42,32,27),
 ('bass','ctg_rubber_bass','Pocket / rubber bass',62,0,0),
 ('softbass','ctg_finger_bass','Ground / finger bass',60,0,0),
 ('kick','kick','Round dry kick',68,0,0),
 ('snare','snare','Backbeat',39,3,7),
 ('hat','hihat_closed','Quiet sixteenth ticks',24,-20,0),
 ('open','hihat_open','Offbeat breath',18,22,4),
 ('tom','tom','Transition toms',35,-12,9),
 ('crash','crash','Section shimmer',16,30,18),
]
ids={}
for role,preset,name,volume,pan,send in SOUNDS:
    value=ctg.call('instrument_add_preset',preset=preset,name=name)
    ids[role]=value['id']
    ctg.call('instrument_update',id=value['id'],volume=volume,pan=pan,reverb_send_percent=send)
ctg.call('instrument_update',id=ids['lead'],envelope={'attack_ms':12,'decay_ms':150,'sustain_percent':76,'release_ms':90},filter={'mode':'low_pass','cutoff_hz':2900,'resonance_percent':8,'envelope_semitones':5},vibrato={'depth_cents':9,'rate_hz':5.2,'delay_ms':230})
ctg.call('instrument_update',id=ids['shine'],filter={'mode':'low_pass','cutoff_hz':2500,'resonance_percent':6,'envelope_semitones':3},vibrato={'depth_cents':8,'rate_hz':5.3,'delay_ms':220})
ctg.call('instrument_update',id=ids['bass'],filter={'mode':'low_pass','cutoff_hz':480,'resonance_percent':16,'envelope_semitones':18},envelope={'attack_ms':5,'decay_ms':180,'sustain_percent':48,'release_ms':55})
ctg.call('instrument_update',id=ids['cello'],vibrato={'depth_cents':12,'rate_hz':5.2,'delay_ms':220},unison={'voices':1,'detune_cents':0})

tracks={role:ctg.call('track_add',name=name,volume=100)['id'] for role,name in [
 ('melody','01 | Theme and hook'),('ep','02 | Electric piano'),('pluck','03 | Offbeat chords'),
 ('pad','04 | Warm air'),('bell','05 | Answering glass'),('bass','06 | Bass pocket'),
 ('kick','07 | Kick'),('snare','08 | Snare'),('hat','09 | Closed hat'),
 ('open','10 | Open hat'),('tom','11 | Fills'),('crash','12 | Section accents')]}

def note(pitch,start,length=1,velocity=90):
    return dict(note=pitch,start=start,length=length,velocity=velocity)

def phrase(text,velocity=92):
    # The score spelling is pitch@start:length, with optional :velocity.
    out=[]
    for cell in text.split():
        pitch,values=cell.split('@'); a=values.split(':')
        out.append(note(pitch,int(a[0]),int(a[1]),int(a[2]) if len(a)>2 else velocity))
    return out

placements=[]; cache={}; score=[]
def put(lane,sound,bar,notes,label):
    if not notes:return
    for n in notes: assert 0 <= n['start'] and n['start']+n['length'] <= 16
    key=json.dumps([sound,notes],sort_keys=True)
    if key not in cache:
        p=ctg.call('pattern_create',name=label,instrument_id=ids[sound],notes=notes)
        cache[key]=p['id']
    placements.append(dict(track_id=tracks[lane],pattern_id=cache[key],start_bar=bar))
    score.append(dict(bar=bar,lane=lane,sound=sound,notes=notes))

# Root/fifth and close-voiced upper harmony. No low stacked chords obscure bass.
CHORDS={
 'Dm9':('D2','A2',['F3','A3','C4','E4']),
 'Bbmaj7':('Bb1','F2',['F3','A3','Bb3','D4']),
 'Fmaj7':('F2','C3',['E3','A3','C4','F4']),
 'C6':('C2','G2',['E3','G3','A3','D4']),
 'Gm9':('G1','D2',['F3','A3','Bb3','D4']),
 'Ehalf':('E2','Bb2',['G3','Bb3','D4','E4']),
 'A7':('A1','E2',['G3','A3','C#4','E4']),
 'G13':('G1','D2',['F3','B3','D4','E4']),
 'Am7':('A1','E2',['G3','A3','C4','E4']),
 'DmEnd':('D2','A2',['F3','A3','D4','E4']),
}
SECTIONS=[
 ('Window',0,['Dm9','Bbmaj7','Fmaj7','C6']),
 ('Afterglow',4,['Dm9','Bbmaj7','Fmaj7','C6','Gm9','Dm9','Ehalf','A7']),
 ('Circuit',12,['Dm9','G13','Dm9','G13','Bbmaj7','C6','Ehalf','A7']),
 ('Hook',20,['Dm9','Bbmaj7','Fmaj7','C6','Dm9','G13','Bbmaj7','A7']),
 ('Still here',28,['Fmaj7','C6','Dm9','Am7','Bbmaj7','Gm9','Ehalf','A7']),
 ('Reconnect',36,['Dm9','C6','Bbmaj7','A7']),
 ('Together',40,['Dm9','Bbmaj7','Fmaj7','C6','Dm9','G13','Bbmaj7','A7','Dm9','G13','Ehalf','A7']),
 ('Home',52,['Dm9','Bbmaj7','A7','DmEnd']),
]
THEME=[
 'A4@0:3 C5@4:1 D5@6:6 E5@14:1',
 'F5@0:5 E5@6:1 D5@8:5 C5@14:1',
 'A4@0:3 C5@4:3 E5@8:5 D5@14:1',
 'E5@0:5 G5@6:1 E5@8:3 D5@12:3',
 'D5@0:4 Bb4@6:2 A4@8:3 G4@12:3',
 'A4@0:3 C5@4:1 D5@6:8',
 'G4@0:3 Bb4@4:3 D5@8:3 E5@12:3',
 'E5@0:3 D5@4:3 C#5@8:5 A4@14:1',
]
FUNK=[
 'A4@0:2 C5@3:1 D5@6:3 A4@11:1 C5@14:1',
 'B4@0:2 D5@3:1 E5@6:3 D5@11:1 B4@14:1 A4@15:1',
 'A4@0:2 C5@3:1 D5@6:3 F5@10:1 E5@12:1 D5@14:1',
 'B4@0:3 A4@6:1 G4@8:3 D5@14:1',
 'F5@0:2 D5@3:1 A4@6:3 Bb4@11:1 D5@14:1',
 'E5@0:2 D5@3:1 C5@6:3 G4@11:1 A4@14:1',
 'G4@0:2 Bb4@3:1 D5@6:3 E5@11:1 D5@14:1',
 'C#5@0:3 E5@6:2 G5@10:2 E5@13:1 C#5@15:1',
]
HOOK=[
 'A4@0:2 C5@3:1 D5@6:5 F5@12:2 E5@15:1',
 'D5@0:5 F5@6:1 A5@8:4 F5@14:1',
 'E5@0:3 C5@4:2 A4@8:3 C5@12:2 D5@15:1',
 'E5@0:5 G5@6:1 E5@8:3 D5@12:2 C5@15:1',
 'A4@0:2 C5@3:1 D5@6:5 E5@12:1 F5@14:1',
 'G5@0:3 E5@4:2 D5@8:3 B4@12:2 A4@15:1',
 'F5@0:3 D5@4:2 A4@8:3 Bb4@12:2',
 'E5@0:3 D5@4:2 C#5@8:4 A4@14:1',
]
BRIDGE=[
 'A4@0:6 G4@8:2 E4@12:3',
 'G4@0:6 E4@8:6',
 'A4@0:3 C5@4:1 D5@6:8',
 'C5@0:6 A4@8:6',
 'D5@0:7 F5@8:3 E5@12:3',
 'D5@0:7 Bb4@8:6',
 'Bb4@0:5 G4@6:3 E4@12:3',
 'D5@0:7 C#5@8:5 E5@14:1',
]
BUILD=[
 'A4@0:3 C5@4:1 D5@6:6',
 'G4@0:3 A4@4:1 C5@6:6',
 'F4@0:3 A4@4:1 Bb4@6:3 D5@12:3',
 'E5@0:3 G5@4:3 E5@8:2 C#5@11:1',
]
CODA=['A4@0:3 C5@4:1 D5@6:7','F5@0:4 E5@6:2 D5@10:4',
      'D5@0:5 C#5@6:5 A4@12:2','D5@0:13']

def octave(pitch,delta):return pitch[:-1]+str(int(pitch[-1])+delta)

for section,start,chords in SECTIONS:
    energetic=section in ('Circuit','Hook','Together')
    soft=section in ('Window','Afterglow','Still here','Home')
    for local,chord in enumerate(chords):
        bar=start+local; root,fifth,voicing=CHORDS[chord]
        final=bar==55
        if section=='Window':
            leadtexts=['A4@8:3 D5@12:3','F5@0:6 D5@8:4','E5@4:4 C5@10:4','E5@0:6 D5@10:4']
            melody=phrase(leadtexts[local],72); sound='bell'
        else:
            sequence={'Afterglow':THEME,'Circuit':FUNK,'Hook':HOOK,'Still here':BRIDGE,'Reconnect':BUILD,'Together':HOOK+FUNK[:2]+THEME[6:],'Home':CODA}[section]
            melody=phrase(sequence[local],96 if energetic else 85)
            sound='cello' if soft else ('shine' if section=='Together' else 'lead')
        put('melody',sound,bar,melody,f'{section} {local+1} | lead')

        # Deliberately spaced accompaniment leaves the sung register clear.
        if soft:
            ep=[]
            for j,p in enumerate(voicing):ep.append(note(p,j%2,11 if final else 5,64 if section=='Window' else 71))
            if not final:
                for p in voicing[1:]:ep.append(note(p,8,6,56))
        else:
            ep=[note(p,s,2,v) for s,v in [(0,67),(6,57),(10,61)] for p in voicing[1:]]
        if bar==39:ep=[n for n in ep if n['start']<8]
        put('ep','ep',bar,ep,f'{section} {chord} | piano')
        if energetic:
            pl=[note(p,s,1,v) for s,v in [(3,66),(9,58),(14,72)] for p in voicing[:3]]
            if local%4==3:pl=[n for n in pl if n['start']!=14]
            put('pluck','pluck',bar,pl,f'{chord} | pocket stabs')
        if section in ('Window','Afterglow','Still here','Home') or (section=='Together' and local<8):
            pad=[note(p,0,14 if not final else 15,56 if soft else 45) for p in [voicing[0],voicing[2]]]
            put('pad','pad',bar,pad,f'{chord} | two-note air')

        # Glass responds only in gaps: no constant arpeggio competing with lead.
        if energetic and local%4 in (1,3):
            bell=[note(octave(voicing[1],2),12,1,56),note(octave(voicing[2],1),14,1,52)]
            put('bell','bell',bar,bell,f'{section} | glass response')
        if section=='Still here' and local in (1,3,5):
            put('bell','bell',bar,[note(octave(voicing[-1],1),14,1,52)],'Bridge | single light')

        if energetic:
            # Rhythmic root -> octave -> fifth -> anticipated next root. The
            # alternating figure is written, not selected by a random algorithm.
            rhythm=[(0,2,root,105),(3,1,octave(root,1),84),(6,2,root,96),(10,1,fifth,87),(12,2,root,99),(15,1,octave(root,1),79)]
            if local%2:rhythm=[(0,3,root,103),(5,1,fifth,87),(7,1,octave(root,1),84),(10,2,root,97),(14,1,fifth,86)]
            bass=[note(p,s,l,v) for s,l,p,v in rhythm]
            basssound='bass'
        else:
            bass=[note(root,0,7,87),note(fifth,8,3,70),note(root,12,3,78)]
            basssound='softbass'
            if section=='Window':bass=[note(root,0,12,72)]
            if section=='Still here':bass=[note(root,0,10,80),note(fifth,12,3,67)]
            if final:bass=[note(root,0,14,85)]
        if bar==39:bass=bass[:2]
        put('bass',basssound,bar,bass,f'{section} {chord} | bass')

        if energetic:
            kicks=[(0,113),(6,99),(8,105),(14,93)] if local%2==0 else [(0,112),(3,91),(10,104)]
            snares=[(4,102),(12,110)]+([(11,43)] if local%2==1 else [])
            hats=[(s,70 if s%4==0 else (56 if s%2==0 else 35)) for s in range(16)]
            # A few holes create breathing groove.
            hats=[(s,v) for s,v in hats if s not in (7,15)]
            if local%4==3:hats=[(s,v) for s,v in hats if s<12]
            put('open','open',bar,[note('C8',6,1,64)] if local%2 else [note('C8',14,1,61)],'Funk | open breath')
        elif section in ('Afterglow','Still here'):
            kicks=[(0,94),(10,67)] if local%2==0 else [(0,86)]
            snares=[(8,68)]
            hats=[(s,44 if s%4==0 else 31) for s in (0,2,4,6,8,10,12,14)]
            if section=='Still here' and local<4:kicks=[];snares=[];hats=[]
        elif section=='Reconnect':
            kicks=[(0,98),(8,93)]
            snares=[(4,76),(12,85)]
            hats=[(s,48 if s%2==0 else 30) for s in range(0,16,2 if local<2 else 1)]
            if local==3:kicks=[(0,102)];snares=[(4,88),(8,90),(10,70),(11,76)];hats=[(s,v) for s,v in hats if s<12]
        elif section=='Home':
            kicks=[(0,91)] if local in (0,3) else []
            snares=[];hats=[]
        else:kicks=[];snares=[];hats=[]
        put('kick','kick',bar,[note('C2',s,1,v) for s,v in kicks],f'{section} | kick')
        put('snare','snare',bar,[note('A5',s,1,v) for s,v in snares],f'{section} | backbeat')
        put('hat','hat',bar,[note('C8',s,1,v) for s,v in hats],f'{section} | hats')
        if bar in (11,19,27,35,47,51):
            put('tom','tom',bar,phrase('G3@12:1:66 F3@14:1:73 D3@15:1:82'),'Turnaround | three toms')
        if bar in (12,20,40,48,55):
            put('crash','crash',bar,[note('C8',0,2,72 if bar!=55 else 48)],'Arrival | shimmer')
    print('Authored section:',section,flush=True)

for offset in range(0,len(placements),400):
    ctg.call('arrangement_place_many',placements=placements[offset:offset+400])
ctg.call('song_save',path='Afterglow Circuit.sbproj')
report=ctg.call('export_wav',path='Afterglow Circuit.wav',sample_rate=48000)
(ROOT/'render-report.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
(ROOT/'authored-score.json').write_text(json.dumps(score,indent=2),encoding='utf-8')
(ROOT/'song-view.json').write_text(json.dumps(ctg.call('song_get'),indent=2),encoding='utf-8')
(ROOT/'cue-sheet.json').write_text(json.dumps([{'name':name,'start_bar':start,'seconds':round(start*240/124,3),'chords':chords} for name,start,chords in SECTIONS],indent=2),encoding='utf-8')
print(json.dumps({'patterns':len(cache),'clips':len(placements),'render':report},indent=2))
