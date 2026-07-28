use crate::balance::player::{
    BANK_INTEREST_DENOMINATOR, BANK_INTEREST_NUMERATOR, DEBT_INTEREST_DENOMINATOR,
    DEBT_INTEREST_NUMERATOR, MUGGING_CASH_KEEP_DENOMINATOR, MUGGING_CASH_KEEP_NUMERATOR,
    STARTING_BANK, STARTING_CASH, STARTING_DAMAGE, STARTING_DEBT, STARTING_GUNS,
    STARTING_HAS_SHOTGUN, STARTING_HOLD, STARTING_MAX_LOAN,
};

use super::{ALL_DRUGS, Drug, DrugMap};

#[derive(Debug, Clone, Copy)]
pub struct Inventory {
    pub(crate) hold: i64,
    pub(crate) stash: DrugMap<i64>,
    pub(crate) trench_coat: DrugMap<i64>,
}

impl Inventory {
    pub(crate) const fn new() -> Self {
        Self {
            hold: STARTING_HOLD,
            stash: DrugMap::zeros(),
            trench_coat: DrugMap::zeros(),
        }
    }

    pub(crate) const fn hold(&self) -> i64 {
        self.hold
    }

    pub(crate) const fn stashed(&self, drug: Drug) -> i64 {
        self.stash.get(drug)
    }

    pub(crate) const fn carried(&self, drug: Drug) -> i64 {
        self.trench_coat.get(drug)
    }

    pub(crate) fn total_carried(&self) -> i64 {
        self.trench_coat.total()
    }

    pub(crate) fn stash_from_coat(&mut self, drug: Drug, amount: i64) -> bool {
        if amount <= 0 || amount > self.trench_coat[drug] {
            return false;
        }

        self.trench_coat[drug] -= amount;
        self.stash[drug] += amount;
        self.hold += amount;
        true
    }

    pub(crate) fn take_from_stash(&mut self, drug: Drug, amount: i64) -> bool {
        if amount <= 0 || amount > self.stash[drug] {
            return false;
        }

        self.stash[drug] -= amount;
        self.trench_coat[drug] += amount;
        self.hold -= amount;
        true
    }

    pub(crate) fn buy(&mut self, drug: Drug, amount: i64) -> bool {
        if amount <= 0 || amount > self.hold {
            return false;
        }

        self.trench_coat[drug] += amount;
        self.hold -= amount;
        true
    }

    pub(crate) fn sell(&mut self, drug: Drug, amount: i64) -> bool {
        if amount <= 0 || amount > self.trench_coat[drug] {
            return false;
        }

        self.trench_coat[drug] -= amount;
        self.hold += amount;
        true
    }

    pub(crate) fn discard_fraction_of_carried(&mut self, drug: Drug, divisor: i64) -> i64 {
        let dropped = self.trench_coat[drug] / divisor;
        self.trench_coat[drug] -= dropped;
        self.hold += dropped;
        dropped
    }

    pub(crate) fn find_drugs(&mut self, drug: Drug, amount: i64) -> i64 {
        if amount <= 0 {
            return 0;
        }

        let taken = amount.min(self.hold);
        self.trench_coat[drug] += taken;
        self.hold -= taken;
        taken
    }

    pub(crate) const fn expand_capacity(&mut self, amount: i64) {
        self.hold += amount;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerState {
    pub(crate) cash: i64,
    pub(crate) guns: i64,
    pub(crate) has_shotgun: bool,
    pub(crate) bank: i64,
    pub(crate) debt: i64,
    pub(crate) max_loan: i64,
    pub(crate) inventory: Inventory,
    pub(crate) damage: i64,
}

impl PlayerState {
    pub(crate) const fn new() -> Self {
        Self {
            cash: STARTING_CASH,
            guns: STARTING_GUNS,
            has_shotgun: STARTING_HAS_SHOTGUN,
            bank: STARTING_BANK,
            debt: STARTING_DEBT,
            max_loan: STARTING_MAX_LOAN,
            inventory: Inventory::new(),
            damage: STARTING_DAMAGE,
        }
    }

    pub(crate) const fn cash(&self) -> i64 {
        self.cash
    }

    pub(crate) const fn bank(&self) -> i64 {
        self.bank
    }

    pub(crate) const fn debt(&self) -> i64 {
        self.debt
    }

    pub(crate) const fn max_loan(&self) -> i64 {
        self.max_loan
    }

    pub(crate) const fn guns(&self) -> i64 {
        self.guns
    }

    pub(crate) const fn has_shotgun(&self) -> bool {
        self.has_shotgun
    }

    pub(crate) const fn damage(&self) -> i64 {
        self.damage
    }

    pub(crate) const fn hold(&self) -> i64 {
        self.inventory.hold()
    }

    pub(crate) const fn carried_units(&self, drug: Drug) -> i64 {
        self.inventory.carried(drug)
    }

    pub(crate) const fn stashed_units(&self, drug: Drug) -> i64 {
        self.inventory.stashed(drug)
    }

    pub(crate) fn total_carried(&self) -> i64 {
        self.inventory.total_carried()
    }

    pub(crate) const fn is_armed(&self) -> bool {
        self.guns > 0 || self.has_shotgun
    }

    pub(crate) const fn affordable_units_at(&self, price: i64) -> i64 {
        if price <= 0 { 0 } else { self.cash / price }
    }

    pub(crate) const fn repay_debt(&mut self, amount: i64) {
        self.cash -= amount;
        self.debt -= amount;
    }

    pub(crate) const fn borrow_from_shark(&mut self, amount: i64) {
        self.debt += amount;
        self.cash += amount;
    }

    pub(crate) const fn deposit_bank_funds(&mut self, amount: i64) {
        self.cash -= amount;
        self.bank += amount;
    }

    pub(crate) const fn withdraw_bank_funds(&mut self, amount: i64) {
        self.bank -= amount;
        self.cash += amount;
    }

    pub(crate) fn stash_drug_units(&mut self, drug: Drug, amount: i64) -> bool {
        self.inventory.stash_from_coat(drug, amount)
    }

    pub(crate) fn take_stashed_drug_units(&mut self, drug: Drug, amount: i64) -> bool {
        self.inventory.take_from_stash(drug, amount)
    }

    pub(crate) fn buy_drug_units(&mut self, drug: Drug, amount: i64, price: i64) -> bool {
        if !self.inventory.buy(drug, amount) {
            return false;
        }

        self.cash -= amount * price;
        true
    }

    pub(crate) fn sell_drug_units(&mut self, drug: Drug, amount: i64, price: i64) -> bool {
        if !self.inventory.sell(drug, amount) {
            return false;
        }

        self.cash += amount * price;
        true
    }

    pub(crate) const fn buy_coat(&mut self, price: i64, extra_capacity: i64) {
        self.cash -= price;
        self.inventory.expand_capacity(extra_capacity);
    }

    pub(crate) const fn buy_handgun(&mut self, price: i64) {
        self.cash -= price;
        self.guns += 1;
    }

    pub(crate) const fn buy_shotgun(&mut self, price: i64) {
        self.cash -= price;
        self.guns = 0;
        self.has_shotgun = true;
    }

    pub(crate) const fn pay_doctor(&mut self, price: i64) {
        self.cash -= price;
        self.damage = 0;
    }

    pub(crate) const fn add_cash(&mut self, amount: i64) {
        self.cash += amount;
    }

    pub(crate) const fn apply_mugging(&mut self) -> i64 {
        let cash_before = self.cash;
        self.cash = self.cash * MUGGING_CASH_KEEP_NUMERATOR / MUGGING_CASH_KEEP_DENOMINATOR;
        cash_before - self.cash
    }

    pub(crate) fn discard_fraction_of_carried(&mut self, drug: Drug, divisor: i64) -> i64 {
        self.inventory.discard_fraction_of_carried(drug, divisor)
    }

    pub(crate) fn discard_fraction_of_all_carried(&mut self, divisor: i64) -> i64 {
        ALL_DRUGS
            .into_iter()
            .map(|drug| self.inventory.discard_fraction_of_carried(drug, divisor))
            .sum()
    }

    pub(crate) fn find_drugs(&mut self, drug: Drug, amount: i64) -> i64 {
        self.inventory.find_drugs(drug, amount)
    }

    pub(crate) const fn add_damage(&mut self, amount: i64) -> i64 {
        self.damage += amount;
        self.damage
    }

    pub(crate) const fn accrue_daily_interest(&mut self) {
        self.debt = self.debt * DEBT_INTEREST_NUMERATOR / DEBT_INTEREST_DENOMINATOR;
        self.bank = self.bank * BANK_INTEREST_NUMERATOR / BANK_INTEREST_DENOMINATOR;
    }
}
