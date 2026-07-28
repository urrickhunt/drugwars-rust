use std::io::{self, Write};

use super::{
    BrowniesOutcome, CoatOffer, DailyEvent, FindDrugsOutcome, GunSaleOffer, MuggingOutcome,
};
use crate::ansi::{ASTER, BLUE, CYAN, DEVILED_EGGS, GREEN, MAGENTA, RED, YELLOW, paint};
use crate::engine::RuleError;
use crate::model::GameState;
use crate::state::format_number;
use crate::ui::sentence_lines;

impl GameState {
    pub(crate) fn roll_event(&mut self) {
        match self.roll_daily_event() {
            DailyEvent::PriceChange(change) => {
                self.pause_event(paint(change.color, change.message));
            }
            DailyEvent::ParaquatOffer => self.handle_paraquat_offer(),
            DailyEvent::Mugging(outcome) => self.show_mugging(outcome),
            DailyEvent::CoatOffer(offer) => self.handle_coat_offer(offer),
            DailyEvent::Brownies(outcome) => {
                self.pause_event(paint(RED, Self::brownies_message(outcome)));
                self.pause_event(paint(RED, "THEY WERE GREAT !!"));
            }
            DailyEvent::PoliceDogs(outcome) => {
                self.pause_event(paint(
                    RED,
                    format_args!("POLICE DOGS CHASE YOU {} BLOCKS !!", outcome.divisor),
                ));
                self.pause_event(paint(RED, "YOU DROPPED SOME DRUGS !! THAT'S A DRAG MAN !!"));
            }
            DailyEvent::FindDrugs(outcome) => self.show_found_drugs(outcome),
        }
    }

    pub(crate) fn gunsale(&mut self) {
        self.hud();
        let sale = self.roll_gun_sale();

        println!("{}", paint(BLUE, "HEY DUDE, THE PRICES OF GUNS HERE ARE:"));
        println!();
        for offer in sale.offers() {
            println!("{}", Self::format_gun_sale_offer(*offer));
        }

        println!();
        print!("WILL YOU STRAP OR NAH? ");
        let _ = io::stdout().flush();

        loop {
            let reply = self.getch_key();
            match reply {
                'n' | 'N' => break,
                _ => {
                    if let Some(offer) = sale.offer_for_key(reply) {
                        self.buy_firearm(offer);
                        break;
                    }
                }
            }
        }
    }

    fn format_gun_sale_offer(offer: GunSaleOffer) -> String {
        let price = if offer.is_shotgun() {
            format_number(offer.price)
        } else {
            offer.price.to_string()
        };
        let line = format!("    {}) {:<30} {}", offer.option_key, offer.name(), price);

        if offer.is_shotgun() {
            paint(DEVILED_EGGS, line).to_string()
        } else {
            paint(YELLOW, line).to_string()
        }
    }

    fn handle_paraquat_offer(&mut self) {
        self.hud();
        println!(
            "{}",
            paint(
                MAGENTA,
                sentence_lines(
                    "THERE IS SOME WEED THAT SMELLS LIKE PARAQUAT HERE !! IT LOOKS GOOD !!"
                ),
            )
        );
        let _ = io::stdout().flush();
        self.yn_prompt(
            paint(MAGENTA, "WILL YOU SMOKE IT? "),
            |state| {
                state.hud();
                print!(
                    "{} ",
                    paint(
                        ASTER,
                        "YOU HALLUCINATE FOR THREE DAYS ON THE WILDEST TRIP YOU EVER IMAGINED !!",
                    )
                );
                let _ = io::stdout().flush();
                state.wait_for_key_press();
                print!(
                    "{} ",
                    paint(
                        ASTER,
                        "THEN YOU DIE BECAUSE YOUR BRAIN HAS DISINTEGRATED !!",
                    )
                );
                let _ = io::stdout().flush();
                state.wait_for_key_press();
                state.finish_game();
            },
            |_| {},
        );
    }

    fn show_mugging(&mut self, outcome: MuggingOutcome) {
        if outcome.shotgun_saved_you {
            self.pause_event(paint(
                GREEN,
                "SOMEONE TRIED TO MUG YOU !! BUT YOU FUCKED THEM WITH YOUR SHOTGUN !!",
            ));
        } else {
            self.pause_event(paint(RED, "YOU WERE MUGGED IN THE SUBWAY !!"));
        }
    }

    fn handle_coat_offer(&mut self, offer: CoatOffer) {
        self.hud();
        print!(
            "{} ",
            paint(
                DEVILED_EGGS,
                format_args!(
                    "WILL YOU BUY A NEW TRENCH COAT WITH MORE POCKETS FOR {}?",
                    offer.price
                ),
            )
        );
        let _ = io::stdout().flush();

        self.yn_prompt(
            "",
            move |state| {
                println!();
                match state.accept_coat_offer(offer) {
                    Ok(()) => print!("{}", paint(GREEN, "ENJOY !!")),
                    Err(RuleError::InsufficientCash) => {
                        print!("{}", paint(RED, "YOU CAN'T AFFORD IT MAN !!"));
                    }
                    Err(_) => {}
                }
                let _ = io::stdout().flush();
                state.wait_for_key_press();
            },
            |_| {},
        );
    }

    fn buy_firearm(&mut self, offer: GunSaleOffer) {
        let result = if offer.is_shotgun() {
            self.buy_shotgun(offer.price)
        } else {
            self.buy_handgun(offer.price)
        };

        println!();
        match result {
            Ok(()) if offer.is_shotgun() => {
                print!("{}", paint(GREEN, "ONE PUMP AND THEY'RE DONE !!"));
            }
            Ok(()) => print!(
                "{}",
                paint(GREEN, format_args!("ENJOY THE {} !!", offer.name()))
            ),
            Err(RuleError::InsufficientCash) => {
                print!("{}", paint(RED, "YOU CAN'T AFFORD IT MAN !!"));
            }
            Err(_) => {}
        }

        let _ = io::stdout().flush();
        self.wait_for_key_press();
    }

    fn show_found_drugs(&mut self, outcome: FindDrugsOutcome) {
        self.hud();
        print!(
            "{}",
            paint(
                CYAN,
                format_args!(
                    "{} ",
                    sentence_lines(&format!(
                        "YOU FIND {} UNITS OF {} ON A DEAD DUDE IN THE SUBWAY !!",
                        outcome.found_amount,
                        outcome.drug.as_str()
                    ))
                ),
            )
        );

        if outcome.taken_amount < outcome.found_amount {
            print!(
                "{}",
                paint(
                    YELLOW,
                    format_args!(
                        "\n{} ",
                        sentence_lines(&format!(
                            "BUT YOU CAN ONLY CARRY {} MAN !!",
                            outcome.taken_amount
                        ))
                    ),
                )
            );
        }

        let _ = io::stdout().flush();
        self.wait_for_key_press();
    }

    const fn brownies_message(outcome: BrowniesOutcome) -> &'static str {
        if outcome.dropped > 0 {
            "YOUR MAMA MADE SOME BROWNIES AND USED YOUR WEED !!"
        } else {
            "YOUR MAMA MADE SOME BROWNIES !!"
        }
    }

    fn pause_event(&mut self, message: impl std::fmt::Display) {
        self.hud();
        let formatted = sentence_lines(&message.to_string());
        print!("{formatted} ");
        let _ = io::stdout().flush();
        self.wait_for_key_press();
    }
}

#[cfg(test)]
mod tests {
    use super::{BrowniesOutcome, GameState};

    #[test]
    fn brownies_message_only_claims_weed_loss_when_some_was_dropped() {
        assert_eq!(
            GameState::brownies_message(BrowniesOutcome { dropped: 3 }),
            "YOUR MAMA MADE SOME BROWNIES AND USED YOUR WEED !!"
        );
        assert_eq!(
            GameState::brownies_message(BrowniesOutcome { dropped: 0 }),
            "YOUR MAMA MADE SOME BROWNIES !!"
        );
    }
}
