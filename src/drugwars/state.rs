use chrono::{Datelike, NaiveDate};
use rand::Rng;
use std::process;
use terminal_size::{terminal_size, Height, Width};

use crate::audio;
use crate::balance::DRUG_PRICE_RANGES;
use crate::model::{GameState, PlayerState, ScreenState, WorldState};

impl GameState {
    pub(crate) fn new() -> Self {
        let music = audio::init_music();

        let mut game = Self {
            player: PlayerState::new(),
            world: WorldState::new(),
            screen: ScreenState::new(),
            music,
        };

        game.term_info();
        game
    }

    pub(crate) fn quit(code: i32) -> ! {
        audio::stop_music();
        process::exit(code);
    }

    pub(crate) fn roll_prices(&mut self) {
        self.reset_market_highlights();

        let mut rng = rand::thread_rng();
        for range in DRUG_PRICE_RANGES {
            self.set_price(range.drug, rng.gen_range(range.min_price..=range.max_price));
        }
    }

    pub(crate) fn term_info(&mut self) {
        if let Some((Width(width), Height(height))) = terminal_size() {
            self.screen.width = width as usize;
            self.screen.height = height as usize;
            self.screen.half_width = self.screen.width / 2;
        }
    }
}

pub fn format_number(value: i64) -> String {
    let digits = value.abs().to_string();
    let mut formatted = String::with_capacity(digits.len() + (digits.len() / 3) + 1);

    for (index, digit) in digits.chars().rev().enumerate() {
        if index != 0 && index % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(digit);
    }

    if value < 0 {
        formatted.push('-');
    }

    formatted.chars().rev().collect()
}

pub fn game_date_str(days_to_add: i64) -> String {
    let start_date = NaiveDate::from_ymd_opt(1983, 12, 4).expect("hardcoded start date is valid");
    let game_date = start_date + chrono::Duration::days(days_to_add);

    format!(
        "{:02} / {:02} / {:02}",
        game_date.month(),
        game_date.day(),
        game_date.year() % 100
    )
}

#[cfg(test)]
mod tests {
    use crate::test_support::game_builder;

    use crate::balance::DRUG_PRICE_RANGES;

    #[test]
    fn rolled_prices_stay_within_declared_ranges() {
        let mut game = game_builder().build();

        for _ in 0..32 {
            game.roll_prices();

            for range in DRUG_PRICE_RANGES {
                assert!(range.contains(game.price(range.drug)));
            }
        }
    }
}
