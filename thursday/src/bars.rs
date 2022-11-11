use minijam::scale::Pitch;

use crate::{EncError, EncNote, Length, MAX_ENCODING_SIZE, PPQN_MAX};

#[derive(Clone)]
pub struct BarBuf {
    ppqn_idx: u16,
    notes: usize,
    buf: Vec<u8>,
}

impl From<EncError> for BarError {
    fn from(e: EncError) -> Self {
        BarError::EncodingErr(e)
    }
}

pub struct Notes<'a> {
    notes: usize,
    buf: &'a [u8],
}

impl<'a> Iterator for Notes<'a> {
    type Item = EncNote;

    fn next(&mut self) -> Option<Self::Item> {
        let (note, rem) = EncNote::take_from_slice(self.buf).ok()?;
        self.buf = rem;
        self.notes = self.notes.saturating_sub(1);
        Some(note)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.notes, Some(self.notes))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BarError {
    BarFull,
    ExceededBarLength,
    EncodingErr(EncError),
}

impl BarBuf {
    pub fn new() -> Self {
        Self {
            ppqn_idx: 0,
            notes: 0,
            buf: Vec::new(),
        }
    }

    // Check if the current buffer is totally full, in terms of
    // the 64-beat/16-bar 4:4 maximum
    fn check_full(&self) -> Result<(), BarError> {
        if self.ppqn_idx >= PPQN_MAX {
            return Err(BarError::BarFull);
        } else {
            Ok(())
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.buf
    }

    // Increment the "start" ppqn index.
    //
    // Note: Will NOT return `BarError::BarFull` if already totally full,
    // and will instead return `BarError::ExceededBarLength`.
    //
    // You should probably call `check_full()` first, before calling this
    // function so the returned errors are consistent.
    fn increment_ppqn(&mut self, length: Length) -> Result<(), BarError> {
        let len = length.to_ppqn();
        let new_idx = self.ppqn_idx + len;

        if new_idx > PPQN_MAX {
            return Err(BarError::ExceededBarLength);
        }

        self.ppqn_idx = new_idx;
        Ok(())
    }

    // Push a rest of a given length to the buffer
    pub fn push_rest_simple(&mut self, length: Length) -> Result<(), BarError> {
        self.check_full()?;
        self.increment_ppqn(length)
    }

    pub fn from_notes_simple<'a, I>(into_iter: I) -> Result<Self, BarError>
    where
        I: IntoIterator<Item = &'a (Length, Pitch, u8)>,
    {
        let mut bbuf = BarBuf::new();
        into_iter
            .into_iter()
            .try_for_each(|(l, p, o)| bbuf.push_note_simple(*l, *p, *o))?;
        Ok(bbuf)
    }

    pub fn push_note_simple(
        &mut self,
        length: Length,
        pitch: Pitch,
        octave: u8,
    ) -> Result<(), BarError> {
        self.check_full()?;

        // Encode the note to a temp buffer, returning if the encoding failed
        let mut buf = [0x00; MAX_ENCODING_SIZE];
        let note = EncNote::new_simple(pitch, octave, self.ppqn_idx, length)?;

        // Extend our internal buffer with the encoded contents
        let rem_len = note.write_to_slice(&mut buf)?.len();
        let used = buf.len() - rem_len;
        self.buf.extend_from_slice(&buf[..used]);

        // Update our tracking variables
        self.increment_ppqn(length)?;
        self.notes += 1;

        Ok(())
    }

    pub fn notes<'a>(&'a self) -> Notes<'a> {
        Notes {
            notes: self.notes,
            buf: &self.buf,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{EncPitch, PPQN_QUARTER};

    use super::*;

    // TODO: move this to a doc comment
    #[test]
    fn showcase() {
        // Length, Pitch, Octave
        let notes = [
            (Length::Quarter, Pitch::F, 3),
            (Length::Eighth, Pitch::A, 4),
            (Length::Eighth, Pitch::C, 4),
            (Length::TripletQuarter, Pitch::A, 3),
            (Length::TripletQuarter, Pitch::D, 4),
            (Length::TripletQuarter, Pitch::E, 5),
        ];

        let bbuf = BarBuf::from_notes_simple(&notes).unwrap();
        println!("len: {}", bbuf.bytes().len());
        println!("[");
        bbuf.bytes().chunks(4).for_each(|ch| {
            print!("  ");
            ch.iter().for_each(|b| print!("0x{:02X}, ", b));
            println!();
        });
        println!("]\n");

        println!(" start | len  |  freq");
        println!(" ppqn  | ppqn |   Hz");
        println!("-------|------|--------");
        for note in bbuf.notes() {
            println!(
                " {:04}  | {:04} | {:04.2}",
                note.start.ppqn_idx,
                note.length.ppqn_ct,
                note.pitch.frequency(),
            );
        }
    }

    #[test]
    fn iter_test() {
        let notes = [
            Pitch::C,
            Pitch::CSharp,
            Pitch::D,
            Pitch::DSharp,
            Pitch::E,
            Pitch::F,
            Pitch::FSharp,
            Pitch::G,
            Pitch::GSharp,
            Pitch::A,
            Pitch::ASharp,
            Pitch::B,
        ];

        let mut bbuf = BarBuf::new();
        for (i, note) in notes.iter().enumerate() {
            bbuf.push_note_simple(Length::Quarter, *note, (i as u8) % 4)
                .unwrap();
        }

        let out = bbuf.notes().collect::<Vec<EncNote>>();
        assert_eq!(out.len(), notes.len());

        for (i, (act_note, exp_note)) in out.iter().zip(notes.iter()).enumerate() {
            let exp_pitch = EncPitch::from_pitch_octave(*exp_note, (i as u8) % 4).unwrap();
            assert_eq!(act_note.pitch, exp_pitch);
            assert_eq!(
                act_note.start.ppqn_idx,
                (i as u16) * Length::Quarter.to_ppqn()
            );
            assert_eq!(act_note.length.ppqn_ct, Length::Quarter.to_ppqn());
        }
    }

    #[test]
    fn from_iter() {
        let simps = [
            (Length::Quarter, Pitch::F, 4),
            (Length::Quarter, Pitch::A, 4),
            (Length::Quarter, Pitch::C, 4),
            (Length::Quarter, Pitch::E, 4),
        ];

        let bbuf = BarBuf::from_notes_simple(&simps).unwrap();
        assert_eq!(bbuf.notes, 4);
    }

    #[test]
    fn buf_smoke() {
        let mut bbuf = BarBuf::new();
        bbuf.push_note_simple(Length::Quarter, Pitch::C, 4).unwrap();

        assert_eq!(bbuf.ppqn_idx, PPQN_QUARTER);
        assert_eq!(bbuf.notes, 1);
        assert_eq!(bbuf.bytes().len(), 3);
    }

    #[test]
    fn buf_fill() {
        let cases = [
            (Length::SixtyFourth, 64 * 16),
            (Length::Sixteenth, 16 * 16),
            (Length::Eighth, 8 * 16),
            (Length::Quarter, 4 * 16),
            (Length::TripletQuarter, 6 * 16),
            (Length::Whole, 16),
            (Length::QuarterCount(16 * 4), 1),
        ];

        for (len, n) in cases {
            println!("notes: {len:?}, {n}");
            let mut bbuf = BarBuf::new();
            pushn_notes(&mut bbuf, len, n);
            assert_eq!(bbuf.notes, n);
            assert_eq!(bbuf.ppqn_idx, PPQN_MAX);

            let res = bbuf.push_note_simple(Length::SixtyFourth, Pitch::C, 4);
            assert_eq!(res, Err(BarError::BarFull));
        }

        for (len, n) in cases {
            println!("rests: {len:?}, {n}");
            let mut bbuf = BarBuf::new();
            pushn_rests(&mut bbuf, len, n);
            assert_eq!(bbuf.notes, 0);
            assert_eq!(bbuf.ppqn_idx, PPQN_MAX);

            let res = bbuf.push_rest_simple(Length::SixtyFourth);
            assert_eq!(res, Err(BarError::BarFull));
        }
    }

    fn pushn_notes(bbuf: &mut BarBuf, len: Length, ct: usize) {
        for _ in 0..ct {
            bbuf.push_note_simple(len, Pitch::C, 4).unwrap();
        }
    }

    fn pushn_rests(bbuf: &mut BarBuf, len: Length, ct: usize) {
        for _ in 0..ct {
            bbuf.push_rest_simple(len).unwrap();
        }
    }
}
