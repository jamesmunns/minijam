use mididemo::bar_to_midi;
use minijam::scale::Pitch;
use thursday::{Length, bars::BarBuf};

fn main() {
    // Input notation
    let mary = [
        (Length::Quarter, Some((Pitch::E, 4))), // Ma
        (Length::Quarter, Some((Pitch::D, 4))), // ry
        (Length::Quarter, Some((Pitch::C, 4))), // had
        (Length::Quarter, Some((Pitch::D, 4))), // a
        (Length::Quarter, Some((Pitch::E, 4))), // lit
        (Length::Quarter, Some((Pitch::E, 4))), // tle
        (Length::Quarter, Some((Pitch::E, 4))), // lamb
        (Length::Quarter, None), //
        (Length::Quarter, Some((Pitch::D, 4))), // lit
        (Length::Quarter, Some((Pitch::D, 4))), // tle
        (Length::Quarter, Some((Pitch::D, 4))), // lamb
        (Length::Quarter, None), //
        (Length::Quarter, Some((Pitch::E, 4))), // lit
        (Length::Quarter, Some((Pitch::E, 4))), // tle
        (Length::Quarter, Some((Pitch::E, 4))), // lamb
        (Length::Quarter, None), //
        (Length::Quarter, Some((Pitch::E, 4))), // Ma
        (Length::Quarter, Some((Pitch::D, 4))), // ry
        (Length::Quarter, Some((Pitch::C, 4))), // had
        (Length::Quarter, Some((Pitch::D, 4))), // a
        (Length::Quarter, Some((Pitch::E, 4))), // lit
        (Length::Quarter, Some((Pitch::E, 4))), // tle
        (Length::Quarter, Some((Pitch::E, 4))), // lamb
        (Length::Quarter, Some((Pitch::E, 4))), // its
        (Length::Quarter, Some((Pitch::D, 4))), // fleece
        (Length::Quarter, Some((Pitch::D, 4))), // was
        (Length::Quarter, Some((Pitch::E, 4))), // white
        (Length::Quarter, Some((Pitch::D, 4))), // as
        (Length::Half, Some((Pitch::C, 4))), // snow
    ];

    // Load into a Bar Buffer
    let mut bbuf = BarBuf::new();
    for (len, note) in mary {
        match note {
            Some((pitch, oct)) => bbuf.push_note_simple(len, pitch, oct).unwrap(),
            None => bbuf.push_rest_simple(len).unwrap(),
        }
    }

    // Write to midi file
    bar_to_midi(&bbuf, "mary.mid", 150, Some(1)).unwrap();
}
