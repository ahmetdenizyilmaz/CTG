"""Package built CTG binaries. CARGO_TARGET_DIR is respected when using a shared cache."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import zipfile

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'out'
PACKAGE = OUT / 'CTG-0.3.2-Windows'
PACKAGE.mkdir(parents=True, exist_ok=True)
TARGET = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target'))
for name in ['ctg-desktop.exe', 'ctg.exe']:
    shutil.copy2(TARGET / 'release' / name, PACKAGE / name)
for name in ['LICENSE', 'README.md', 'THIRD_PARTY_NOTICES.md', 'CHANGELOG.md']:
    shutil.copy2(ROOT / name, PACKAGE / name)
shutil.copy2(ROOT / 'docs/CTG-Studio-guide.txt', PACKAGE / 'CTG-guide.txt')
shutil.copy2(ROOT / 'docs/zzfx-compatibility.txt', PACKAGE / 'zzfx-compatibility.txt')
shutil.copytree(ROOT / 'examples', PACKAGE / 'examples', dirs_exist_ok=True,
                ignore=shutil.ignore_patterns('__pycache__', '*.pyc'))
shutil.copytree(ROOT / 'apps/desktop/public/fonts', PACKAGE / 'font-licenses',
                dirs_exist_ok=True, ignore=shutil.ignore_patterns('*.woff2'))
if (OUT / 'validation-report.json').exists():
    shutil.copy2(OUT / 'validation-report.json', PACKAGE / 'validation-report.json')
checksums = {str(p.relative_to(PACKAGE)): hashlib.sha256(p.read_bytes()).hexdigest()
             for p in PACKAGE.rglob('*') if p.is_file() and p.name != 'SHA256.json'}
(PACKAGE / 'SHA256.json').write_text(json.dumps(checksums, indent=2))
archive = shutil.make_archive(str(PACKAGE), 'zip', OUT, PACKAGE.name)
with zipfile.ZipFile(archive) as file:
    assert file.testzip() is None
print(json.dumps({'folder': str(PACKAGE), 'archive': archive, 'files': len(checksums)}, indent=2))
