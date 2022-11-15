#![allow(dead_code)]

use std::fmt::Debug;

use crate::{EncLength, EncStart, Length};
use minijam::{
    scale::{Pitch, Semitones, MAJOR_SCALES, MINOR_SCALES},
    tones::ToneKind,
};
use rand::Rng;

#[derive(Debug, Default)]
pub struct PhraseDataBuilder {
    bpm: Option<u16>,
    key_kind: Option<KeyKind>,
    time_signature: Option<TimeSignature>,
    scale: Option<Scale>,
    num_measures: Option<u8>,
    chord_progression: Option<ChordProgression>,

    key: Option<Pitch>,
    lead_voices: Vec<VoiceData>,
    chorus_voices: Vec<VoiceData>,
}

impl PhraseDataBuilder {
    pub fn fill<R: Rng>(&mut self, rng: &mut R, parameters: &PhraseDataParameters) {
        let key_kind;
        let time_signature;
        let num_measures;

        self.bpm = Some(match self.bpm.take() {
            Some(bpm) => parameters.bpm.step(rng, bpm),
            None => parameters.bpm.generate(rng),
        });
        self.key_kind = Some({
            key_kind = match self.key_kind.take() {
                Some(old) => parameters.key_kind.step(rng, old),
                None => parameters.key_kind.generate(rng),
            };
            key_kind.clone()
        });
        self.time_signature = Some({
            time_signature = match self.time_signature.take() {
                Some(old) => parameters.time_signature.step(rng, old),
                None => parameters.time_signature.generate(rng),
            };
            time_signature.clone()
        });
        self.scale = Some(match self.scale.take() {
            Some(old) => parameters.scale.step(rng, old, key_kind),
            None => parameters.scale.generate(rng, key_kind),
        });
        self.num_measures = Some({
            num_measures = match self.num_measures.take() {
                Some(old) => parameters.num_measures.step(rng, old, time_signature),
                None => parameters.num_measures.generate(rng, time_signature),
            };
            num_measures
        });
        self.chord_progression = Some(match self.chord_progression.take() {
            Some(old) => parameters.chord_progression.step(rng, old, num_measures),
            None => parameters.chord_progression.generate(rng, num_measures),
        });
    }
}

#[derive(Debug, Default)]
pub struct PhraseDataParameters {
    pub bpm: BpmParameters,
    pub key_kind: KeyKindParameters,
    pub time_signature: TimeSignatureParameters,
    pub scale: ScaleParameters,
    pub num_measures: NumMeasuresParameters,
    pub chord_progression: ChordProgressionParameters,
}

#[derive(Debug)]
pub struct BpmParameters {
    pub min: u16,
    pub max: u16,
    pub max_delta_per_phrase: u16,
    pub mutation_probability: f32,
}

impl Default for BpmParameters {
    fn default() -> Self {
        Self {
            min: 50,
            max: 150,
            max_delta_per_phrase: 10,
            mutation_probability: 0.1,
        }
    }
}

impl BpmParameters {
    fn generate<R: Rng>(&self, rng: &mut R) -> u16 {
        rng.gen_range(self.min..=self.max + 1)
    }

    fn step<R: Rng>(&self, rng: &mut R, old: u16) -> u16 {
        if !rng.gen_bool(self.mutation_probability.into()) {
            return old;
        }

        let delta = rng.gen_range(0..=(self.max_delta_per_phrase));
        if rng.gen() {
            let mut new = old;
            new = new.saturating_add(delta);
            new = new.min(self.max);
            new
        } else {
            let mut new = old;
            new = new.saturating_sub(delta);
            new = new.max(self.min);
            new
        }
    }
}

#[derive(Debug)]
pub struct KeyKindParameters {
    pub mutation_probablity: f32,
}

impl Default for KeyKindParameters {
    fn default() -> Self {
        Self {
            mutation_probablity: 0.05,
        }
    }
}

impl KeyKindParameters {
    fn generate<R: Rng>(&self, rng: &mut R) -> KeyKind {
        match rng.gen() {
            true => KeyKind::Major,
            false => KeyKind::Minor,
        }
    }

    fn step<R: Rng>(&self, rng: &mut R, old: KeyKind) -> KeyKind {
        if rng.gen_bool(self.mutation_probablity.into()) {
            self.generate(rng)
        } else {
            old
        }
    }
}

#[derive(Debug)]
pub struct TimeSignatureParameters {
    // TODO: This is probably a place where I want
    // a distribution instead of a mutation?
    num_min: u8,
    num_max: u8,
    num_mutation_probability: f32,
    // TODO: I don't *really* want to deal with changing denoms?
    // no math rock for now
}

impl Default for TimeSignatureParameters {
    fn default() -> Self {
        Self {
            num_min: 3,
            num_max: 6,
            num_mutation_probability: 0.1,
        }
    }
}

impl TimeSignatureParameters {
    fn generate<R: Rng>(&self, rng: &mut R) -> TimeSignature {
        TimeSignature {
            numerator: rng.gen_range(self.num_min..=self.num_max),
            denominator: SignatureDenominator::Quarter,
        }
    }

    fn step<R: Rng>(&self, rng: &mut R, old: TimeSignature) -> TimeSignature {
        if rng.gen_bool(self.num_mutation_probability.into()) {
            self.generate(rng)
        } else {
            old
        }
    }
}

#[derive(Debug)]
pub struct ScaleParameters {
    mutation_probability: f32,
}

impl Default for ScaleParameters {
    fn default() -> Self {
        ScaleParameters {
            mutation_probability: 0.10,
        }
    }
}

impl ScaleParameters {
    fn generate<R: Rng>(&self, rng: &mut R, key_kind: KeyKind) -> Scale {
        match key_kind {
            KeyKind::Major => {
                let idx = rng.gen_range(0..MAJOR_SCALES.len());
                Scale {
                    scale: &MAJOR_SCALES[idx],
                }
            }
            KeyKind::Minor => {
                let idx = rng.gen_range(0..MINOR_SCALES.len());
                Scale {
                    scale: &MINOR_SCALES[idx],
                }
            }
        }
    }

    fn scale_valid(scale: Scale, key_kind: KeyKind) -> bool {
        match key_kind {
            KeyKind::Major => MAJOR_SCALES.contains(&scale.scale),
            KeyKind::Minor => MINOR_SCALES.contains(&scale.scale),
        }
    }

    fn step<R: Rng>(&self, rng: &mut R, old: Scale, key_kind: KeyKind) -> Scale {
        let valid = Self::scale_valid(old.clone(), key_kind.clone());
        if !valid || rng.gen_bool(self.mutation_probability.into()) {
            self.generate(rng, key_kind)
        } else {
            old
        }
    }
}

#[derive(Debug)]
pub struct NumMeasuresParameters {
    min_measures: u8,
    max_measures: u8,
    mutation_probability: f32,
}

impl Default for NumMeasuresParameters {
    fn default() -> Self {
        NumMeasuresParameters {
            min_measures: 3,
            max_measures: 16,
            mutation_probability: 0.1,
        }
    }
}

impl NumMeasuresParameters {
    fn max_meas(&self, time_sig: TimeSignature) -> u8 {
        let lim: u16 = match time_sig.denominator {
            SignatureDenominator::Quarter => 16 * 4,
            SignatureDenominator::Eighth => 16 * 8,
            SignatureDenominator::Sixteenth => 16 * 16,
        };
        let max = (lim / (time_sig.numerator as u16)) as u8;
        let max = max.min(self.max_measures);
        max
    }

    fn generate<R: Rng>(&self, rng: &mut R, time_sig: TimeSignature) -> u8 {
        let max = self.max_meas(time_sig);
        rng.gen_range(self.min_measures..=max)
    }

    fn step<R: Rng>(&self, rng: &mut R, old: u8, time_sig: TimeSignature) -> u8 {
        let max = self.max_meas(time_sig);

        if rng.gen_bool(self.mutation_probability.into()) {
            rng.gen_range(self.min_measures..=max)
        } else {
            old.min(max)
        }
    }
}

#[derive(Debug)]
pub struct ChordProgressionParameters {
    mutation_probability: f32,
}

impl Default for ChordProgressionParameters {
    fn default() -> Self {
        ChordProgressionParameters {
            mutation_probability: 0.1,
        }
    }
}

impl ChordProgressionParameters {
    fn generate<R: Rng>(&self, rng: &mut R, num_meas: u8) -> ChordProgression {
        assert!(num_meas >= 3);
        let mut new = Vec::with_capacity(num_meas as usize);
        new.push(Chord::I);
        for _ in 0..(num_meas - 3) {
            new.push(rng.gen_range(Chord::I as u8..=Chord::VI as u8).into());
        }
        new.push(if rng.gen() { Chord::IV } else { Chord::V });
        new.push(Chord::I);
        ChordProgression { chords: new }
    }

    fn step<R: Rng>(&self, rng: &mut R, mut old: ChordProgression, num_meas: u8) -> ChordProgression {
        assert!(num_meas >= 3);
        let num_meas = num_meas as usize;
        old.chords.resize_with(num_meas, || Chord::I);
        for ch in &mut old.chords[1..(num_meas - 2)] {
            if rng.gen_bool(self.mutation_probability.into()) {
                *ch = rng.gen_range(Chord::I as u8..=Chord::VI as u8).into();
            }
        }
        let sec = &mut old.chords[num_meas - 2];
        let good = (*sec == Chord::IV) || (*sec == Chord::V);
        if !good || rng.gen_bool(self.mutation_probability.into()) {
            *sec = if rng.gen() { Chord::IV } else { Chord::V };
        }
        old.chords[num_meas - 1] = Chord::I;
        old
    }
}


#[derive(Debug)]
pub struct VoiceData {
    resolution: Option<Length>,
    probability: Option<f32>,
    rhythm: Option<Rhythm>,
    voice: Option<ToneKind>,
    refrain_measures: Vec<bool>,
}

#[derive(Debug, Clone)]
pub struct TimeSignature {
    numerator: u8,
    denominator: SignatureDenominator,
}

#[derive(Debug, Clone)]
pub enum SignatureDenominator {
    Quarter,
    Eighth,
    Sixteenth,
}

#[derive(Debug, Clone)]
pub enum KeyKind {
    Major,
    Minor,
}

#[derive(Debug)]
pub struct Rhythm {
    pattern: EncRhythm,
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Chord {
    I,
    II,
    III,
    IV,
    V,
    VI,
}

impl From<u8> for Chord {
    fn from(val: u8) -> Self {
        match val {
            0 => Chord::I,
            1 => Chord::II,
            2 => Chord::III,
            3 => Chord::IV,
            4 => Chord::V,
            5 => Chord::VI,
            _ => Chord::I,
        }
    }
}

pub struct ChordProgression {
    chords: Vec<Chord>,
}

impl Debug for ChordProgression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;
        for (i, st) in self.chords.iter().enumerate() {
            st.fmt(f)?;
            if (i + 1) != self.chords.len() {
                f.write_str(", ")?;
            }
        }
        f.write_str("]")
    }
}

#[derive(Clone)]
pub struct Scale {
    scale: &'static [Semitones],
}

impl Debug for Scale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;
        for (i, st) in self.scale.iter().enumerate() {
            st.0.fmt(f)?;
            if (i + 1) != self.scale.len() {
                f.write_str(", ")?;
            }
        }
        f.write_str("]")
    }
}

#[derive(Debug)]
pub struct EncRhythm {
    start: EncStart,
    length: EncLength,
}
