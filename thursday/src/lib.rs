// I want to be able to store (and later generate)
// musical notes in a bar (or multiple bars).

use minijam::scale::Pitch;

pub mod bars;
pub mod euc;
pub mod phrdat;

pub const PPQN: u16 = 192;
pub const PPQN_WHOLE: u16 = PPQN * 4;
pub const PPQN_HALF: u16 = PPQN_WHOLE / 2;
pub const PPQN_QUARTER: u16 = PPQN_WHOLE / 4;
pub const PPQN_EIGHTH: u16 = PPQN_WHOLE / 8;
pub const PPQN_16TH: u16 = PPQN_WHOLE / 16;
pub const PPQN_32ND: u16 = PPQN_WHOLE / 32;
pub const PPQN_64TH: u16 = PPQN_WHOLE / 64;
pub const PPQN_32ND_TRIPLET: u16 = (PPQN_32ND * 2) / 3;
pub const PPQN_16TH_TRIPLET: u16 = (PPQN_16TH * 2) / 3;
pub const PPQN_EIGHTH_TRIPLET: u16 = (PPQN_EIGHTH * 2) / 3;
pub const PPQN_QUARTER_TRIPLET: u16 = (PPQN_QUARTER * 2) / 3;
pub const PPQN_HALF_TRIPLET: u16 = (PPQN_HALF * 2) / 3;
pub const QN_BEATS_MAX: u16 = 64;
pub const EIGHTH_BEATS_MAX: u8 = 128;
pub const PPQN_MAX: u16 = QN_BEATS_MAX * PPQN_QUARTER;

pub const MIN_ENCODING_SIZE: usize = 3;
pub const MAX_ENCODING_SIZE: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Length {
    TripletThirtySeconds,
    TripletSixteenth,
    TripletEighth,
    TripletQuarter,
    TripletHalf,
    SixtyFourth,
    ThirtySecond,
    Sixteenth,
    Eighth,
    Quarter,
    Half,
    Whole,
    QuarterCount(u8),
    PPQNCount(u16),
}

impl Length {
    pub fn to_ppqn(&self) -> u16 {
        match self {
            Length::TripletThirtySeconds => PPQN_32ND_TRIPLET,
            Length::TripletSixteenth => PPQN_16TH_TRIPLET,
            Length::TripletEighth => PPQN_EIGHTH_TRIPLET,
            Length::TripletQuarter => PPQN_QUARTER_TRIPLET,
            Length::TripletHalf => PPQN_HALF_TRIPLET,
            Length::SixtyFourth => PPQN_64TH,
            Length::ThirtySecond => PPQN_32ND,
            Length::Sixteenth => PPQN_16TH,
            Length::Eighth => PPQN_EIGHTH,
            Length::Quarter => PPQN_QUARTER,
            Length::Half => 2 * PPQN_QUARTER,
            Length::Whole => 4 * PPQN_QUARTER,
            // Todo: error checking on these?
            Length::QuarterCount(n) => (*n as u16) * PPQN_QUARTER,
            Length::PPQNCount(n) => *n,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum EncError {
    ValueOutOfBounds,
    EndOfStream,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EncPitch {
    // NOTE: Valid Range 0..=0x7F
    tone: u8,
    offset: u8,
}

impl EncPitch {
    pub fn from_pitch_octave(pitch: Pitch, octave: u8) -> Result<Self, EncError> {
        let pitch: u8 = pitch.into();

        let pitch = match octave {
            0..=9 => (octave * 12) + pitch,
            10 if pitch <= 7 => (octave * 12) + pitch,
            _ => {
                return Err(EncError::ValueOutOfBounds);
            }
        };

        Ok(EncPitch {
            tone: pitch,
            offset: 0,
        })
    }

    #[allow(dead_code)]
    pub fn frequency(&self) -> f32 {
        let base = tone_to_freq(self.tone);
        if self.offset == 0 {
            base
        } else {
            let next = tone_to_freq(self.tone + 1);
            let weight = (self.offset as f32) / 256.0;
            let diff = next - base;
            base + (diff * weight)
        }
    }
}

#[allow(dead_code)]
fn tone_to_freq(tone: u8) -> f32 {
    let oct = tone / 12;
    let pitch: Pitch = (tone % 12).into();
    pitch.freq_with_octave(oct)
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

impl From<EncPitch> for KCInt {
    fn from(value: EncPitch) -> Self {
        debug_assert!(value.tone < 0x80, "Invalid Tone Value");
        if value.offset == 0 {
            KCInt::Short(value.tone)
        } else {
            KCInt::Long {
                upper: value.offset,
                lower: value.tone | 0x80,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EncStart {
    // NOTE: Valid Range 0..PPQN_MAX
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

impl From<EncStart> for KCInt {
    fn from(value: EncStart) -> Self {
        debug_assert!(value.ppqn_idx < PPQN_MAX);

        let div = value.ppqn_idx / PPQN_EIGHTH;
        let rem = value.ppqn_idx % PPQN_EIGHTH;

        if rem == 0 {
            KCInt::Short(div as u8)
        } else {
            KCInt::Long {
                upper: (value.ppqn_idx >> 7) as u8,
                lower: (value.ppqn_idx as u8) | 0x80,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EncLength {
    // NOTE: Valid Range 0..=PPQN_MAX
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

impl From<EncLength> for KCInt {
    fn from(value: EncLength) -> Self {
        debug_assert!(value.ppqn_ct != 0);
        debug_assert!(value.ppqn_ct <= PPQN_MAX);

        match value.ppqn_ct {
            PPQN_32ND_TRIPLET => KCInt::Short(0x00),
            PPQN_16TH_TRIPLET => KCInt::Short(0x01),
            PPQN_EIGHTH_TRIPLET => KCInt::Short(0x02),
            PPQN_QUARTER_TRIPLET => KCInt::Short(0x03),
            PPQN_HALF_TRIPLET => KCInt::Short(0x04),
            PPQN_64TH => KCInt::Short(0x05),
            PPQN_32ND => KCInt::Short(0x06),
            PPQN_16TH => KCInt::Short(0x07),
            PPQN_EIGHTH => KCInt::Short(0x08),
            _ => {
                let div = value.ppqn_ct / PPQN_QUARTER;
                let rem = value.ppqn_ct % PPQN_QUARTER;

                if rem == 0 {
                    KCInt::Short(0x3F + (div as u8))
                } else {
                    KCInt::Long {
                        upper: (value.ppqn_ct >> 7) as u8,
                        lower: (value.ppqn_ct as u8) | 0x80,
                    }
                }
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
    pub fn new_simple(
        pitch: Pitch,
        octave: u8,
        start_ppqn: u16,
        length: Length,
    ) -> Result<Self, EncError> {
        let pitch = EncPitch::from_pitch_octave(pitch, octave)?;

        if start_ppqn >= PPQN_MAX {
            return Err(EncError::ValueOutOfBounds);
        }

        let ppqn_len = length.to_ppqn();

        Ok(EncNote {
            pitch,
            start: EncStart {
                ppqn_idx: start_ppqn,
            },
            length: EncLength { ppqn_ct: ppqn_len },
        })
    }

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

    pub fn write_to_slice<'a>(&self, sli: &'a mut [u8]) -> Result<&'a mut [u8], EncError> {
        let outs: [KCInt; 3] = [
            self.pitch.clone().into(),
            self.start.clone().into(),
            self.length.clone().into(),
        ];
        outs.into_iter()
            .try_fold(sli, |sli, out| out.write_to_slice(sli))
    }

    pub fn pitch_tone_offset(&self) -> (u8, u8) {
        (self.pitch.tone, self.pitch.offset)
    }

    pub fn ppqn_start(&self) -> u16 {
        self.start.ppqn_idx
    }

    pub fn ppqn_len(&self) -> u16 {
        self.length.ppqn_ct
    }
}

#[derive(Debug, PartialEq, Eq)]
enum KCInt {
    Long {
        upper: u8,
        // NOTE: DOES contain flag bit, values not adjusted
        lower: u8,
    },
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

    pub fn write_to_slice<'a>(&self, sli: &'a mut [u8]) -> Result<&'a mut [u8], EncError> {
        match self {
            KCInt::Long { upper, lower } => {
                if sli.len() < 2 {
                    return Err(EncError::EndOfStream);
                }
                let (dest, rem) = sli.split_at_mut(2);
                dest.copy_from_slice(&[*lower, *upper]);
                Ok(rem)
            }
            KCInt::Short(lower) => {
                let (dest, rem) = sli.split_first_mut().ok_or(EncError::EndOfStream)?;
                *dest = *lower;
                Ok(rem)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pitch_rt_exhaustive() {
        for tone in 0x00..=0x7F {
            for offset in 0x00..=0xFF {
                let pitch = EncPitch { tone, offset };
                let mut buf = [0xFFu8; 2];
                let kcpitch: KCInt = pitch.clone().into();
                let remain = kcpitch.write_to_slice(&mut buf).unwrap();
                let enc_remain_len = remain.len();

                let (dec_kc, remain) = KCInt::take_from_slice(&buf).unwrap();
                let dec_remain_len = remain.len();
                let dec_pitch: EncPitch = dec_kc.into();
                assert_eq!(pitch, dec_pitch);
                assert_eq!(enc_remain_len, dec_remain_len);
            }
        }
    }

    #[test]
    fn start_rt_exhaustive() {
        for start_idx in 0x0000..PPQN_MAX {
            let start = EncStart {
                ppqn_idx: start_idx,
            };
            let mut buf = [0xFFu8; 2];
            let kcstart: KCInt = start.clone().into();
            let remain = kcstart.write_to_slice(&mut buf).unwrap();
            let enc_remain_len = remain.len();

            let (dec_kc, remain) = KCInt::take_from_slice(&buf).unwrap();
            let dec_remain_len = remain.len();
            let dec_start: EncStart = dec_kc.try_into().unwrap();
            assert_eq!(start, dec_start);
            assert_eq!(enc_remain_len, dec_remain_len);
        }
    }

    #[test]
    fn length_rt_exhaustive() {
        for length_ct in 0x0001..=PPQN_MAX {
            let length = EncLength { ppqn_ct: length_ct };
            let mut buf = [0xFFu8; 2];
            let kclength: KCInt = length.clone().into();
            let remain = kclength.write_to_slice(&mut buf).unwrap();
            let enc_remain_len = remain.len();

            let (dec_kc, remain) = KCInt::take_from_slice(&buf).unwrap();
            let dec_remain_len = remain.len();
            let dec_length: EncLength = dec_kc.try_into().unwrap();
            assert_eq!(length, dec_length);
            assert_eq!(enc_remain_len, dec_remain_len);
        }
    }

    #[test]
    fn kcint_rt_exhaustive_short() {
        for i in 0..=127 {
            // decode
            let vals = [i];
            let (short_dec, remain) = KCInt::take_from_slice(&vals).unwrap();
            assert_eq!(short_dec, KCInt::Short(i));
            assert!(remain.is_empty());

            // encode
            let mut dest = [0xFFu8; 1];
            let remain = short_dec.write_to_slice(&mut dest).unwrap();
            assert!(remain.is_empty());
            assert_eq!(&dest, &vals);
        }
    }

    #[test]
    fn kcint_rt_exhaustive_long() {
        for lower in 0x80..=0xFF {
            for upper in 0x00..=0xFF {
                let vals = [lower, upper];
                let (long_dec, remain) = KCInt::take_from_slice(&vals).unwrap();
                assert_eq!(long_dec, KCInt::Long { lower, upper });
                assert!(remain.is_empty());

                // encode
                let mut dest = [0xFFu8; 2];
                let remain = long_dec.write_to_slice(&mut dest).unwrap();
                assert!(remain.is_empty());
                assert_eq!(&dest, &vals);
            }
        }
    }

    #[test]
    fn kcint_early_fail() {
        for lower in 0x80..=0xFF {
            let vals = [lower];
            let result = KCInt::take_from_slice(&vals);
            assert_eq!(result, None);

            // encode
            for upper in 0x00..=0xFF {
                let mut dest = [0xFFu8; 1];
                let result = KCInt::Long { upper, lower }.write_to_slice(&mut dest);
                assert_eq!(result, Err(EncError::EndOfStream));
            }
        }
    }

    #[test]
    fn smoke_full_rt_short() {
        let vals = [
            0x3C, // C4
            0x08, // Bar 2, beat 1
            0x41, // half note
        ];
        let (note, remain) = EncNote::take_from_slice(&vals).unwrap();
        assert!(remain.is_empty());

        let mut out = [0xFF; MIN_ENCODING_SIZE];
        let remain = note.write_to_slice(&mut out).unwrap();
        assert!(remain.is_empty());
        assert_eq!(vals, out);
    }

    #[test]
    fn smoke_full_rt_long() {
        let vals = [
            0xBC, 0xC0, // C4 + 75%
            0xC0, 0x02, // Bar 1, beat 2 + 1/64
            0xA0, 0x02, // quarter + eighth
        ];
        let (note, remain) = EncNote::take_from_slice(&vals).unwrap();
        assert!(remain.is_empty());

        let mut out = [0xFF; MAX_ENCODING_SIZE];
        let remain = note.write_to_slice(&mut out).unwrap();
        assert!(remain.is_empty());
        assert_eq!(vals, out);
    }

    #[test]
    fn smoke_dec_short() {
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
    fn smoke_dec_long() {
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
