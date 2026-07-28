use std::io::{self, Write};

use super::{BankAction, LoanAction, MarketEntry, MarketMenuAction, StashAction, TradeAction};
use crate::ansi::{CYAN, DEVILED_EGGS, GREEN, MAGENTA, RED, YELLOW, paint};
use crate::engine::{RuleError, ScoreSummary};
use crate::input::enable_input_echo;
use crate::model::{Borough, Drug, GameState};
use crate::scoreboard::HighScoreTable;
use crate::state::format_number;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MarketAmountContext {
    RepayDebt,
    Borrow,
    DepositBank,
    WithdrawBank,
    StashDrug,
    TakeDrug,
    BuyDrug,
    SellDrug,
}

impl MarketAmountContext {
    const fn error_message(self, error: RuleError) -> &'static str {
        match (self, error) {
            (_, RuleError::InvalidAmount) => "THAT AIN'T A VALID AMOUNT MAN !!",
            (Self::RepayDebt | Self::DepositBank | Self::BuyDrug, RuleError::InsufficientCash) => {
                "YOU DON'T HAVE THAT MUCH CASH MAN !!"
            }
            (Self::RepayDebt, RuleError::InsufficientDebt) => "YOU DON'T OWE THAT MUCH MAN !!",
            (Self::Borrow, RuleError::LoanLimitExceeded) => "YOU THINK HE IS CRAZY MAN !!!",
            (Self::WithdrawBank, RuleError::InsufficientBankFunds) => {
                "YOU DON'T HAVE THAT MUCH IN THE BANK !!"
            }
            (Self::StashDrug, RuleError::InsufficientCarriedDrugs) => {
                "YOU DON'T HAVE THAT MUCH ON YOU MAN !!"
            }
            (Self::TakeDrug | Self::BuyDrug, RuleError::InsufficientCarrySpace) => {
                "YOU CAN'T CARRY THAT MUCH MAN !!"
            }
            (Self::TakeDrug, RuleError::InsufficientStashedDrugs) => {
                "YOU DON'T HAVE THAT MUCH IN THE STASH !!"
            }
            (Self::SellDrug, RuleError::InsufficientCarriedDrugs) => {
                "YOU DON'T HAVE THAT MUCH TO SELL MAN !!"
            }
            _ => "THAT AIN'T GONNA WORK MAN !!",
        }
    }
}

impl GameState {
    pub(crate) fn yn_prompt<F1, F2>(
        &mut self,
        prompt: impl std::fmt::Display,
        yes_action: F1,
        no_action: F2,
    ) where
        F1: FnOnce(&mut Self),
        F2: FnOnce(&mut Self),
    {
        print!("{prompt}");
        let _ = io::stdout().flush();

        loop {
            match self.getch_key() {
                'y' | 'Y' => {
                    yes_action(self);
                    break;
                }
                'n' | 'N' => {
                    no_action(self);
                    break;
                }
                _ => {}
            }
        }
    }

    pub(crate) fn run_market_phase(&mut self) {
        let entry = self.market_entry();
        self.prepare_market_phase();

        if entry == MarketEntry::LoanSharkVisit {
            self.loan();
        }

        self.buy_sell_jet();
    }

    pub(crate) fn finish_game(&self) -> ! {
        use crate::ui::clear_screen;

        enable_input_echo();
        clear_screen();

        let ScoreSummary {
            total_money,
            score,
            dealer_rank,
            comment,
        } = self.score_summary();
        let total_money_display = format_number(total_money);

        println!(
            "{}\nYOU SURVIVED FOR {} DAYS!",
            paint(DEVILED_EGGS, "GAME OVER"),
            paint(YELLOW, self.day())
        );
        println!("YOUR TOTAL MONEY: {}", paint(GREEN, total_money_display));
        println!("YOUR SCORE: {} OUT OF 100", paint(MAGENTA, score));
        println!("DEALER RANK: {} ... {comment}", paint(CYAN, dealer_rank));

        let mut high_scores = HighScoreTable::load();
        let save_failed = high_scores.record(total_money, dealer_rank).is_err();

        if save_failed {
            println!("{}", paint(RED, "COULDN'T SAVE HIGH SCORES MAN !!"));
        }

        println!("{}", paint(DEVILED_EGGS, "\nTOP 10 HIGH SCORES\n"));
        for (index, score) in high_scores.entries().iter().enumerate() {
            println!(
                "{:02}  {}  {}  {}",
                index + 1,
                paint(GREEN, format_number(score.total_money)),
                paint(CYAN, score.dealer_rank.as_str()),
                paint(MAGENTA, score.date.as_str())
            );
        }

        Self::quit(0);
    }

    fn refresh_market_screen(&mut self) {
        self.hud();
        self.show_prices();
    }

    fn choose_drug(&self, prompt: &str) -> Option<Drug> {
        print!("{prompt}");
        let _ = io::stdout().flush();

        let reply = self.getch_key().to_ascii_lowercase();
        println!("{reply}");
        Drug::from_key(reply)
    }

    fn loan(&mut self) {
        self.hud();
        self.yn_prompt(
            "DO YOU WANT TO VISIT THE LOAN SHARK? ",
            Self::do_loan_shark,
            Self::do_stash_menu,
        );
    }

    fn do_loan_shark(&mut self) {
        self.prompt_market_amount_with_retry(
            Self::hud,
            "HOW MUCH TO REPAY? ",
            MarketAmountContext::RepayDebt,
            |state, amount| state.apply_loan_action(LoanAction::Repay(amount)),
        );

        self.prompt_market_amount_with_retry(
            Self::hud,
            "HOW MUCH TO BORROW? ",
            MarketAmountContext::Borrow,
            |state, amount| state.apply_loan_action(LoanAction::Borrow(amount)),
        );

        self.do_stash_menu();
    }

    fn do_stash_menu(&mut self) {
        self.hud();
        self.yn_prompt(
            "DO YOU WISH TO TRANSFER DRUGS TO YOUR STASH? ",
            |state| loop {
                state.hud();
                if let Some(drug) = state.choose_drug("WHICH DRUG DO YOU WANT TO STASH OR TAKE? ") {
                    state.do_stash_deposit(drug);
                    break;
                }
            },
            |_| {},
        );
        self.do_bank_menu();
    }

    fn do_stash_deposit(&mut self, drug: Drug) {
        let stash_prompt = format!("HOW MUCH {} DO YOU WANT TO STASH? ", drug.as_str());
        self.prompt_market_amount_with_retry(
            Self::hud,
            &stash_prompt,
            MarketAmountContext::StashDrug,
            |state, amount| state.apply_stash_action(StashAction::Store { drug, amount }),
        );

        let take_prompt = format!("HOW MUCH {} DO YOU WANT TO TAKE? ", drug.as_str());
        self.prompt_market_amount_with_retry(
            Self::hud,
            &take_prompt,
            MarketAmountContext::TakeDrug,
            |state, amount| state.apply_stash_action(StashAction::Take { drug, amount }),
        );
    }

    fn do_bank_menu(&mut self) {
        self.hud();
        self.yn_prompt(
            "DO YOU WISH TO VISIT THE BANK? ",
            |state| {
                state.prompt_market_amount_with_retry(
                    Self::hud,
                    "HOW MUCH TO DEPOSIT? ",
                    MarketAmountContext::DepositBank,
                    |state, amount| state.apply_bank_action(BankAction::Deposit(amount)),
                );

                state.prompt_market_amount_with_retry(
                    Self::hud,
                    "HOW MUCH TO WITHDRAW? ",
                    MarketAmountContext::WithdrawBank,
                    |state, amount| state.apply_bank_action(BankAction::Withdraw(amount)),
                );
            },
            |_| {},
        );
    }

    fn buy_sell_jet(&mut self) {
        loop {
            self.refresh_market_screen();
            print!("WILL YOU BUY, SELL OR JET? ");
            let _ = io::stdout().flush();

            match MarketMenuAction::from_key(self.getch_key()) {
                Some(MarketMenuAction::Buy) => {
                    println!();
                    self.do_buy();
                }
                Some(MarketMenuAction::Sell) => {
                    println!();
                    self.do_sell();
                }
                Some(MarketMenuAction::Jet) => {
                    println!();
                    self.do_jet();
                    return;
                }
                Some(MarketMenuAction::ToggleMusic) => {
                    self.toggle_music();
                }
                None => {}
            }
        }
    }

    fn do_buy(&mut self) {
        if let Some(drug) = self.choose_drug("WHAT WILL YOU BUY? ") {
            let prompt = format!("HOW MUCH {} DO YOU WANT TO BUY? ", drug.as_str());
            self.prompt_market_amount_with_retry(
                |state| {
                    state.refresh_market_screen();
                    let quote = state.buy_quote(drug);
                    println!("YOU CAN AFFORD ( {} )", quote.affordable_units);
                    if quote.affordable_units > quote.hold_limit {
                        println!("YOU CAN HOLD ( {} )", paint(YELLOW, quote.hold_limit));
                    }
                },
                &prompt,
                MarketAmountContext::BuyDrug,
                |state, amount| state.apply_trade_action(TradeAction::Buy { drug, amount }),
            );
        }
    }

    fn do_sell(&mut self) {
        if let Some(drug) = self.choose_drug("WHAT WILL YOU SELL? ") {
            let prompt = format!("HOW MUCH {} DO YOU WANT TO SELL? ", drug.as_str());
            self.prompt_market_amount_with_retry(
                |state| {
                    state.refresh_market_screen();
                    let quote = state.sell_quote(drug);
                    println!("YOU CAN SELL ( {} )", quote.carried_units);
                },
                &prompt,
                MarketAmountContext::SellDrug,
                |state, amount| state.apply_trade_action(TradeAction::Sell { drug, amount }),
            );
        }
    }

    fn do_jet(&mut self) {
        loop {
            self.hud();
            println!();
            println!("   1) BRONX        2) GHETTO          3) CENTRAL PARK");
            println!("   4) MANHATTAN    5) CONEY ISLAND    6) BROOKLYN");
            println!();
            print!("WHERE TO DUDE: ");
            let _ = io::stdout().flush();

            let reply = self.getch_key();
            println!("{reply}");
            if let Some(destination) = Borough::from_travel_key(reply) {
                self.travel_to(destination);
                return;
            }
        }
    }

    fn show_market_amount_error(&self, context: MarketAmountContext, error: RuleError) {
        println!("{}", paint(RED, context.error_message(error)));
        let _ = io::stdout().flush();
        self.wait_for_key_press();
    }

    fn prompt_market_amount_with_retry<R, F>(
        &mut self,
        mut render: R,
        prompt: &str,
        context: MarketAmountContext,
        mut action: F,
    ) where
        R: FnMut(&mut Self),
        F: FnMut(&mut Self, i64) -> Result<(), RuleError>,
    {
        loop {
            render(self);
            let amount = self.prompt_number(prompt);
            if amount == 0 {
                break;
            }

            match action(self, amount) {
                Ok(()) => break,
                Err(error) => self.show_market_amount_error(context, error),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::RuleError;

    use super::MarketAmountContext;

    #[test]
    fn market_amount_errors_use_context_specific_messages() {
        assert_eq!(
            MarketAmountContext::RepayDebt.error_message(RuleError::InsufficientDebt),
            "YOU DON'T OWE THAT MUCH MAN !!"
        );
        assert_eq!(
            MarketAmountContext::TakeDrug.error_message(RuleError::InsufficientStashedDrugs),
            "YOU DON'T HAVE THAT MUCH IN THE STASH !!"
        );
        assert_eq!(
            MarketAmountContext::SellDrug.error_message(RuleError::InsufficientCarriedDrugs),
            "YOU DON'T HAVE THAT MUCH TO SELL MAN !!"
        );
    }
}
