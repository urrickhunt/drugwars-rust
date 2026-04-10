use rand::Rng;

use crate::balance::combat::{
    BASE_COPS_IN_CHASE, CHASE_ROLL_MAX, DEATH_DAMAGE_THRESHOLD, DOCTOR_OFFER_DAMAGE_THRESHOLD,
    DOCTOR_PRICE_MULTIPLIER_MAX, DOCTOR_PRICE_MULTIPLIER_MIN, FIGHT_CHANCE_TO_COPS_DIVISOR,
    MAX_RETURN_FIRE_DAMAGE, MIN_FIGHT_CHANCE, OFFICER_CASH_REWARD_MAX, OFFICER_CASH_REWARD_MIN,
    RETURN_FIRE_ATTACK_MAX, RETURN_FIRE_ATTACK_MIN, RETURN_FIRE_DEFENSE_MAX,
    RETURN_FIRE_DEFENSE_MIN, SHOTGUN_ACCURACY_ROLL_MAX, SHOTGUN_ACCURACY_ROLL_MIN,
    SHOTGUN_BASE_KILLS, SHOTGUN_BONUS_ROLL_MAX, SHOTGUN_BONUS_ROLL_MIN,
};
use crate::model::GameState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatAction {
    Run,
    Fight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FightOutcome {
    Escaped,
    Killed,
    Dead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatStart {
    pub cops: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatRound {
    pub resolution: CombatResolution,
    pub outcome: Option<FightOutcome>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatResolution {
    Escaped,
    RanIntoReturnFire(FireResult),
    FiredAndMissed(FireResult),
    KilledAll { reward: i64 },
    FiredAndHit { kills: i64, return_fire: FireResult },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FireResult {
    Missed,
    Hit { damage: i64, total_damage: i64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoctorOffer {
    pub price: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoctorTreatment {
    Healed,
    CouldNotAfford,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ReturnFireRoll {
    attack: i64,
    defense: i64,
}

impl GameState {
    pub(crate) fn start_fight(&mut self) -> Option<CombatStart> {
        let mut rng = rand::thread_rng();
        self.start_fight_from_roll(rng.gen_range(1..=CHASE_ROLL_MAX))
    }

    pub(crate) const fn is_armed(&self) -> bool {
        self.is_armed_player()
    }

    pub(crate) fn resolve_combat_action(&mut self, action: CombatAction) -> CombatRound {
        let mut rng = rand::thread_rng();
        match action {
            CombatAction::Run => self
                .resolve_escape_round(rng.gen_range(1..=2) == 1, Self::roll_return_fire(&mut rng)),
            CombatAction::Fight => self.resolve_attack_round(
                self.roll_accuracy(&mut rng),
                self.roll_kills(&mut rng),
                self.roll_shotgun_bonus(&mut rng),
                rng.gen_range(OFFICER_CASH_REWARD_MIN..=OFFICER_CASH_REWARD_MAX),
                Self::roll_return_fire(&mut rng),
            ),
        }
    }

    pub(crate) fn roll_doctor_offer(&self) -> Option<DoctorOffer> {
        if self.damage() <= DOCTOR_OFFER_DAMAGE_THRESHOLD {
            return None;
        }

        let mut rng = rand::thread_rng();
        Some(DoctorOffer {
            price: self.damage()
                * rng.gen_range(DOCTOR_PRICE_MULTIPLIER_MIN..=DOCTOR_PRICE_MULTIPLIER_MAX)
                / DOCTOR_OFFER_DAMAGE_THRESHOLD,
        })
    }

    pub(crate) fn accept_doctor_offer(&mut self, offer: DoctorOffer) -> DoctorTreatment {
        match self.pay_doctor(offer.price) {
            Ok(()) => DoctorTreatment::Healed,
            Err(_) => DoctorTreatment::CouldNotAfford,
        }
    }

    fn start_fight_from_roll(&mut self, chase_roll: i64) -> Option<CombatStart> {
        let fight_chance = chase_roll * self.total_carried() / CHASE_ROLL_MAX;
        if fight_chance < MIN_FIGHT_CHANCE {
            return None;
        }

        self.set_cops(fight_chance / FIGHT_CHANCE_TO_COPS_DIVISOR + BASE_COPS_IN_CHASE);
        Some(CombatStart { cops: self.cops() })
    }

    fn resolve_escape_round(
        &mut self,
        escaped: bool,
        return_fire_roll: ReturnFireRoll,
    ) -> CombatRound {
        if escaped {
            return CombatRound {
                resolution: CombatResolution::Escaped,
                outcome: Some(FightOutcome::Escaped),
            };
        }

        let (return_fire, outcome) = self.apply_return_fire(return_fire_roll);
        CombatRound {
            resolution: CombatResolution::RanIntoReturnFire(return_fire),
            outcome,
        }
    }

    fn resolve_attack_round(
        &mut self,
        accuracy_roll: i64,
        kill_roll: i64,
        shotgun_bonus: i64,
        reward: i64,
        return_fire_roll: ReturnFireRoll,
    ) -> CombatRound {
        if self.attack_missed(accuracy_roll) {
            let (return_fire, outcome) = self.apply_return_fire(return_fire_roll);
            return CombatRound {
                resolution: CombatResolution::FiredAndMissed(return_fire),
                outcome,
            };
        }

        let kills = self.kills_from_roll(kill_roll, shotgun_bonus);
        if self.reduce_cops(kills) <= 0 {
            self.add_cash(reward);
            return CombatRound {
                resolution: CombatResolution::KilledAll { reward },
                outcome: Some(FightOutcome::Killed),
            };
        }

        let (return_fire, outcome) = self.apply_return_fire(return_fire_roll);
        CombatRound {
            resolution: CombatResolution::FiredAndHit { kills, return_fire },
            outcome,
        }
    }

    fn roll_accuracy<R: Rng + ?Sized>(&self, rng: &mut R) -> i64 {
        if self.has_shotgun() {
            rng.gen_range(SHOTGUN_ACCURACY_ROLL_MIN..=SHOTGUN_ACCURACY_ROLL_MAX)
        } else {
            rng.gen_range(0..=self.guns() * 2)
        }
    }

    fn roll_kills<R: Rng + ?Sized>(&self, rng: &mut R) -> i64 {
        if self.has_shotgun() || self.guns() <= 0 {
            0
        } else {
            rng.gen_range(1..=self.guns())
        }
    }

    fn roll_shotgun_bonus<R: Rng + ?Sized>(&self, rng: &mut R) -> i64 {
        if self.has_shotgun() {
            rng.gen_range(SHOTGUN_BONUS_ROLL_MIN..=SHOTGUN_BONUS_ROLL_MAX)
                + rng.gen_range(SHOTGUN_BONUS_ROLL_MIN..=SHOTGUN_BONUS_ROLL_MAX)
        } else {
            0
        }
    }

    fn roll_return_fire<R: Rng + ?Sized>(rng: &mut R) -> ReturnFireRoll {
        ReturnFireRoll {
            attack: rng.gen_range(RETURN_FIRE_ATTACK_MIN..=RETURN_FIRE_ATTACK_MAX),
            defense: rng.gen_range(RETURN_FIRE_DEFENSE_MIN..=RETURN_FIRE_DEFENSE_MAX),
        }
    }

    const fn attack_missed(&self, accuracy_roll: i64) -> bool {
        if self.has_shotgun() {
            accuracy_roll == 1
        } else {
            accuracy_roll == 0
        }
    }

    fn kills_from_roll(&self, kill_roll: i64, shotgun_bonus: i64) -> i64 {
        if self.has_shotgun() {
            (SHOTGUN_BASE_KILLS + shotgun_bonus).min(self.cops())
        } else {
            kill_roll.min(self.guns()).min(self.cops())
        }
    }

    fn apply_return_fire(&mut self, roll: ReturnFireRoll) -> (FireResult, Option<FightOutcome>) {
        let damage = (roll.attack * self.cops() - roll.defense).clamp(0, MAX_RETURN_FIRE_DAMAGE);
        if damage == 0 {
            return (FireResult::Missed, None);
        }

        let total_damage = self.add_damage_points(damage);
        let fire = FireResult::Hit {
            damage,
            total_damage,
        };
        let outcome = if total_damage >= DEATH_DAMAGE_THRESHOLD {
            Some(FightOutcome::Dead)
        } else {
            None
        };
        (fire, outcome)
    }
}

#[cfg(test)]
mod tests {
    use rand::thread_rng;

    use crate::test_support::game_builder;

    use crate::model::Drug;

    use super::{CombatResolution, DoctorOffer, DoctorTreatment, FightOutcome, ReturnFireRoll};

    #[test]
    fn fight_trigger_depends_on_carried_inventory() {
        let mut game = game_builder().with_carried_drugs(Drug::Weed, 20).build();

        let start = game
            .start_fight_from_roll(50)
            .expect("fight should start with enough inventory");
        assert_eq!(start.cops, 3);
        assert_eq!(game.cops(), 3);
    }

    #[test]
    fn attack_round_can_finish_the_fight_and_award_cash() {
        let mut game = game_builder().with_guns(1).with_cops(1).build();

        let round = game.resolve_attack_round(
            1,
            1,
            0,
            750,
            ReturnFireRoll {
                attack: 0,
                defense: 18,
            },
        );

        assert_eq!(round.outcome, Some(FightOutcome::Killed));
        assert_eq!(
            round.resolution,
            CombatResolution::KilledAll { reward: 750 }
        );
        assert_eq!(game.cash(), 2_750);
    }

    #[test]
    fn doctor_offer_and_treatment_are_engine_driven() {
        let mut game = game_builder().with_damage(24).build();

        assert!(game.roll_doctor_offer().is_some());

        let treatment = game.accept_doctor_offer(DoctorOffer { price: 3_000 });
        assert_eq!(treatment, DoctorTreatment::CouldNotAfford);

        let mut game = game_builder().with_damage(24).with_cash(5_000).build();
        let treatment = game.accept_doctor_offer(DoctorOffer { price: 3_000 });
        assert_eq!(treatment, DoctorTreatment::Healed);
        assert_eq!(game.damage(), 0);
    }

    #[test]
    fn roll_kills_is_safe_when_called_unarmed() {
        let game = game_builder().build();
        let mut rng = thread_rng();

        assert_eq!(game.roll_kills(&mut rng), 0);
    }
}
