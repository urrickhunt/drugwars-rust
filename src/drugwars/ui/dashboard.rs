use crate::ansi::{paint, BLUE, BRIGHT_BLACK, CYAN, GREEN, MAGENTA, RESET, YELLOW};
use crate::model::{Drug, GameState};
use crate::state::{format_number, game_date_str};
use crate::view::{ArsenalView, PriceTrend};

use super::clear_screen;

const PRICE_ROWS: &[(Drug, Drug)] = &[
    (Drug::Cocaine, Drug::Weed),
    (Drug::Heroin, Drug::Speed),
    (Drug::Acid, Drug::Ludes),
];

impl GameState {
    pub(crate) fn hud(&mut self) {
        self.term_info();
        clear_screen();

        let hud = self.hud_view();
        let game_date = game_date_str(hud.day);
        let bank_formatted = format_number(hud.bank);
        let debt_formatted = format_number(hud.debt);
        let cash_formatted = format_number(hud.cash);
        let music_status = if hud.music_active {
            paint(GREEN, "♪")
        } else {
            paint(BRIGHT_BLACK, "♪")
        };

        println!("\n");
        println!(
            "DATE {}                        HOLD  {}  {}",
            game_date,
            paint(YELLOW, hud.hold),
            music_status
        );
        println!(
            "DAYS LEFT      {}                        {}\n",
            paint(MAGENTA, format_args!("{:02}", hud.days_left)),
            paint(BLUE, hud.borough)
        );

        println!("  ┌─────────────────────────┬─────────────────────────┐");
        println!("  │        STASH            │       TRENCH COAT       │");
        println!("  ├─────────────────────────┼─────────────────────────┤");
        for row in self.inventory_rows() {
            println!(
                "  │ {:<9}  {:<6}       │ {:<9}  {:<6}       │",
                row.drug.as_str(),
                row.stashed,
                row.drug.as_str(),
                row.carried
            );
        }
        println!("  │                         │                         │");
        match hud.arsenal {
            ArsenalView::Shotgun => {
                println!(
                    "  │ BANK       {CYAN}{bank_formatted:<12}{RESET} │ SHOTGUN                 │"
                );
            }
            ArsenalView::Guns(guns) => {
                println!(
                    "  │ BANK       {CYAN}{bank_formatted:<12}{RESET} │ GUNS       {guns:<6}       │"
                );
            }
        }
        println!(
            "  │ DEBT       {MAGENTA}{debt_formatted:<8}{RESET}     │ CASH       {GREEN}{cash_formatted:<12}{RESET} │"
        );
        println!("  └─────────────────────────┴─────────────────────────┘\n");
    }

    pub(crate) fn show_prices(&self) {
        let price_str = |drug: Drug| -> String {
            let price_view = self.price_view(drug);
            let price = format_number(price_view.price);
            let spaces = " ".repeat(11usize.saturating_sub(price.len()));

            match price_view.trend {
                PriceTrend::Cheap => format!("{}{spaces}", paint(YELLOW, price)),
                PriceTrend::Expensive => format!("{}{spaces}", paint(CYAN, price)),
                PriceTrend::Normal => format!("{price}{spaces}"),
            }
        };

        println!("HEY DUDE, THE PRICES OF DRUGS HERE ARE:");
        println!();
        for &(left, right) in PRICE_ROWS {
            println!(
                "    {:<10} {}    {:<10} {}",
                left.as_str(),
                price_str(left),
                right.as_str(),
                price_str(right),
            );
        }
        println!();
    }
}
