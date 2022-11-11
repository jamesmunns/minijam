use minijam::scale::Pitch;

use crate::{EncError, Length, EncNote, PPQN_MAX};


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

#[derive(Debug)]
pub enum BarError {
    ExceededBarLength,
    EncodingErr(EncError),
    NotSimpleEnough,
}

impl BarBuf {
    pub fn new() -> Self {
        Self {
            ppqn_idx: 0,
            notes: 0,
            buf: Vec::new(),
        }
    }

    // fn next_eighth_start_ppqn(&self) -> (u8, u16) {
    //     // add (N - 1) to cause the next div to round up
    //     let idx = self.ppqn_idx + (PPQN_EIGHTH - 1);
    //     let eighth = idx / PPQN_EIGHTH;
    //     let roundup = eighth * PPQN_EIGHTH;
    //     (eighth as u8, roundup)
    // }

    pub fn push_note_simple(&mut self, length: Length, pitch: Pitch, octave: u8) -> Result<(), BarError> {
        let mut buf = [0x00; 6];
        let start_idx = self.ppqn_idx;
        let note = EncNote::new_simple(pitch, octave, start_idx, length)?;
        let note_ppqn = note.ppqn_len();
        let new_idx = start_idx + note_ppqn;

        if new_idx > PPQN_MAX {
            return Err(BarError::ExceededBarLength);
        }

        let rem_len = note.write_to_slice(&mut buf)?.len();
        let used = buf.len() - rem_len;
        self.buf.extend_from_slice(&buf[..used]);

        self.ppqn_idx = new_idx;
        self.notes += 1;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::PPQN_QUARTER;

    use super::*;

    #[test]
    fn buf_smoke() {
        let mut bbuf = BarBuf::new();
        bbuf.push_note_simple(Length::Quarter, Pitch::C, 4).unwrap();

        assert_eq!(bbuf.ppqn_idx, PPQN_QUARTER);
        assert_eq!(bbuf.notes, 1);
        assert_eq!(bbuf.buf.len(), 3);
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
            println!("{len:?}, {n}");
            let mut bbuf = BarBuf::new();
            pushn(&mut bbuf, len, n);
            assert_eq!(bbuf.notes, n);
            assert_eq!(bbuf.ppqn_idx, PPQN_MAX);
        }
    }

    fn pushn(bbuf: &mut BarBuf, len: Length, ct: usize) {
        for _ in 0..ct {
            bbuf.push_note_simple(len, Pitch::C, 4).unwrap();
        }
    }
}
