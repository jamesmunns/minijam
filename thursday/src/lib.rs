// I want to be able to store (and later generate)
// musical notes in a bar (or multiple bars).

pub const PPQN: u16 = 192;
pub const PPQN_WHOLE: u16 = PPQN * 4;
pub const PPQN_HALF: u16 = PPQN_WHOLE / 2;
pub const PPQN_QUARTER: u16 = PPQN_WHOLE / 4;
pub const PPQN_EIGHTH: u16 = PPQN_WHOLE / 8;
pub const PPQN_16TH: u16 = PPQN_WHOLE / 16;
pub const PPQN_32ND: u16 = PPQN_WHOLE / 32;
pub const PPQN_64TH: u16 = PPQN_WHOLE / 64;
pub const PPQN_32ND_TRIPLET: u16 = (PPQN_16TH * 2) / 3;
pub const PPQN_16TH_TRIPLET: u16 = (PPQN_EIGHTH * 2) / 3;
pub const PPQN_EIGHTH_TRIPLET: u16 = (PPQN_QUARTER * 2) / 3;
pub const PPQN_QUARTER_TRIPLET: u16 = (PPQN_HALF * 2) / 3;
pub const PPQN_HALF_TRIPLET: u16 = (PPQN_WHOLE * 2) / 3;
pub const QN_BEATS_MAX: u16 = 64;
pub const PPQN_MAX: u16 = QN_BEATS_MAX * PPQN_QUARTER;

#[derive(Debug, PartialEq)]
pub enum EncError {
    ValueOutOfBounds,
    EndOfStream,
}

#[derive(Debug)]
struct EncPitch {
    tone: u8,
    offset: u8,
}

impl From<KCInt> for EncPitch {
    fn from(value: KCInt) -> Self {
        match value {
            KCInt::Long { upper, lower } => {
                let lower = lower & 0x7F;
                EncPitch {
                    tone: lower,
                    offset: upper,
                }
            }
            KCInt::Short(lower) => EncPitch {
                tone: lower,
                offset: 0,
            },
        }
    }
}

#[derive(Debug)]
struct EncStart {
    ppqn_idx: u16,
}

impl TryFrom<KCInt> for EncStart {
    type Error = EncError;

    fn try_from(value: KCInt) -> Result<Self, Self::Error> {
        match value {
            KCInt::Long { upper, lower } => {
                let upper = (upper as u16) << 7;
                let lower = (lower as u16) & 0x7F;
                let count = upper | lower;
                if count >= PPQN_MAX {
                    // NOTE! Cannot be = to PPQN_MAX, as that would be the first
                    // pulse on the NEXT segment
                    Err(EncError::ValueOutOfBounds)
                } else {
                    Ok(EncStart { ppqn_idx: count })
                }
            }
            KCInt::Short(lower) => {
                let eighths: u16 = lower.into();
                Ok(EncStart {
                    ppqn_idx: eighths * PPQN_EIGHTH,
                })
            }
        }
    }
}

#[derive(Debug)]
struct EncLength {
    ppqn_ct: u16,
}

impl TryFrom<KCInt> for EncLength {
    type Error = EncError;

    fn try_from(value: KCInt) -> Result<Self, Self::Error> {
        match value {
            KCInt::Long { upper, lower } => {
                let upper = (upper as u16) << 7;
                let lower = (lower as u16) & 0x7F;
                let count = upper | lower;

                if count == 0 {
                    Err(EncError::ValueOutOfBounds)
                } else if count > PPQN_MAX {
                    // NOTE: CAN be equal to PPQN_MAX, as this would be equivalent
                    // to a 64-QN hold
                    Err(EncError::ValueOutOfBounds)
                } else {
                    Ok(EncLength { ppqn_ct: count })
                }
            }
            KCInt::Short(lower) => {
                let count = match lower {
                    0x00 => PPQN_32ND_TRIPLET,
                    0x01 => PPQN_16TH_TRIPLET,
                    0x02 => PPQN_EIGHTH_TRIPLET,
                    0x03 => PPQN_QUARTER_TRIPLET,
                    0x04 => PPQN_HALF_TRIPLET,
                    0x05 => PPQN_64TH,
                    0x06 => PPQN_32ND,
                    0x07 => PPQN_16TH,
                    0x08 => PPQN_EIGHTH,
                    _x @ 0x09..=0x3F => return Err(EncError::ValueOutOfBounds),
                    x => {
                        // NOTE: We know that `x` here is in the range
                        // 0x40..=0x7F, so this value can not exceed the upper limit
                        let qns = x - 0x3F;
                        let qns: u16 = qns.into();
                        qns * PPQN_QUARTER
                    }
                };

                Ok(EncLength { ppqn_ct: count })
            }
        }
    }
}

pub struct EncNote {
    pitch: EncPitch,
    start: EncStart,
    length: EncLength,
}

impl EncNote {
    pub fn take_from_slice(sli: &[u8]) -> Result<(Self, &[u8]), EncError> {
        let (pitch, sli) = KCInt::take_from_slice(sli).ok_or(EncError::EndOfStream)?;
        let (start, sli) = KCInt::take_from_slice(sli).ok_or(EncError::EndOfStream)?;
        let (length, sli) = KCInt::take_from_slice(sli).ok_or(EncError::EndOfStream)?;
        Ok((
            EncNote {
                pitch: pitch.into(),
                start: start.try_into()?,
                length: length.try_into()?,
            },
            sli,
        ))
    }
}

enum KCInt {
    Long { upper: u8, lower: u8 },
    Short(u8),
}

impl KCInt {
    pub fn take_from_slice(sli: &[u8]) -> Option<(Self, &[u8])> {
        let lower = *sli.get(0)?;
        if (0x80 & lower) != 0 {
            let upper = *sli.get(1)?;
            let remain = &sli[2..];
            Some((KCInt::Long { upper, lower }, remain))
        } else {
            let remain = &sli[1..];
            Some((KCInt::Short(lower), remain))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn smoke_short() {
        let vals = [
            0x3C, // C4
            0x08, // Bar 2, beat 1
            0x41, // half note
        ];
        let (note, remain) = EncNote::take_from_slice(&vals).unwrap();

        assert_eq!(note.pitch.tone, 0x3C);
        assert_eq!(note.pitch.offset, 0x00);

        assert_eq!(note.start.ppqn_idx, PPQN_QUARTER * 4);

        assert_eq!(note.length.ppqn_ct, PPQN_HALF);

        assert!(remain.is_empty());
    }

    #[test]
    fn smoke_long() {
        let vals = [
            0xBC, 0xC0, // C4 + 75%
            0xC0, 0x01, // Bar 1, beat 2
            0xA0, 0x02, // quarter + eighth
        ];
        let (note, remain) = EncNote::take_from_slice(&vals).unwrap();

        assert_eq!(note.pitch.tone, 0x3C);
        assert_eq!(note.pitch.offset, 192);

        assert_eq!(note.start.ppqn_idx, PPQN_QUARTER);

        assert_eq!(note.length.ppqn_ct, PPQN_QUARTER + PPQN_EIGHTH);

        assert!(remain.is_empty());
    }
}
