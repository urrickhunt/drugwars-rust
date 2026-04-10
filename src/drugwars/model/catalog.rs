use std::ops::{Index, IndexMut};

use crate::ansi::{CYAN, GREEN, YELLOW};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Drug {
    Cocaine = 0,
    Heroin = 1,
    Acid = 2,
    Weed = 3,
    Speed = 4,
    Ludes = 5,
}

pub const DRUG_COUNT: usize = 6;

pub const ALL_DRUGS: [Drug; DRUG_COUNT] = [
    Drug::Cocaine,
    Drug::Heroin,
    Drug::Acid,
    Drug::Weed,
    Drug::Speed,
    Drug::Ludes,
];

impl Drug {
    pub(crate) const fn idx(self) -> usize {
        self as usize
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Cocaine => "COCAINE",
            Self::Heroin => "HEROIN",
            Self::Acid => "ACID",
            Self::Weed => "WEED",
            Self::Speed => "SPEED",
            Self::Ludes => "LUDES",
        }
    }

    pub(crate) const fn from_key(c: char) -> Option<Self> {
        match c {
            'c' | 'C' => Some(Self::Cocaine),
            'h' | 'H' => Some(Self::Heroin),
            'a' | 'A' => Some(Self::Acid),
            'w' | 'W' => Some(Self::Weed),
            's' | 'S' => Some(Self::Speed),
            'l' | 'L' => Some(Self::Ludes),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DrugMap<T> {
    values: [T; DRUG_COUNT],
}

impl<T> DrugMap<T> {
    pub(crate) const fn new(values: [T; DRUG_COUNT]) -> Self {
        Self { values }
    }
}

impl<T: Copy> DrugMap<T> {
    pub(crate) const fn get(&self, drug: Drug) -> T {
        self.values[drug.idx()]
    }
}

impl DrugMap<i64> {
    pub(crate) const fn zeros() -> Self {
        Self::new([0; DRUG_COUNT])
    }

    pub(crate) fn total(&self) -> i64 {
        self.values.iter().sum()
    }
}

impl DrugMap<bool> {
    pub(crate) const fn all(value: bool) -> Self {
        Self::new([value; DRUG_COUNT])
    }
}

impl<T> Index<Drug> for DrugMap<T> {
    type Output = T;

    fn index(&self, drug: Drug) -> &Self::Output {
        &self.values[drug.idx()]
    }
}

impl<T> IndexMut<Drug> for DrugMap<T> {
    fn index_mut(&mut self, drug: Drug) -> &mut Self::Output {
        &mut self.values[drug.idx()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceEffect {
    Spike(i64),
    Drop(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PriceEvent {
    pub(crate) drug: Drug,
    pub(crate) effect: PriceEffect,
    pub(crate) color: &'static str,
    pub(crate) message: &'static str,
}

pub const PRICE_EVENTS: &[PriceEvent] = &[
    PriceEvent {
        drug: Drug::Cocaine,
        effect: PriceEffect::Spike(6),
        color: CYAN,
        message: "COPS MADE A BIG COKE BUST !! PRICES ARE OUTRAGEOUS !!",
    },
    PriceEvent {
        drug: Drug::Heroin,
        effect: PriceEffect::Spike(6),
        color: CYAN,
        message: "ADDICTS ARE BUYING HEROIN AT OUTRAGEOUS PRICES !!",
    },
    PriceEvent {
        drug: Drug::Weed,
        effect: PriceEffect::Drop(5),
        color: YELLOW,
        message: "COLOMBIAN FREIGHTER DUSTED THE COAST GUARD !!  WEED PRICES HAVE BOTTOMED OUT !!",
    },
    PriceEvent {
        drug: Drug::Cocaine,
        effect: PriceEffect::Drop(6),
        color: GREEN,
        message: "PIGS ARE SELLING CHEAP COCAINE FROM LAST WEEKS RAID !!",
    },
    PriceEvent {
        drug: Drug::Heroin,
        effect: PriceEffect::Drop(6),
        color: GREEN,
        message: "PIGS ARE SELLING CHEAP HEROIN FROM LAST WEEKS RAID !!",
    },
    PriceEvent {
        drug: Drug::Ludes,
        effect: PriceEffect::Drop(15),
        color: YELLOW,
        message: "RIVAL DRUG DEALERS RAIDED A PHARMACY AND ARE SELLING CHEAP LUDES !!",
    },
    PriceEvent {
        drug: Drug::Acid,
        effect: PriceEffect::Drop(10),
        color: YELLOW,
        message: "THE MARKET HAS BEEN FLOODED WITH CHEAP HOME MADE ACID !!",
    },
];
