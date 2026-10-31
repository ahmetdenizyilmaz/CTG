//! Reproducible, non-gating benchmark for editor hot paths.
use soundbit_app::{
    PresetLibrary,
    model::{InstrumentId, Note, Pattern, Pitch},
};
use std::{hint::black_box, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    for _ in 0..2_000 {
        black_box(PresetLibrary::builtin()?);
    }
    println!(
        "preset_load_2000_ms={:.3}",
        start.elapsed().as_secs_f64() * 1000.0
    );
    let mut pattern = Pattern::new("benchmark", InstrumentId::new(1));
    let notes: Vec<_> = (0..512)
        .rev()
        .map(|n| Note::new(Pitch::default(), n, 1))
        .collect::<Result<_, _>>()?;
    let start = Instant::now();
    for _ in 0..2_000 {
        black_box(pattern.set_notes(notes.clone())?);
    }
    println!(
        "set_512_notes_2000_ms={:.3}",
        start.elapsed().as_secs_f64() * 1000.0
    );
    Ok(())
}
