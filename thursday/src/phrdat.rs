#![allow(dead_code)]

use std::fmt::Debug;

use crate::{EncLength, EncStart, Length};
use minijam::{
    scale::{Pitch, Semitones, MAJOR_SCALES, MINOR_SCALES, PITCHES_PER_OCTAVE},
    tones::ToneKind,
};
use rand::Rng;

#[derive(Debug, Default)]
pub struct PhraseDataBuilder {
    header: PhraseDataHeaderBuilder,
    lead_voices: Vec<LeadVoiceDataBuilder>,
    chorus_voices: Vec<ChorusVoiceDataBuilder>,
}

#[derive(Debug, Default)]
pub struct PhraseDataHeaderBuilder {
    bpm: Option<u16>,
    key_kind: Option<KeyKind>,
    time_signature: Option<TimeSignature>,
    scale: Option<Scale>,
    num_measures: Option<u8>,
    chord_progression: Option<ChordProgression>,
    key: Option<Pitch>,
    voices_ct: Option<(u8, u8)>,
}

#[derive(Debug)]
pub struct PhraseDataHeader {
    bpm: u16,
    key_kind: KeyKind,
    time_signature: TimeSignature,
    scale: Scale,
    num_measures: u8,
    chord_progression: ChordProgression,
    key: Pitch,
    voices_ct: (u8, u8),
}

impl PhraseDataBuilder {
    pub fn build_header(&self) -> PhraseDataHeader {
        PhraseDataHeader {
            bpm: *self.header.bpm.as_ref().unwrap(),
            key_kind: self.header.key_kind.as_ref().unwrap().clone(),
            time_signature: self.header.time_signature.as_ref().unwrap().clone(),
            scale: self.header.scale.as_ref().unwrap().clone(),
            num_measures: *self.header.num_measures.as_ref().unwrap(),
            chord_progression: self.header.chord_progression.as_ref().unwrap().clone(),
            key: *self.header.key.as_ref().unwrap(),
            voices_ct: *self.header.voices_ct.as_ref().unwrap(),
        }
    }

    pub fn fill<R: Rng>(&mut self, rng: &mut R, parameters: &PhraseDataParameters) {
        let key_kind;
        let time_signature;
        let num_measures;
        let lead_voices;
        let chorus_voices;

        self.header.bpm = Some(match self.header.bpm.take() {
            Some(bpm) => parameters.bpm.step(rng, bpm),
            None => parameters.bpm.generate(rng),
        });
        self.header.key_kind = Some({
            key_kind = match self.header.key_kind.take() {
                Some(old) => parameters.key_kind.step(rng, old),
                None => parameters.key_kind.generate(rng),
            };
            key_kind.clone()
        });
        self.header.time_signature = Some({
            time_signature = match self.header.time_signature.take() {
                Some(old) => parameters.time_signature.step(rng, old),
                None => parameters.time_signature.generate(rng),
            };
            time_signature.clone()
        });
        self.header.scale = Some(match self.header.scale.take() {
            Some(old) => parameters.scale.step(rng, old, key_kind),
            None => parameters.scale.generate(rng, key_kind),
        });
        self.header.num_measures = Some({
            num_measures = match self.header.num_measures.take() {
                Some(old) => parameters.num_measures.step(rng, old, time_signature),
                None => parameters.num_measures.generate(rng, time_signature),
            };
            num_measures
        });
        self.header.chord_progression = Some(match self.header.chord_progression.take() {
            Some(old) => parameters.chord_progression.step(rng, old, num_measures),
            None => parameters.chord_progression.generate(rng, num_measures),
        });
        self.header.key = Some(match self.header.key.take() {
            Some(old) => parameters.key.step(rng, old),
            None => parameters.key.generate(rng),
        });
        self.header.voices_ct = Some({
            let (ld, ch) = match self.header.voices_ct.take() {
                Some((old_lead, old_chorus)) => parameters.voices.step(rng, old_lead, old_chorus),
                None => parameters.voices.generate(rng),
            };
            lead_voices = ld;
            chorus_voices = ch;
            (ld, ch)
        });

        // voices
        //
        // TODO: This biases to keep the lowest index voices around. Consider (sometimes?)
        // shuffling/rotating the voices so that we don't have one or more "sticky" ones
        // below the min threshold
        self.lead_voices.resize_with(lead_voices.into(), LeadVoiceDataBuilder::default);
        self.chorus_voices.resize_with(chorus_voices.into(), ChorusVoiceDataBuilder::default);

        let header = self.build_header();

        self.lead_voices.iter_mut().for_each(|lvdb| {
            lvdb.fill(rng, &header);
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
    pub key: KeyParameters,
    pub voices: VoicesParameters,
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
pub struct KeyParameters {
    mutation_probability: f32,
}

impl Default for KeyParameters {
    fn default() -> Self {
        Self { mutation_probability: 0.1 }
    }
}

impl KeyParameters {
    fn generate<R: Rng>(&self, rng: &mut R) -> Pitch {
        let val: u8 = rng.gen_range(0..PITCHES_PER_OCTAVE as u8);
        val.into()
    }

    fn step<R: Rng>(&self, rng: &mut R, old: Pitch) -> Pitch {
        if rng.gen_bool(self.mutation_probability.into()) {
            self.generate(rng)
        } else {
            old
        }
    }
}

#[derive(Debug)]
pub struct VoicesParameters {
    lead_voices_min: u8,
    lead_voices_max: u8,
    chorus_voices_min: u8,
    chorus_voices_max: u8,
    total_voices_max: u8,
    lead_voices_mutation_probability: f32,
    chorus_voices_mutation_probability: f32,
}

impl Default for VoicesParameters {
    fn default() -> Self {
        Self {
            lead_voices_min: 1,
            lead_voices_max: 5,
            chorus_voices_min: 1,
            chorus_voices_max: 3,
            total_voices_max: 8,
            lead_voices_mutation_probability: 0.1,
            chorus_voices_mutation_probability: 0.1,
        }
    }
}

impl VoicesParameters {
    fn gen_chorus<R: Rng>(&self, rng: &mut R) -> u8 {
        rng.gen_range(self.chorus_voices_min..self.chorus_voices_max)
    }

    fn gen_leads<R: Rng>(&self, rng: &mut R, chorus: u8) -> u8 {
        let leads_max = (self.total_voices_max - chorus).min(self.lead_voices_max);
        rng.gen_range(self.lead_voices_min..leads_max)
    }

    fn generate<R: Rng>(&self, rng: &mut R) -> (u8, u8) {
        let chorus = self.gen_chorus(rng);
        let leads = self.gen_leads(rng, chorus);
        (leads, chorus)
    }

    fn step<R: Rng>(&self, rng: &mut R, old_lead: u8, old_chorus: u8) -> (u8, u8) {
        let chorus = if rng.gen_bool(self.chorus_voices_mutation_probability.into()) {
            self.gen_chorus(rng)
        } else {
            old_chorus
        };
        let leads = if rng.gen_bool(self.lead_voices_mutation_probability.into()) {
            self.gen_leads(rng, chorus)
        } else {
            old_lead.min(self.total_voices_max - chorus)
        };
        (leads, chorus)
    }
}

#[derive(Debug, Default)]
pub struct LeadVoiceDataBuilder {
    resolution: Option<Length>,
    probability: Option<f32>,
    rhythm: Option<Rhythm>,
    voice: Option<ToneKind>,
    refrain_measures: Vec<bool>,
}

impl LeadVoiceDataBuilder {
    pub fn fill<R: Rng>(&mut self, rng: &mut R, header: &PhraseDataHeader) {

    }
}

#[derive(Debug, Default)]
pub struct ChorusVoiceDataBuilder {
    resolution: Option<Length>,
    probability: Option<f32>,
    rhythm: Option<Rhythm>,
    voice: Option<ToneKind>,
    refrain_measures: Vec<bool>,
}

#[derive(Debug, Default)]
pub struct LeadVoiceDataParameters {
    resolution: LeadVoiceResolutionParameters,
}

#[derive(Debug, Default)]
pub struct ChorusVoiceDataParameters {
    resolution: ChorusVoiceResolutionParameters,
}

#[derive(Debug)]
pub struct VoiceResolutionParameters {
    resolution_choices: Vec<Length>,
    mutation_percentage: f32,
}

#[derive(Debug)]
pub struct LeadVoiceResolutionParameters(VoiceResolutionParameters);

impl Default for LeadVoiceResolutionParameters {
    fn default() -> Self {
        Self(VoiceResolutionParameters {
            resolution_choices: vec![
                Length::TripletSixteenth,
                Length::TripletEighth,
                Length::TripletQuarter,
                Length::Sixteenth,
                Length::Eighth,
                Length::Quarter,
            ],
            mutation_percentage: 0.1,
        })
    }
}

#[derive(Debug)]
pub struct ChorusVoiceResolutionParameters(VoiceResolutionParameters);

impl Default for ChorusVoiceResolutionParameters {
    fn default() -> Self {
        Self(VoiceResolutionParameters {
            resolution_choices: vec![
                Length::TripletHalf,
                Length::Quarter,
                Length::Half,
                Length::Whole,
            ],
            mutation_percentage: 0.1,
        })
    }
}

impl ChorusVoiceResolutionParameters {
    #[inline]
    fn generate<R: Rng>(&self, rng: &mut R) -> Length {
        self.0.generate(rng)
    }

    #[inline]
    fn step<R: Rng>(&self, rng: &mut R, old: Length) -> (bool, Length) {
        self.0.step(rng, old)
    }
}

impl LeadVoiceResolutionParameters {
    #[inline]
    fn generate<R: Rng>(&self, rng: &mut R) -> Length {
        self.0.generate(rng)
    }

    #[inline]
    fn step<R: Rng>(&self, rng: &mut R, old: Length) -> (bool, Length) {
        self.0.step(rng, old)
    }
}

impl VoiceResolutionParameters {
    fn generate<R: Rng>(&self, rng: &mut R) -> Length {
        let idx = rng.gen_range(0..self.resolution_choices.len());
        self.resolution_choices[idx]
    }

    fn step<R: Rng>(&self, rng: &mut R, old: Length) -> (bool, Length) {
        if rng.gen_bool(self.mutation_percentage.into()) {
            let length = self.generate(rng);
            (old != length, length)
        } else {
            (false, old)
        }
    }
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

#[derive(Clone)]
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
