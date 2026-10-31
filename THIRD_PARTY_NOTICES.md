# CTG attribution and third-party notices

CTG is distributed under GPL-3.0-or-later. Its audio engine and editing foundations derive
from SoundBit by Renas Demirbas and ahmetdenizyilmaz/SoundBit (618b266). Original copyright
and license notices are retained. CTG has an independent application identity, interface,
repository and release process; it is not affiliated with the original authors.

The workstation visual direction follows the user's PixelForge project: slate surfaces,
emerald/cyan accents, compact docks and Geist / JetBrains Mono typography. The CTG mark,
layout and website implementation are new. Font licenses are bundled in
apps/desktop/public/fonts (Geist SIL OFL; JetBrains Mono SIL OFL).


SoundBit's own source code is licensed under the GNU GPL, version 3 or later (see `LICENSE`).
This file lists outside work that SoundBit's code or data is based on. Dependencies pulled in
through Cargo and npm keep their own licenses; `cargo deny` checks that they are compatible.

## Algorithms reimplemented from published descriptions

No code was copied for these; details are in `docs/references/`.

| Work | Author | License | Used in |
|---|---|---|---|
| Freeverb (structure and tuning constants) | Jezar at Dreampoint | Public domain | `crates/soundbit-dsp/src/reverb.rs` |
| Linear trapezoidal state-variable filter | Andrew Simper (Cytomic) | Published paper | `crates/soundbit-dsp/src/filter.rs` |

## Inspiration

SoundBit's simple song model (one-bar patterns placed on an arrangement) is inspired by
[Bosca Ceoil](https://github.com/TerryCavanagh/boscaceoil) by Terry Cavanagh. No code or data was
taken from it.

## Presets and example songs

All presets in `assets/presets/` and all songs in `examples/songs/` are original work by SoundBit
contributors, released under the project license.


CTG Studio composition algorithms adapt ideas and progression / arrangement tables from ZzFX Studio.
Source: https://github.com/thejustinwalsh/zzfx-studio

MIT License

Copyright (c) 2026 Justin Walsh

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.


## ZzFX synthesis compatibility

The Rust procedural sample compiler adapts ZzFX by Frank Force (MIT).
Source: https://github.com/KilledByAPixel/ZzFX
Studio effect mapping, channel expansion and archetypes adapt Justin Walsh's ZzFX Studio under the MIT notice above. ZzFXM playback conventions are credited
to Keith Clark and Frank Force; https://github.com/keithclark/ZzFXM .
Reference sample tests use upstream ZzFX buildSamples.

MIT License

Copyright (c) 2019 Frank Force

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
