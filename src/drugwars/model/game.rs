use std::sync::{Arc, Mutex};

use crate::audio::MusicPlayer;
use crate::balance::player::MAX_HANDGUNS;
use crate::engine::RuleError;

use super::{Borough, DayAdvance, Drug, GamePhase, PlayerState, ScreenState, WorldState};

pub struct GameState {
    pub(crate) player: PlayerState,
    pub(crate) world: WorldState,
    pub(crate) screen: ScreenState,
    pub(crate) music: Arc<Mutex<MusicPlayer>>,
}

impl GameState {
    pub(crate) const fn cash(&self) -> i64 {
        self.player.cash()
    }

    pub(crate) const fn bank(&self) -> i64 {
        self.player.bank()
    }

    pub(crate) const fn debt(&self) -> i64 {
        self.player.debt()
    }

    pub(crate) const fn guns(&self) -> i64 {
        self.player.guns()
    }

    pub(crate) const fn has_shotgun(&self) -> bool {
        self.player.has_shotgun()
    }

    pub(crate) const fn damage(&self) -> i64 {
        self.player.damage()
    }

    pub(crate) const fn hold(&self) -> i64 {
        self.player.hold()
    }

    pub(crate) const fn carried_units(&self, drug: Drug) -> i64 {
        self.player.carried_units(drug)
    }

    pub(crate) const fn stashed_units(&self, drug: Drug) -> i64 {
        self.player.stashed_units(drug)
    }

    pub(crate) const fn day(&self) -> i64 {
        self.world.day()
    }

    pub(crate) const fn days_left(&self) -> i64 {
        self.world.days_left()
    }

    pub(crate) const fn borough(&self) -> Borough {
        self.world.borough()
    }

    pub(crate) const fn phase(&self) -> GamePhase {
        self.world.phase()
    }

    pub(crate) const fn cops(&self) -> i64 {
        self.world.cops()
    }

    pub(crate) const fn price(&self, drug: Drug) -> i64 {
        self.world.price(drug)
    }

    pub(crate) const fn max_loan(&self) -> i64 {
        self.player.max_loan()
    }

    pub(crate) const fn is_price_cheap(&self, drug: Drug) -> bool {
        self.world.is_cheap(drug)
    }

    pub(crate) const fn is_price_expensive(&self, drug: Drug) -> bool {
        self.world.is_expensive(drug)
    }

    pub(crate) fn total_carried(&self) -> i64 {
        self.player.total_carried()
    }

    pub(crate) const fn is_armed_player(&self) -> bool {
        self.player.is_armed()
    }

    pub(crate) const fn affordable_units_at(&self, price: i64) -> i64 {
        self.player.affordable_units_at(price)
    }

    pub(crate) const fn affordable_units(&self, drug: Drug) -> i64 {
        self.affordable_units_at(self.price(drug))
    }
}

#[allow(clippy::missing_const_for_fn)]
impl GameState {
    pub(crate) fn repay_debt(&mut self, amount: i64) -> Result<(), RuleError> {
        if amount <= 0 {
            return Err(RuleError::InvalidAmount);
        }
        if amount > self.cash() {
            return Err(RuleError::InsufficientCash);
        }
        if amount > self.debt() {
            return Err(RuleError::InsufficientDebt);
        }

        self.player.repay_debt(amount);
        Ok(())
    }

    pub(crate) fn borrow_from_shark(&mut self, amount: i64) -> Result<(), RuleError> {
        if amount <= 0 {
            return Err(RuleError::InvalidAmount);
        }
        if amount + self.debt() > self.max_loan() {
            return Err(RuleError::LoanLimitExceeded);
        }

        self.player.borrow_from_shark(amount);
        Ok(())
    }

    pub(crate) fn deposit_bank_funds(&mut self, amount: i64) -> Result<(), RuleError> {
        if amount <= 0 {
            return Err(RuleError::InvalidAmount);
        }
        if amount > self.cash() {
            return Err(RuleError::InsufficientCash);
        }

        self.player.deposit_bank_funds(amount);
        Ok(())
    }

    pub(crate) fn withdraw_bank_funds(&mut self, amount: i64) -> Result<(), RuleError> {
        if amount <= 0 {
            return Err(RuleError::InvalidAmount);
        }
        if amount > self.bank() {
            return Err(RuleError::InsufficientBankFunds);
        }

        self.player.withdraw_bank_funds(amount);
        Ok(())
    }

    pub(crate) fn stash_drug_units(&mut self, drug: Drug, amount: i64) -> Result<(), RuleError> {
        if amount <= 0 {
            return Err(RuleError::InvalidAmount);
        }
        if self.player.stash_drug_units(drug, amount) {
            Ok(())
        } else {
            Err(RuleError::InsufficientCarriedDrugs)
        }
    }

    pub(crate) fn take_stashed_drug_units(
        &mut self,
        drug: Drug,
        amount: i64,
    ) -> Result<(), RuleError> {
        if amount <= 0 {
            return Err(RuleError::InvalidAmount);
        }
        if amount > self.hold() {
            return Err(RuleError::InsufficientCarrySpace);
        }
        if self.player.take_stashed_drug_units(drug, amount) {
            Ok(())
        } else {
            Err(RuleError::InsufficientStashedDrugs)
        }
    }

    pub(crate) fn buy_drug_units(&mut self, drug: Drug, amount: i64) -> Result<(), RuleError> {
        if amount <= 0 {
            return Err(RuleError::InvalidAmount);
        }
        if amount > self.affordable_units(drug) {
            return Err(RuleError::InsufficientCash);
        }

        let price = self.price(drug);
        if self.player.buy_drug_units(drug, amount, price) {
            Ok(())
        } else {
            Err(RuleError::InsufficientCarrySpace)
        }
    }

    pub(crate) fn sell_drug_units(&mut self, drug: Drug, amount: i64) -> Result<(), RuleError> {
        if amount <= 0 {
            return Err(RuleError::InvalidAmount);
        }
        if !self.player.sell_drug_units(drug, amount, self.price(drug)) {
            return Err(RuleError::InsufficientCarriedDrugs);
        }
        Ok(())
    }

    pub(crate) fn buy_coat(&mut self, price: i64, extra_capacity: i64) -> Result<(), RuleError> {
        if price <= 0 || extra_capacity <= 0 {
            return Err(RuleError::InvalidAmount);
        }
        if price > self.cash() {
            return Err(RuleError::InsufficientCash);
        }

        self.player.buy_coat(price, extra_capacity);
        Ok(())
    }

    pub(crate) fn buy_handgun(&mut self, price: i64) -> Result<(), RuleError> {
        if price <= 0 {
            return Err(RuleError::InvalidAmount);
        }
        if self.has_shotgun() || self.guns() >= MAX_HANDGUNS {
            return Err(RuleError::GunLimitReached);
        }
        if price > self.cash() {
            return Err(RuleError::InsufficientCash);
        }

        self.player.buy_handgun(price);
        Ok(())
    }

    pub(crate) fn buy_shotgun(&mut self, price: i64) -> Result<(), RuleError> {
        if price <= 0 {
            return Err(RuleError::InvalidAmount);
        }
        if self.has_shotgun() {
            return Err(RuleError::AlreadyHasShotgun);
        }
        if price > self.cash() {
            return Err(RuleError::InsufficientCash);
        }

        self.player.buy_shotgun(price);
        Ok(())
    }

    pub(crate) fn pay_doctor(&mut self, price: i64) -> Result<(), RuleError> {
        if price <= 0 {
            return Err(RuleError::InvalidAmount);
        }
        if price > self.cash() {
            return Err(RuleError::InsufficientCash);
        }

        self.player.pay_doctor(price);
        Ok(())
    }

    pub(crate) fn add_cash(&mut self, amount: i64) {
        self.player.add_cash(amount);
    }

    pub(crate) fn apply_mugging_loss(&mut self) -> i64 {
        self.player.apply_mugging()
    }

    pub(crate) fn discard_carried_fraction(&mut self, drug: Drug, divisor: i64) -> i64 {
        self.player.discard_fraction_of_carried(drug, divisor)
    }

    pub(crate) fn discard_all_carried_fractions(&mut self, divisor: i64) -> i64 {
        self.player.discard_fraction_of_all_carried(divisor)
    }

    pub(crate) fn find_drug_amount(&mut self, drug: Drug, amount: i64) -> i64 {
        self.player.find_drugs(drug, amount)
    }

    pub(crate) fn add_damage_points(&mut self, amount: i64) -> i64 {
        self.player.add_damage(amount)
    }

    pub(crate) fn is_in_borough(&self, borough: Borough) -> bool {
        self.world.is_in_borough(borough)
    }

    pub(crate) fn set_phase(&mut self, phase: GamePhase) {
        self.world.set_phase(phase);
    }

    pub(crate) fn set_price(&mut self, drug: Drug, price: i64) {
        self.world.set_price(drug, price);
    }

    pub(crate) fn reset_market_highlights(&mut self) {
        self.world.reset_market_highlights();
    }

    pub(crate) fn mark_price_spike(&mut self, drug: Drug, multiplier: i64) {
        self.world.mark_price_spike(drug, multiplier);
    }

    pub(crate) fn mark_price_drop(&mut self, drug: Drug, divisor: i64) {
        self.world.mark_price_drop(drug, divisor);
    }

    pub(crate) fn should_offer_guns(&self) -> bool {
        self.is_in_borough(Borough::Ghetto) && !self.has_shotgun()
    }

    pub(crate) fn travel_to(&mut self, destination: Borough) {
        self.world.travel_to(destination);
    }

    pub(crate) fn advance_day(&mut self) -> DayAdvance {
        if self.world.advance_day() == DayAdvance::GameOver {
            return DayAdvance::GameOver;
        }

        self.roll_prices();
        self.player.accrue_daily_interest();
        DayAdvance::Continue
    }

    pub(crate) fn set_cops(&mut self, cops: i64) {
        self.world.set_cops(cops);
    }

    pub(crate) fn reduce_cops(&mut self, amount: i64) -> i64 {
        self.world.reduce_cops(amount)
    }
}
