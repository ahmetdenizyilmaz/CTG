const audio = document.querySelector('#player');
const status = document.querySelector('#player-status');
let current = null;
function resetButtons() {
  document.querySelectorAll('.song').forEach(row => {
    row.classList.remove('playing');
    const button = row.querySelector('.play');
    button.textContent = '▶';
    button.setAttribute('aria-label', `Play ${row.querySelector('h3').textContent}`);
  });
}
function wireSong(row) {
  row.querySelector('.play').addEventListener('click', async () => {
    if (current === row && !audio.paused) { audio.pause(); return; }
    if (current !== row) { audio.pause(); audio.src = row.dataset.src; current = row; }
    try { await audio.play(); } catch { status.textContent = 'Audio could not start. Please try again.'; }
  });
}
document.querySelectorAll('.song').forEach(wireSong);
audio.addEventListener('play', () => {
  resetButtons(); if (!current) return;
  current.classList.add('playing');
  current.querySelector('.play').textContent = 'Ⅱ';
  current.querySelector('.play').setAttribute('aria-label', `Pause ${current.querySelector('h3').textContent}`);
  status.textContent = `Now playing: ${current.querySelector('h3').textContent}`;
});
audio.addEventListener('pause', () => { resetButtons(); status.textContent = current ? `Paused: ${current.querySelector('h3').textContent}` : 'Choose a track to listen.'; });
audio.addEventListener('ended', () => { resetButtons(); status.textContent = 'Track finished. Open the project to explore its arrangement.'; });
audio.addEventListener('error', () => { resetButtons(); status.textContent = 'The audio file could not be loaded. Please try again.'; });
audio.addEventListener('timeupdate', () => {
  if (current && Number.isFinite(audio.duration)) current.querySelector('.song-progress>div').style.width = `${audio.currentTime / audio.duration * 100}%`;
});
document.querySelector('#copy-config').addEventListener('click', async event => {
  const button = event.currentTarget;
  try { await navigator.clipboard.writeText(document.querySelector('#config').textContent); button.textContent = 'Copied!'; }
  catch { button.textContent = 'Select config to copy'; }
});

async function loadArchive() {
  try {
    const response=await fetch('catalog.json');
    if(!response.ok) throw new Error('Catalog unavailable');
    const songs=(await response.json()).filter(song=>song.category!=='Featured');
    const list=document.querySelector('#archive-list');
    const search=document.querySelector('#archive-search');
    const creator=document.querySelector('#archive-creator');
    function show() {
      if(current && list.contains(current)) { audio.pause();current=null; }
      list.replaceChildren();
      const matches=songs.filter(s=>(!creator.value||s.author===creator.value)&&`${s.title} ${s.author} ${s.detail}`.toLowerCase().includes(search.value.toLowerCase()));
      document.querySelector('#archive-count').textContent=`${matches.length} songs`;
      for(const song of matches) {
        const row=document.createElement('article'); row.className='song';row.dataset.src=song.audio;
        row.innerHTML='<button class="play">&#9654;</button><div class="song-info"><span class="song-type"></span><h3></h3><p></p></div><div class="song-meta"><span></span><span></span></div><div class="song-progress"><div></div></div>';
        row.querySelector('.play').setAttribute('aria-label',`Play ${song.title}`);
        row.querySelector('.song-type').textContent=`${song.author} / ${song.project?'NATIVE PROJECT':'ZZFX AUDIO REFERENCE'}`;
        row.querySelector('h3').textContent=song.title;row.querySelector('p').textContent=song.detail;
        row.querySelector('.song-meta span').textContent=song.bpm?`${song.bpm} BPM`:'';
        row.querySelector('.song-meta span:last-child').textContent=song.time;
        if(song.project) {const link=document.createElement('a');link.className='project';link.href=`projects/${song.id}.sbproj`;link.download='';link.textContent='Project';link.setAttribute('aria-label',`Download ${song.title} project`);row.append(link);}
        wireSong(row);list.append(row);
      }
    }
    search.addEventListener('input',show);creator.addEventListener('change',show);show();
  } catch { document.querySelector('#archive-count').textContent='Library could not load. Please refresh.'; }
}
loadArchive();
