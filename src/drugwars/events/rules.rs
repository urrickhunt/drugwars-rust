use rand::Rng;

use crate::balance::events::{
    BROWNIES_DIVISOR_MAX, BROWNIES_DIVISOR_MIN, COAT_EXTRA_CAPACITY_MAX, COAT_EXTRA_CAPACITY_MIN,
    COAT_PRICE_MAX, COAT_PRICE_MIN, FIND_DRUGS_AMOUNT_MAX, FIND_DRUGS_AMOUNT_MIN,
    HANDGUN_PRICE_MAX, HANDGUN_PRICE_MIN, POLICE_DOGS_DIVISOR_MAX, POLICE_DOGS_DIVISOR_MIN,
    SHOTGUN_PRICE_MAX, SHOTGUN_PRICE_MIN, STORY_EVENT_HOLD_THRESHOLD,
};
use crate::engine::RuleError;
use crate::model::{ALL_DRUGS, Drug, GameState, PRICE_EVENTS, PriceEffect, PriceEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DailyEvent {
    PriceChange(PriceChange),
    ParaquatOffer,
    Mugging(MuggingOutcome),
    CoatOffer(CoatOffer),
    Brownies(BrowniesOutcome),
    PoliceDogs(PoliceDogsOutcome),
    FindDrugs(FindDrugsOutcome),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PriceChange {
    pub drug: Drug,
    pub effect: PriceEffect,
    pub color: &'static str,
    pub message: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MuggingOutcome {
    pub shotgun_saved_you: bool,
    pub cash_lost: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoatOffer {
    pub price: i64,
    pub extra_capacity: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrowniesOutcome {
    pub dropped: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoliceDogsOutcome {
    pub divisor: i64,
    pub dropped_total: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FindDrugsOutcome {
    pub drug: Drug,
    pub found_amount: i64,
    pub taken_amount: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirearmKind {
    Ruger,
    ThirtyEightSpecial,
    SaturdayNightSpecial,
    Baretta,
    Mossberg590Shotgun,
}

impl FirearmKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ruger => "RUGER",
            Self::ThirtyEightSpecial => ".38 SPECIAL",
            Self::SaturdayNightSpecial => "SATURDAY NIGHT SPECIAL",
            Self::Baretta => "BARETTA",
            Self::Mossberg590Shotgun => "MOSSBERG 590 SHOTGUN",
        }
    }

    pub const fn is_shotgun(self) -> bool {
        matches!(self, Self::Mossberg590Shotgun)
    }

    fn roll_price<R: Rng + ?Sized>(self, rng: &mut R) -> i64 {
        match self {
            Self::Mossberg590Shotgun => rng.gen_range(SHOTGUN_PRICE_MIN..=SHOTGUN_PRICE_MAX),
            Self::Ruger | Self::ThirtyEightSpecial | Self::SaturdayNightSpecial | Self::Baretta => {
                rng.gen_range(HANDGUN_PRICE_MIN..=HANDGUN_PRICE_MAX)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GunSaleOffer {
    pub option_key: char,
    pub firearm: FirearmKind,
    pub price: i64,
}

impl GunSaleOffer {
    pub const fn name(self) -> &'static str {
        self.firearm.name()
    }

    pub const fn is_shotgun(self) -> bool {
        self.firearm.is_shotgun()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GunSaleMenu {
    offers: Vec<GunSaleOffer>,
}

impl GunSaleMenu {
    pub fn offers(&self) -> &[GunSaleOffer] {
        &self.offers
    }

    pub fn offer_for_key(&self, key: char) -> Option<GunSaleOffer> {
        self.offers
            .iter()
            .copied()
            .find(|offer| offer.option_key == key)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StoryEventKind {
    Paraquat,
    Mugged,
    CoatSale,
    Brownies,
    PoliceDogs,
    FindDrugs,
}

impl GameState {
    pub(crate) fn roll_daily_event(&mut self) -> DailyEvent {
        let mut rng = rand::thread_rng();
        let story_events = self.available_story_events();
        let pick = rng.gen_range(0..PRICE_EVENTS.len() + story_events.len());

        if pick < PRICE_EVENTS.len() {
            DailyEvent::PriceChange(self.apply_price_change(&PRICE_EVENTS[pick]))
        } else {
            self.roll_story_event(story_events[pick - PRICE_EVENTS.len()], &mut rng)
        }
    }

    pub(crate) fn accept_coat_offer(&mut self, offer: CoatOffer) -> Result<(), RuleError> {
        self.buy_coat(offer.price, offer.extra_capacity)
    }

    pub(crate) fn roll_gun_sale(&self) -> GunSaleMenu {
        let mut rng = rand::thread_rng();
        let offers = if self.guns() < 2 {
            [
                ('1', FirearmKind::Ruger),
                ('2', FirearmKind::ThirtyEightSpecial),
                ('3', FirearmKind::SaturdayNightSpecial),
                ('4', FirearmKind::Baretta),
                ('5', FirearmKind::Mossberg590Shotgun),
            ]
            .into_iter()
            .map(|(option_key, firearm)| GunSaleOffer {
                option_key,
                firearm,
                price: firearm.roll_price(&mut rng),
            })
            .collect()
        } else {
            vec![GunSaleOffer {
                option_key: '1',
                firearm: FirearmKind::Mossberg590Shotgun,
                price: FirearmKind::Mossberg590Shotgun.roll_price(&mut rng),
            }]
        };

        GunSaleMenu { offers }
    }

    fn available_story_events(&self) -> Vec<StoryEventKind> {
        let mut story_events = vec![
            StoryEventKind::Paraquat,
            StoryEventKind::Mugged,
            StoryEventKind::CoatSale,
        ];

        if self.carried_units(Drug::Weed) > 1 {
            story_events.push(StoryEventKind::Brownies);
        }
        if self.hold() < STORY_EVENT_HOLD_THRESHOLD {
            story_events.push(StoryEventKind::PoliceDogs);
        } else {
            story_events.push(StoryEventKind::FindDrugs);
        }

        story_events
    }

    fn roll_story_event<R: Rng + ?Sized>(
        &mut self,
        story_event: StoryEventKind,
        rng: &mut R,
    ) -> DailyEvent {
        match story_event {
            StoryEventKind::Paraquat => DailyEvent::ParaquatOffer,
            StoryEventKind::Mugged => DailyEvent::Mugging(self.resolve_mugging()),
            StoryEventKind::CoatSale => DailyEvent::CoatOffer(CoatOffer {
                price: rng.gen_range(COAT_PRICE_MIN..=COAT_PRICE_MAX),
                extra_capacity: rng.gen_range(COAT_EXTRA_CAPACITY_MIN..=COAT_EXTRA_CAPACITY_MAX),
            }),
            StoryEventKind::Brownies => DailyEvent::Brownies(
                self.resolve_brownies(rng.gen_range(BROWNIES_DIVISOR_MIN..=BROWNIES_DIVISOR_MAX)),
            ),
            StoryEventKind::PoliceDogs => DailyEvent::PoliceDogs(self.resolve_police_dogs(
                rng.gen_range(POLICE_DOGS_DIVISOR_MIN..=POLICE_DOGS_DIVISOR_MAX),
            )),
            StoryEventKind::FindDrugs => DailyEvent::FindDrugs(self.resolve_find_drugs(
                ALL_DRUGS[rng.gen_range(0..ALL_DRUGS.len())],
                rng.gen_range(FIND_DRUGS_AMOUNT_MIN..=FIND_DRUGS_AMOUNT_MAX),
            )),
        }
    }

    fn apply_price_change(&mut self, event: &PriceEvent) -> PriceChange {
        match event.effect {
            PriceEffect::Spike(multiplier) => {
                self.mark_price_spike(event.drug, multiplier);
            }
            PriceEffect::Drop(divisor) => {
                self.mark_price_drop(event.drug, divisor);
            }
        }

        PriceChange {
            drug: event.drug,
            effect: event.effect,
            color: event.color,
            message: event.message,
        }
    }

    fn resolve_brownies(&mut self, divisor: i64) -> BrowniesOutcome {
        BrowniesOutcome {
            dropped: self.discard_carried_fraction(Drug::Weed, divisor),
        }
    }

    fn resolve_mugging(&mut self) -> MuggingOutcome {
        if self.has_shotgun() {
            return MuggingOutcome {
                shotgun_saved_you: true,
                cash_lost: 0,
            };
        }

        MuggingOutcome {
            shotgun_saved_you: false,
            cash_lost: self.apply_mugging_loss(),
        }
    }

    fn resolve_police_dogs(&mut self, divisor: i64) -> PoliceDogsOutcome {
        PoliceDogsOutcome {
            divisor,
            dropped_total: self.discard_all_carried_fractions(divisor),
        }
    }

    fn resolve_find_drugs(&mut self, drug: Drug, amount: i64) -> FindDrugsOutcome {
        let taken_amount = self.find_drug_amount(drug, amount);
        FindDrugsOutcome {
            drug,
            found_amount: amount,
            taken_amount,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Drug, PRICE_EVENTS};
    use crate::test_support::game_builder;

    use super::{CoatOffer, FirearmKind, MuggingOutcome};
    use crate::model::PriceEffect;

    #[test]
    fn mugging_reflects_shotgun_protection() {
        let mut game = game_builder().build();
        let outcome = game.resolve_mugging();
        assert_eq!(
            outcome,
            MuggingOutcome {
                shotgun_saved_you: false,
                cash_lost: 400,
            }
        );

        let mut armed_game = game_builder().with_shotgun().build();
        let outcome = armed_game.resolve_mugging();
        assert_eq!(
            outcome,
            MuggingOutcome {
                shotgun_saved_you: true,
                cash_lost: 0,
            }
        );
    }

    #[test]
    fn price_changes_update_market_state() {
        let mut game = game_builder().with_price(Drug::Cocaine, 10_000).build();

        let change = game.apply_price_change(&PRICE_EVENTS[0]);
        assert_eq!(change.effect, PriceEffect::Spike(6));
        assert!(game.is_price_expensive(Drug::Cocaine));
        assert_eq!(game.price(Drug::Cocaine), 60_000);
    }

    #[test]
    fn coat_and_paraquat_offers_are_typed() {
        let mut game = game_builder().build();
        game.accept_coat_offer(CoatOffer {
            price: 300,
            extra_capacity: 50,
        })
        .expect("coat offer should be accepted");
        assert_eq!(game.cash(), 1_700);
        assert_eq!(game.hold(), 150);
    }

    #[test]
    fn gun_sale_menu_depends_on_current_arsenal() {
        let game = game_builder().build();
        let sale = game.roll_gun_sale();
        assert_eq!(sale.offers().len(), 5);
        assert_eq!(
            sale.offer_for_key('5').map(|offer| offer.firearm),
            Some(FirearmKind::Mossberg590Shotgun)
        );

        let armed_game = game_builder().with_guns(2).build();
        let sale = armed_game.roll_gun_sale();
        assert_eq!(sale.offers().len(), 1);
        assert_eq!(sale.offers()[0].firearm, FirearmKind::Mossberg590Shotgun);
    }

    #[test]
    fn find_drugs_event_is_capped_by_remaining_capacity() {
        let mut game = game_builder().with_carried_drugs(Drug::Cocaine, 95).build();

        let outcome = game.resolve_find_drugs(Drug::Weed, 10);

        assert_eq!(outcome.drug, Drug::Weed);
        assert_eq!(outcome.found_amount, 10);
        assert_eq!(outcome.taken_amount, 5);
        assert_eq!(game.carried_units(Drug::Weed), 5);
        assert_eq!(game.hold(), 0);
    }
}
