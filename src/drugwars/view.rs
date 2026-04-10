use crate::audio::MusicState;
use crate::model::{Borough, Drug, GameState, ALL_DRUGS, DRUG_COUNT};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArsenalView {
    Shotgun,
    Guns(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudView {
    pub day: i64,
    pub days_left: i64,
    pub borough: Borough,
    pub hold: i64,
    pub bank: i64,
    pub debt: i64,
    pub cash: i64,
    pub arsenal: ArsenalView,
    pub music_active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InventoryRow {
    pub drug: Drug,
    pub stashed: i64,
    pub carried: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceTrend {
    Normal,
    Cheap,
    Expensive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PriceView {
    pub price: i64,
    pub trend: PriceTrend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatView {
    pub width: usize,
    pub damage: i64,
    pub cops: i64,
    pub arsenal: ArsenalView,
}

impl GameState {
    pub(crate) const fn screen_center(&self) -> usize {
        self.screen.half_width
    }

    pub(crate) fn hud_view(&self) -> HudView {
        HudView {
            day: self.day(),
            days_left: self.days_left(),
            borough: self.borough(),
            hold: self.hold(),
            bank: self.bank(),
            debt: self.debt(),
            cash: self.cash(),
            arsenal: self.arsenal_view(),
            music_active: self.music_active(),
        }
    }

    pub(crate) fn inventory_rows(&self) -> [InventoryRow; DRUG_COUNT] {
        ALL_DRUGS.map(|drug| InventoryRow {
            drug,
            stashed: self.stashed_units(drug),
            carried: self.carried_units(drug),
        })
    }

    pub(crate) const fn price_view(&self, drug: Drug) -> PriceView {
        let trend = if self.is_price_cheap(drug) {
            PriceTrend::Cheap
        } else if self.is_price_expensive(drug) {
            PriceTrend::Expensive
        } else {
            PriceTrend::Normal
        };

        PriceView {
            price: self.price(drug),
            trend,
        }
    }

    pub(crate) const fn combat_view(&self) -> CombatView {
        CombatView {
            width: self.screen.width,
            damage: self.damage(),
            cops: self.cops(),
            arsenal: self.arsenal_view(),
        }
    }

    pub(crate) fn toggle_music(&self) {
        if let Ok(mut player) = self.music.lock() {
            player.toggle();
        }
    }

    const fn arsenal_view(&self) -> ArsenalView {
        if self.has_shotgun() {
            ArsenalView::Shotgun
        } else {
            ArsenalView::Guns(self.guns())
        }
    }

    fn music_active(&self) -> bool {
        self.music
            .lock()
            .is_ok_and(|player| player.state() == MusicState::Playing)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Drug;
    use crate::test_support::game_builder;

    use super::{ArsenalView, PriceTrend};

    #[test]
    fn hud_view_exposes_render_data_without_raw_state_access() {
        let game = game_builder()
            .with_shotgun()
            .with_carried_drugs(Drug::Weed, 7)
            .build();

        let hud = game.hud_view();
        let rows = game.inventory_rows();

        assert_eq!(hud.day, 0);
        assert_eq!(hud.hold, 93);
        assert_eq!(hud.arsenal, ArsenalView::Shotgun);
        assert_eq!(rows[Drug::Weed as usize].carried, 7);
    }

    #[test]
    fn price_and_combat_views_encode_display_state() {
        let game = game_builder()
            .with_price(Drug::Cocaine, 10_000)
            .with_price_spike(Drug::Cocaine, 2)
            .with_cops(4)
            .with_damage(9)
            .with_guns(2)
            .build();

        let price = game.price_view(Drug::Cocaine);
        let combat = game.combat_view();

        assert_eq!(price.price, 20_000);
        assert_eq!(price.trend, PriceTrend::Expensive);
        assert_eq!(combat.cops, 4);
        assert_eq!(combat.damage, 9);
        assert_eq!(combat.arsenal, ArsenalView::Guns(2));
    }

    #[test]
    fn day_reflects_elapsed_campaign_days() {
        let mut game = game_builder().build();

        for _ in 0..30 {
            assert_eq!(game.advance_day(), crate::model::DayAdvance::Continue);
        }
        assert_eq!(game.day(), 30);

        assert_eq!(game.advance_day(), crate::model::DayAdvance::GameOver);
        assert_eq!(game.days_left(), 0);
        assert_eq!(game.day(), 31);
    }
}
