use crate::model::{Drug, DRUG_COUNT};

#[derive(Debug, Clone, Copy)]
pub struct DrugPriceRange {
    pub drug: Drug,
    pub min_price: i64,
    pub max_price: i64,
}

impl DrugPriceRange {
    const fn new(drug: Drug, min_price: i64, max_price: i64) -> Self {
        Self {
            drug,
            min_price,
            max_price,
        }
    }

    #[cfg(test)]
    pub(crate) const fn contains(self, price: i64) -> bool {
        price >= self.min_price && price <= self.max_price
    }
}

pub const DRUG_PRICE_RANGES: [DrugPriceRange; DRUG_COUNT] = [
    DrugPriceRange::new(Drug::Cocaine, 15_000, 30_000),
    DrugPriceRange::new(Drug::Heroin, 5_000, 14_000),
    DrugPriceRange::new(Drug::Acid, 1_000, 4_500),
    DrugPriceRange::new(Drug::Weed, 300, 900),
    DrugPriceRange::new(Drug::Speed, 70, 250),
    DrugPriceRange::new(Drug::Ludes, 10, 60),
];

pub mod player {
    pub const CAMPAIGN_DAYS: i64 = 31;
    pub const STARTING_HOLD: i64 = 100;
    pub const STARTING_CASH: i64 = 2_000;
    pub const STARTING_GUNS: i64 = 0;
    pub const MAX_HANDGUNS: i64 = 2;
    pub const STARTING_HAS_SHOTGUN: bool = false;
    pub const STARTING_BANK: i64 = 0;
    pub const STARTING_DEBT: i64 = 5_500;
    pub const STARTING_MAX_LOAN: i64 = 9_450;
    pub const STARTING_DAMAGE: i64 = 0;

    pub const MUGGING_CASH_KEEP_NUMERATOR: i64 = 4;
    pub const MUGGING_CASH_KEEP_DENOMINATOR: i64 = 5;

    pub const DEBT_INTEREST_NUMERATOR: i64 = 110;
    pub const DEBT_INTEREST_DENOMINATOR: i64 = 100;
    pub const BANK_INTEREST_NUMERATOR: i64 = 105;
    pub const BANK_INTEREST_DENOMINATOR: i64 = 100;
}

pub mod combat {
    pub const CHASE_ROLL_MAX: i64 = 100;
    pub const MIN_FIGHT_CHANCE: i64 = 10;
    pub const FIGHT_CHANCE_TO_COPS_DIVISOR: i64 = 9;
    pub const BASE_COPS_IN_CHASE: i64 = 2;

    pub const OFFICER_CASH_REWARD_MIN: i64 = 200;
    pub const OFFICER_CASH_REWARD_MAX: i64 = 1_000;
    pub const DOCTOR_PRICE_MULTIPLIER_MIN: i64 = 200;
    pub const DOCTOR_PRICE_MULTIPLIER_MAX: i64 = 1_000;
    pub const DOCTOR_OFFER_DAMAGE_THRESHOLD: i64 = 10;

    pub const SHOTGUN_ACCURACY_ROLL_MIN: i64 = 1;
    pub const SHOTGUN_ACCURACY_ROLL_MAX: i64 = 10;
    pub const RETURN_FIRE_ATTACK_MIN: i64 = 0;
    pub const RETURN_FIRE_ATTACK_MAX: i64 = 3;
    pub const RETURN_FIRE_DEFENSE_MIN: i64 = 2;
    pub const RETURN_FIRE_DEFENSE_MAX: i64 = 18;
    pub const MAX_RETURN_FIRE_DAMAGE: i64 = 9;
    pub const DEATH_DAMAGE_THRESHOLD: i64 = 50;
    pub const SHOTGUN_BASE_KILLS: i64 = 3;
    pub const SHOTGUN_BONUS_ROLL_MIN: i64 = 0;
    pub const SHOTGUN_BONUS_ROLL_MAX: i64 = 1;
}

pub mod events {
    pub const STORY_EVENT_HOLD_THRESHOLD: i64 = 32;

    pub const COAT_PRICE_MIN: i64 = 150;
    pub const COAT_PRICE_MAX: i64 = 400;
    pub const COAT_EXTRA_CAPACITY_MIN: i64 = 32;
    pub const COAT_EXTRA_CAPACITY_MAX: i64 = 64;

    pub const BROWNIES_DIVISOR_MIN: i64 = 2;
    pub const BROWNIES_DIVISOR_MAX: i64 = 5;
    pub const POLICE_DOGS_DIVISOR_MIN: i64 = 2;
    pub const POLICE_DOGS_DIVISOR_MAX: i64 = 5;
    pub const FIND_DRUGS_AMOUNT_MIN: i64 = 1;
    pub const FIND_DRUGS_AMOUNT_MAX: i64 = 32;

    pub const HANDGUN_PRICE_MIN: i64 = 249;
    pub const HANDGUN_PRICE_MAX: i64 = 469;
    pub const SHOTGUN_PRICE_MIN: i64 = 1_499;
    pub const SHOTGUN_PRICE_MAX: i64 = 2_999;
}
