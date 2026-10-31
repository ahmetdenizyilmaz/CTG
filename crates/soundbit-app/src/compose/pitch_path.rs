//! Bounded look-ahead: choose a complete bar's melodic route to its destination.
use super::parts::BarContext;

pub(super) struct Target {
    pub(super) desired: i16,
    pub(super) stable: bool,
    pub(super) cadence: bool,
}

#[derive(Clone)]
struct Path {
    notes: Vec<i16>,
    cost: i32,
    last: i16,
    motion: i16,
    repeated: u8,
    tension: bool,
}

pub(super) fn plan(
    context: &BarContext<'_>,
    targets: &[Target],
    previous: i16,
    motion: i16,
) -> Vec<i16> {
    let mut paths = vec![Path {
        notes: Vec::new(),
        cost: 0,
        last: previous,
        motion,
        repeated: 0,
        tension: false,
    }];
    for target in targets {
        let candidates: Vec<_> = (48_i16..=88)
            .filter(|pitch| {
                let pc = pitch % 12;
                if target.cadence {
                    return pc == context.chord.root;
                }
                if target.stable {
                    return context.chord.tones.contains(&pc);
                }
                context
                    .scale
                    .kind
                    .intervals()
                    .iter()
                    .any(|d| (i16::from(context.scale.root.get()) + i16::from(*d)) % 12 == pc)
            })
            .collect();
        let mut next = Vec::new();
        for path in &paths {
            for &pitch in &candidates {
                let movement = pitch - path.last;
                let distance = movement.abs();
                let tension = !context.chord.tones.contains(&(pitch % 12));
                let mut cost = i32::from((pitch - target.desired).abs()) * 3;
                if context.config.voice_leading {
                    cost += i32::from(distance + (distance - 5).max(0) * 4);
                    if path.motion.abs() > 4
                        && (movement.signum() == path.motion.signum() || distance > 3)
                    {
                        cost += 16;
                    }
                    // Non-chord tones are connective motion, not isolated arbitrary leaps.
                    if path.tension && (tension || distance > 2) {
                        cost += 22;
                    }
                    if tension && distance > 2 {
                        cost += 12;
                    }
                }
                let repeated = if movement == 0 { path.repeated + 1 } else { 0 };
                cost += i32::from(repeated.saturating_sub(1)) * 12;
                let mut notes = path.notes.clone();
                notes.push(pitch);
                next.push(Path {
                    notes,
                    cost: path.cost + cost,
                    last: pitch,
                    motion: movement,
                    repeated,
                    tension,
                });
            }
        }
        next.sort_by_key(|p| p.cost);
        // Preserve routes with different last pitches/motions for the next target.
        let mut seen = std::collections::BTreeSet::new();
        next.retain(|p| seen.insert((p.last, p.motion, p.repeated)));
        next.truncate(16);
        paths = next;
    }
    paths.into_iter().next().map_or_else(Vec::new, |p| p.notes)
}
