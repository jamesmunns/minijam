use minijam::scale::Pitch;

use crate::{EncError, Length, EncNote, PPQN_MAX, MAX_ENCODING_SIZE};


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

#[derive(Debug, PartialEq, Eq)]
pub enum BarError {
    BarFull,
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

    // Check if the current buffer is totally full, in terms of
    // the 64-beat/16-bar 4:4 maximum
    fn check_full(&self) -> Result<(), BarError> {
        if self.ppqn_idx >= PPQN_MAX {
            return Err(BarError::BarFull);
        } else {
            Ok(())
        }
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

    pub fn push_rest_simple(&mut self, length: Length) -> Result<(), BarError> {
        self.check_full()?;
        self.increment_ppqn(length)
    }

    pub fn push_note_simple(&mut self, length: Length, pitch: Pitch, octave: u8) -> Result<(), BarError> {
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
