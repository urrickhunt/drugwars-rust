use crate::model::GameState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleError {
    InvalidAmount,
    InsufficientCash,
    InsufficientDebt,
    InsufficientBankFunds,
    LoanLimitExceeded,
    InsufficientCarrySpace,
    InsufficientCarriedDrugs,
    InsufficientStashedDrugs,
    GunLimitReached,
    AlreadyHasShotgun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreSummary {
    pub total_money: i64,
    pub score: i64,
    pub dealer_rank: &'static str,
    pub comment: &'static str,
}

const fn dealer_rank(score: i64) -> (&'static str, &'static str) {
    match score {
        100 => ("GANGSTA MOTHERFUCKER", "YOU ARE MY HERO"),
        99 => ("HUSTLER FUCKER", "YOU THA DOPE MAN"),
        98 => ("PABLO ESCOBAR", "YOU ARE A GOD"),
        76..=97 => ("KINGPIN", "GOD DAMN"),
        51..=75 => ("RUN THE TOWN", "PRETTY GOOD"),
        31..=50 => ("OWN THE BLOCK", "NOT BAD"),
        _ => ("SMALL TIME PUSHA", "WEAK"),
    }
}

fn score_for_total_money(total_money: i64) -> i64 {
    if total_money > 50_000_000 {
        100
    } else if total_money >= 25_000_000 {
        99
    } else if total_money >= 10_000_000 {
        98
    } else {
        (total_money * 100 / 10_000_000).clamp(0, 97)
    }
}

impl GameState {
    pub(crate) const fn total_money(&self) -> i64 {
        self.bank() + self.cash() - self.debt()
    }

    pub(crate) fn score_summary(&self) -> ScoreSummary {
        let total_money = self.total_money();
        let score = score_for_total_money(total_money);
        let (dealer_rank, comment) = dealer_rank(score);

        ScoreSummary {
            total_money,
            score,
            dealer_rank,
            comment,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Borough, Drug, GamePhase};
    use crate::test_support::game_builder;

    use super::{RuleError, ScoreSummary};

    #[test]
    fn finance_operations_enforce_limits() {
        let mut game = game_builder().build();

        assert_eq!(game.repay_debt(6_000), Err(RuleError::InsufficientCash));
        assert_eq!(
            game.borrow_from_shark(5_000),
            Err(RuleError::LoanLimitExceeded)
        );

        game.repay_debt(500).expect("repayment should succeed");
        assert_eq!(game.cash(), 1_500);
        assert_eq!(game.debt(), 5_000);

        game.borrow_from_shark(1_000)
            .expect("borrow should succeed");
        assert_eq!(game.cash(), 2_500);
        assert_eq!(game.debt(), 6_000);

        game.deposit_bank_funds(500)
            .expect("deposit should succeed");
        assert_eq!(game.bank(), 500);
        assert_eq!(game.cash(), 2_000);

        game.withdraw_bank_funds(200)
            .expect("withdrawal should succeed");
        assert_eq!(game.bank(), 300);
        assert_eq!(game.cash(), 2_200);
    }

    #[test]
    fn drug_transactions_go_through_engine_methods() {
        let mut game = game_builder().with_price(Drug::Weed, 100).build();

        game.buy_drug_units(Drug::Weed, 10)
            .expect("buy should succeed");
        assert_eq!(game.carried_units(Drug::Weed), 10);
        assert_eq!(game.hold(), 90);
        assert_eq!(game.cash(), 1_000);

        game.stash_drug_units(Drug::Weed, 4)
            .expect("stash should succeed");
        assert_eq!(game.carried_units(Drug::Weed), 6);
        assert_eq!(game.stashed_units(Drug::Weed), 4);
        assert_eq!(game.hold(), 94);

        game.take_stashed_drug_units(Drug::Weed, 2)
            .expect("retrieve should succeed");
        assert_eq!(game.carried_units(Drug::Weed), 8);
        assert_eq!(game.stashed_units(Drug::Weed), 2);
        assert_eq!(game.hold(), 92);

        game.sell_drug_units(Drug::Weed, 3)
            .expect("sell should succeed");
        assert_eq!(game.carried_units(Drug::Weed), 5);
        assert_eq!(game.hold(), 95);
        assert_eq!(game.cash(), 1_300);
    }

    #[test]
    fn buy_drug_units_reports_carry_space_from_inventory_layer() {
        let mut game = game_builder()
            .with_price(Drug::Weed, 100)
            .with_carried_drugs(Drug::Cocaine, 99)
            .build();

        assert_eq!(
            game.buy_drug_units(Drug::Weed, 2),
            Err(RuleError::InsufficientCarrySpace)
        );
    }

    #[test]
    fn take_stashed_drug_units_reports_carry_space_before_inventory_mismatch() {
        let mut game = game_builder()
            .with_carried_drugs(Drug::Cocaine, 99)
            .with_stashed_drugs(Drug::Weed, 5)
            .build();

        assert_eq!(
            game.take_stashed_drug_units(Drug::Weed, 2),
            Err(RuleError::InsufficientCarrySpace)
        );
        assert_eq!(game.stashed_units(Drug::Weed), 5);
        assert_eq!(game.carried_units(Drug::Weed), 0);
    }

    #[test]
    fn travel_day_advance_and_score_summary_are_pure() {
        let mut game = game_builder()
            .with_cash(20_000_000)
            .with_bank(15_000_000)
            .with_debt(1_000_000)
            .build();

        game.travel_to(Borough::Bronx);
        assert_eq!(game.borough(), Borough::Bronx);
        assert_eq!(game.phase(), GamePhase::BronxDo);

        let outcome = game.advance_day();
        assert_eq!(outcome, crate::model::DayAdvance::Continue);
        assert_eq!(game.day(), 1);
        assert_eq!(game.days_left(), 30);

        let ScoreSummary {
            total_money,
            score,
            dealer_rank,
            comment,
        } = game.score_summary();
        assert_eq!(total_money, 34_650_000);
        assert_eq!(score, 99);
        assert_eq!(dealer_rank, "HUSTLER FUCKER");
        assert_eq!(comment, "YOU THA DOPE MAN");
    }

    #[test]
    fn weapon_and_doctor_purchases_use_rule_checks() {
        let mut game = game_builder().build();

        game.buy_handgun(300).expect("handgun should succeed");
        assert_eq!(game.guns(), 1);
        assert_eq!(game.cash(), 1_700);

        game.buy_shotgun(1_500).expect("shotgun should succeed");
        assert!(game.has_shotgun());
        assert_eq!(game.guns(), 0);
        assert_eq!(game.cash(), 200);

        let mut game = game_builder().with_cash(200).with_damage(20).build();
        assert_eq!(game.pay_doctor(300), Err(RuleError::InsufficientCash));
        game.add_cash(200);
        game.pay_doctor(300).expect("doctor should succeed");
        assert_eq!(game.damage(), 0);
        assert_eq!(game.cash(), 100);
    }
}
