//! Seeded randomness, Euclidean onsets and groove templates.
use super::config::BassRhythm;

pub(super) struct Rng(u64);
impl Rng {
    pub(super) fn new(seed: u32) -> Self {
        Self(u64::from(seed))
    }
    pub(super) fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    pub(super) fn pick(&mut self, count: usize) -> usize {
        usize::try_from(self.next() % u64::try_from(count.max(1)).unwrap_or(1)).unwrap_or(0)
    }
    pub(super) fn chance(&mut self, percent: usize) -> bool {
        self.pick(100) < percent
    }
}

/// Evenly distributed attacks, rotated by `rotation` steps. Returns empty for zero steps.
/// Hits above steps are capped. This is the Euclidean rhythm's modular construction.
pub fn euclidean(hits: usize, steps: usize, rotation: usize) -> Vec<bool> {
    if steps == 0 {
        return Vec::new();
    }
    let hits = hits.min(steps);
    (0..steps)
        .map(|i| ((i + steps - rotation % steps) % steps) * hits % steps < hits)
        .collect()
}

pub(super) fn bass_mask(
    hits: usize,
    mode: BassRhythm,
    kicks: &[bool; 16],
    rng: &mut Rng,
) -> Vec<bool> {
    if mode == BassRhythm::Euclidean || (mode == BassRhythm::Mixed && rng.chance(50)) {
        let masks: Vec<_> = (0..16)
            .map(|rotation| euclidean(hits, 16, rotation))
            .collect();
        let score = |m: &[bool]| {
            m.iter()
                .enumerate()
                .filter(|(_, on)| **on)
                .map(|(i, _)| usize::from(kicks[i]) * 2 + usize::from(i == 0) * 3)
                .sum::<usize>()
        };
        let best = masks.iter().map(|m| score(m)).max().unwrap_or(0);
        let choices: Vec<_> = masks.into_iter().filter(|m| score(m) == best).collect();
        return choices[rng.pick(choices.len())].clone();
    }
    let templates = [
        [0, 3, 6, 8, 10, 11, 14, 15],
        [0, 2, 4, 6, 8, 10, 12, 14],
        [0, 4, 7, 8, 11, 12, 14, 15],
    ];
    let template = templates[rng.pick(templates.len())];
    let mut ranked: Vec<_> = (0..16)
        .map(|i| {
            (
                i,
                usize::from(i == 0) * 30
                    + usize::from(template.contains(&i)) * 10
                    + usize::from(kicks[i]) * 4,
                rng.next(),
            )
        })
        .collect();
    ranked.sort_by_key(|&(_, score, tie)| (std::cmp::Reverse(score), tie));
    let mut mask = vec![false; 16];
    for &(i, _, _) in ranked.iter().take(hits.min(16)) {
        mask[i] = true;
    }
    mask
}
