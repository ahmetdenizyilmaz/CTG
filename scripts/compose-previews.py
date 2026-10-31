"""Render three short examples with the native CLI; no manual note rewriting."""
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXE = ROOT / "target/release/ctg.exe"
OUT = ROOT / "out/creative-previews"
OUT.mkdir(exist_ok=True)
recipes = {}
forms = set()
grooves = set()
for seed in range(256):
    recipe = json.loads(subprocess.check_output([str(EXE), "random-config", "--seed", str(seed)], text=True))
    if recipe["vibe"] not in ["funk", "title_screen", "adventure"] or recipe["vibe"] in recipes:
        continue
    recipe["form"] = "short"
    candidate = OUT / "candidate.recipe.json"
    candidate.write_text(json.dumps(recipe))
    result = subprocess.run([str(EXE), "compose", "--config", str(candidate), "-o", str(OUT / "candidate.sbproj"), "--force"], capture_output=True, text=True, check=True)
    design = json.loads(result.stdout)["design"]
    form = tuple((s["role"], s["bars"]) for s in design["sections"])
    if form in forms or design["groove"] in grooves:
        continue
    recipes[recipe["vibe"]] = recipe
    forms.add(form); grooves.add(design["groove"])
    if len(recipes) == 3:
        break
manifest = []
for index, vibe in enumerate(["funk", "title_screen", "adventure"], 1):
    recipe = recipes[vibe]
    recipe["form"] = "short"
    stem = f"{index:02} - {recipe['title']}"
    config = OUT / f"{stem}.recipe.json"
    config.write_text(json.dumps(recipe, indent=2))
    result = subprocess.run([str(EXE), "compose", "--config", str(config), "-o", str(OUT / f"{stem}.sbproj"),
        "--wav", str(OUT / f"{stem}.wav"), "--report", str(OUT / f"{stem}.report.json"), "--force"],
        capture_output=True, text=True, check=True)
    report = json.loads(result.stdout)
    assert "design" in report, "Expected composer v2"
    subprocess.run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-i", str(OUT / f"{stem}.wav"),
        "-codec:a", "libmp3lame", "-b:a", "320k", str(OUT / f"{stem}.mp3")], check=True)
    manifest.append({"file": stem, "vibe": vibe, "seed": recipe["seed"], "bpm": recipe["bpm"],
        "seconds": report["duration_seconds"], "design": report["design"]})
(OUT / "manifest.json").write_text(json.dumps(manifest, indent=2))
print(json.dumps(manifest, indent=2))
