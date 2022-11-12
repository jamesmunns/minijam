use mididemo::bars_to_midi;
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
    let mut bbuf_1 = BarBuf::new();
    let mut bbuf_2 = BarBuf::new();
    for (len, note) in mary {
        match note {
            Some((pitch, oct)) => bbuf_1.push_note_simple(len, pitch, oct).unwrap(),
            None => bbuf_1.push_rest_simple(len).unwrap(),
        }
    }

    let mary = [
        (Length::Whole, None),
        (Length::Quarter, Some((Pitch::E, 3))), // Ma
        (Length::Quarter, Some((Pitch::D, 3))), // ry
        (Length::Quarter, Some((Pitch::C, 3))), // had
        (Length::Quarter, Some((Pitch::D, 3))), // a
        (Length::Quarter, Some((Pitch::E, 3))), // lit
        (Length::Quarter, Some((Pitch::E, 3))), // tle
        (Length::Quarter, Some((Pitch::E, 3))), // lamb
        (Length::Quarter, None), //
        (Length::Quarter, Some((Pitch::D, 3))), // lit
        (Length::Quarter, Some((Pitch::D, 3))), // tle
        (Length::Quarter, Some((Pitch::D, 3))), // lamb
        (Length::Quarter, None), //
        (Length::Quarter, Some((Pitch::E, 3))), // lit
        (Length::Quarter, Some((Pitch::E, 3))), // tle
        (Length::Quarter, Some((Pitch::E, 3))), // lamb
        (Length::Quarter, None), //
        (Length::Quarter, Some((Pitch::E, 3))), // Ma
        (Length::Quarter, Some((Pitch::D, 3))), // ry
        (Length::Quarter, Some((Pitch::C, 3))), // had
        (Length::Quarter, Some((Pitch::D, 3))), // a
        (Length::Quarter, Some((Pitch::E, 3))), // lit
        (Length::Quarter, Some((Pitch::E, 3))), // tle
        (Length::Quarter, Some((Pitch::E, 3))), // lamb
        (Length::Quarter, Some((Pitch::E, 3))), // its
        (Length::Quarter, Some((Pitch::D, 3))), // fleece
        (Length::Quarter, Some((Pitch::D, 3))), // was
        (Length::Quarter, Some((Pitch::E, 3))), // white
        (Length::Quarter, Some((Pitch::D, 3))), // as
        (Length::Half, Some((Pitch::C, 3))), // snow
    ];
    for (len, note) in mary {
        match note {
            Some((pitch, oct)) => bbuf_2.push_note_simple(len, pitch, oct).unwrap(),
            None => bbuf_2.push_rest_simple(len).unwrap(),
        }
    }

    // Write to midi file
    bars_to_midi(&[(&bbuf_1, Some(1)), (&bbuf_2, Some(33)),], "mary2.mid", 150, ).unwrap();
}
