use std::sync::{Arc, Mutex};

use crate::audio::MusicPlayer;
use crate::model::{Drug, GamePhase, GameState, PlayerState, ScreenState, WorldState};

pub(crate) fn game_builder() -> GameBuilder {
    GameBuilder::new()
}

pub(crate) struct GameBuilder {
    game: GameState,
}

impl GameBuilder {
    fn new() -> Self {
        Self {
            game: GameState {
                player: PlayerState::new(),
                world: WorldState::new(),
                screen: ScreenState::new(),
                music: Arc::new(Mutex::new(MusicPlayer::new(String::new()))),
            },
        }
    }

    pub(crate) fn with_cash(mut self, cash: i64) -> Self {
        self.game.player.cash = cash;
        self
    }

    pub(crate) fn with_bank(mut self, bank: i64) -> Self {
        self.game.player.bank = bank;
        self
    }

    pub(crate) fn with_debt(mut self, debt: i64) -> Self {
        self.game.player.debt = debt;
        self
    }

    pub(crate) fn with_damage(mut self, damage: i64) -> Self {
        self.game.player.damage = damage;
        self
    }

    pub(crate) fn with_guns(mut self, guns: i64) -> Self {
        self.game.player.guns = guns;
        self.game.player.has_shotgun = false;
        self
    }

    pub(crate) fn with_shotgun(mut self) -> Self {
        self.game.player.guns = 0;
        self.game.player.has_shotgun = true;
        self
    }

    pub(crate) fn with_cops(mut self, cops: i64) -> Self {
        self.game.world.set_cops(cops);
        self
    }

    pub(crate) fn with_phase(mut self, phase: GamePhase) -> Self {
        self.game.world.set_phase(phase);
        self
    }

    pub(crate) fn with_price(mut self, drug: Drug, price: i64) -> Self {
        self.game.world.set_price(drug, price);
        self
    }

    pub(crate) fn with_price_spike(mut self, drug: Drug, multiplier: i64) -> Self {
        self.game.world.mark_price_spike(drug, multiplier);
        self
    }

    pub(crate) fn with_carried_drugs(mut self, drug: Drug, amount: i64) -> Self {
        self.game.player.find_drugs(drug, amount);
        self
    }

    pub(crate) fn with_stashed_drugs(mut self, drug: Drug, amount: i64) -> Self {
        self.game.player.inventory.stash[drug] = amount;
        self
    }

    pub(crate) fn build(self) -> GameState {
        self.game
    }
}
