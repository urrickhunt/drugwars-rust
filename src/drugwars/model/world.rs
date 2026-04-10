use std::fmt;

use crate::balance::player::CAMPAIGN_DAYS;

use super::{Drug, DrugMap};

#[derive(Debug, Clone, Copy)]
pub struct MarketState {
    pub(crate) prices: DrugMap<i64>,
    pub(crate) cheap: DrugMap<bool>,
    pub(crate) expensive: DrugMap<bool>,
}

impl MarketState {
    pub(crate) const fn new() -> Self {
        Self {
            prices: DrugMap::zeros(),
            cheap: DrugMap::all(false),
            expensive: DrugMap::all(false),
        }
    }

    pub(crate) const fn reset_highlights(&mut self) {
        self.cheap = DrugMap::all(false);
        self.expensive = DrugMap::all(false);
    }

    pub(crate) fn set_price(&mut self, drug: Drug, price: i64) {
        self.prices[drug] = price;
    }

    pub(crate) const fn price(&self, drug: Drug) -> i64 {
        self.prices.get(drug)
    }

    pub(crate) fn mark_spike(&mut self, drug: Drug, multiplier: i64) {
        self.prices[drug] *= multiplier;
        self.expensive[drug] = true;
    }

    pub(crate) fn mark_drop(&mut self, drug: Drug, divisor: i64) {
        self.prices[drug] = (self.prices[drug] / divisor).max(1);
        self.cheap[drug] = true;
    }

    pub(crate) const fn is_cheap(&self, drug: Drug) -> bool {
        self.cheap.get(drug)
    }

    pub(crate) const fn is_expensive(&self, drug: Drug) -> bool {
        self.expensive.get(drug)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Begin,
    Normal,
    BronxDo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Borough {
    Bronx,
    Ghetto,
    CentralPark,
    Manhattan,
    ConeyIsland,
    Brooklyn,
}

impl Borough {
    pub const fn from_travel_key(key: char) -> Option<Self> {
        match key {
            '1' => Some(Self::Bronx),
            '2' => Some(Self::Ghetto),
            '3' => Some(Self::CentralPark),
            '4' => Some(Self::Manhattan),
            '5' => Some(Self::ConeyIsland),
            '6' => Some(Self::Brooklyn),
            _ => None,
        }
    }
}

impl fmt::Display for Borough {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Bronx => "BRONX",
            Self::Ghetto => "GHETTO",
            Self::CentralPark => "CENTRAL PARK",
            Self::Manhattan => "MANHATTAN",
            Self::ConeyIsland => "CONEY ISLAND",
            Self::Brooklyn => "BROOKLYN",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayAdvance {
    Continue,
    GameOver,
}

#[derive(Debug, Clone, Copy)]
pub struct WorldState {
    pub(crate) days_left: i64,
    pub(crate) borough: Borough,
    pub(crate) phase: GamePhase,
    pub(crate) cops: i64,
    pub(crate) market: MarketState,
}

impl WorldState {
    pub(crate) const fn new() -> Self {
        Self {
            days_left: CAMPAIGN_DAYS,
            borough: Borough::Bronx,
            phase: GamePhase::Begin,
            cops: 0,
            market: MarketState::new(),
        }
    }

    pub(crate) const fn day(&self) -> i64 {
        CAMPAIGN_DAYS - self.days_left
    }

    pub(crate) const fn days_left(&self) -> i64 {
        self.days_left
    }

    pub(crate) const fn borough(&self) -> Borough {
        self.borough
    }

    pub(crate) const fn phase(&self) -> GamePhase {
        self.phase
    }

    pub(crate) const fn cops(&self) -> i64 {
        self.cops
    }

    pub(crate) const fn price(&self, drug: Drug) -> i64 {
        self.market.price(drug)
    }

    pub(crate) const fn is_cheap(&self, drug: Drug) -> bool {
        self.market.is_cheap(drug)
    }

    pub(crate) const fn is_expensive(&self, drug: Drug) -> bool {
        self.market.is_expensive(drug)
    }

    pub(crate) fn is_in_borough(&self, borough: Borough) -> bool {
        self.borough == borough
    }

    pub(crate) const fn set_phase(&mut self, phase: GamePhase) {
        self.phase = phase;
    }

    pub(crate) fn set_price(&mut self, drug: Drug, price: i64) {
        self.market.set_price(drug, price);
    }

    pub(crate) const fn reset_market_highlights(&mut self) {
        self.market.reset_highlights();
    }

    pub(crate) fn mark_price_spike(&mut self, drug: Drug, multiplier: i64) {
        self.market.mark_spike(drug, multiplier);
    }

    pub(crate) fn mark_price_drop(&mut self, drug: Drug, divisor: i64) {
        self.market.mark_drop(drug, divisor);
    }

    pub(crate) fn travel_to(&mut self, destination: Borough) {
        self.borough = destination;
        self.phase = if destination == Borough::Bronx {
            GamePhase::BronxDo
        } else {
            GamePhase::Normal
        };
    }

    pub(crate) const fn advance_day(&mut self) -> DayAdvance {
        self.days_left -= 1;
        if self.days_left <= 0 {
            DayAdvance::GameOver
        } else {
            DayAdvance::Continue
        }
    }

    pub(crate) const fn set_cops(&mut self, cops: i64) {
        self.cops = cops;
    }

    pub(crate) const fn reduce_cops(&mut self, amount: i64) -> i64 {
        self.cops -= amount;
        self.cops
    }
}
