use std::io::{self, Write};

use super::{
    CombatAction, CombatResolution, CombatRound, DoctorTreatment, FightOutcome, FireResult,
};
use crate::ansi::{paint, BLUE, CYAN, GREEN, MAGENTA, RED, YELLOW};
use crate::model::GameState;
use crate::ui::{clear_screen, sentence_lines};
use crate::view::ArsenalView;

impl GameState {
    pub(crate) fn roll_fight(&mut self) {
        let Some(start) = self.start_fight() else {
            return;
        };
        self.hud();
        print!(
            "{}",
            paint(
                RED,
                format_args!(
                    "OFFICER HARDASS AND {} OF HIS DEPUTIES ARE CHASING YOU !!!!!",
                    start.cops
                ),
            )
        );
        let _ = io::stdout().flush();
        self.wait_for_key_press();

        match self.do_fight() {
            FightOutcome::Dead => self.finish_game(),
            FightOutcome::Escaped | FightOutcome::Killed => self.do_doctor_check(),
        }
    }

    fn fight_hud(&mut self) {
        self.term_info();
        clear_screen();
        let combat = self.combat_view();

        println!();
        let bar = "█".repeat(combat.width);
        println!("{}", paint(MAGENTA, bar.as_str()));
        println!();

        match combat.arsenal {
            ArsenalView::Shotgun => {
                println!(
                    "{}DAMAGE    {}       COPS    {}       SHOTGUN {}",
                    " ".repeat(combat.width / 8),
                    paint(MAGENTA, combat.damage),
                    paint(CYAN, combat.cops),
                    paint(BLUE, 1),
                );
            }
            ArsenalView::Guns(guns) => {
                println!(
                    "{}DAMAGE    {}       COPS    {}       GUNS    {}",
                    " ".repeat(combat.width / 8),
                    paint(MAGENTA, combat.damage),
                    paint(CYAN, combat.cops),
                    paint(BLUE, guns)
                );
            }
        }

        println!();
        println!("{}", paint(MAGENTA, bar.as_str()));
        println!("\n\n");
    }

    fn do_fight(&mut self) -> FightOutcome {
        loop {
            self.term_info();
            self.fight_hud();

            let action = match self.prompt_fight_action() {
                'r' | 'y' => Some(CombatAction::Run),
                'f' if self.is_armed() => Some(CombatAction::Fight),
                _ => None,
            };
            let Some(action) = action else {
                continue;
            };

            let round = self.resolve_combat_action(action);
            if let Some(outcome) = self.render_combat_round(round) {
                return outcome;
            }
        }
    }

    fn prompt_fight_action(&self) -> char {
        if self.is_armed() {
            print!("WILL YOU RUN OR FIGHT? ");
        } else {
            print!("WILL YOU RUN? ");
        }
        let _ = io::stdout().flush();

        let reply = self.getch_key().to_ascii_lowercase();
        println!("{reply}");
        reply
    }

    fn render_combat_round(&mut self, round: CombatRound) -> Option<FightOutcome> {
        match round.resolution {
            CombatResolution::Escaped => {
                self.show_combat_event(paint(CYAN, "YOU LOST THEM IN THE ALLEYS !!"));
            }
            CombatResolution::RanIntoReturnFire(return_fire) => {
                self.render_return_fire(return_fire);
            }
            CombatResolution::FiredAndMissed(return_fire) => {
                self.show_combat_event("YOU'RE FIRING ON THEM!!");
                self.show_combat_event("YOU MISSED THEM !!");
                self.render_return_fire(return_fire);
            }
            CombatResolution::KilledAll { reward } => {
                self.show_combat_event("YOU'RE FIRING ON THEM!!");
                self.show_combat_event(paint(GREEN, "YOU KILLED ALL OF THEM!!!!"));
                self.show_combat_event(paint(
                    GREEN,
                    format_args!("YOU FOUND {reward} DOLLARS ON OFFICER HARDASS' CARCASS !!"),
                ));
            }
            CombatResolution::FiredAndHit { kills, return_fire } => {
                self.show_combat_event("YOU'RE FIRING ON THEM!!");

                if kills > 1 {
                    self.show_combat_event(paint(
                        YELLOW,
                        format_args!("YOU KILLED {kills} OF THEM!!"),
                    ));
                } else {
                    self.show_combat_event(paint(YELLOW, "YOU KILLED ONE!!"));
                }

                self.render_return_fire(return_fire);
            }
        }

        if round.outcome == Some(FightOutcome::Dead) {
            self.show_combat_event(paint(RED, "THEY WASTED YOU MAN !! WHAT A DRAG !!!"));
        }

        round.outcome
    }

    fn render_return_fire(&mut self, return_fire: FireResult) {
        self.show_combat_event("THEY ARE FIRING ON YOU MAN !!");
        match return_fire {
            FireResult::Missed => self.show_combat_event("THEY MISSED !!"),
            FireResult::Hit { .. } => {
                self.show_combat_event(paint(RED, "YOU'VE BEEN HIT !!"));
            }
        }
    }

    fn do_doctor_check(&mut self) {
        let Some(offer) = self.roll_doctor_offer() else {
            return;
        };

        self.fight_hud();
        self.yn_prompt(
            paint(
                CYAN,
                format_args!(
                    "WILL YOU PAY {} DOLLARS TO HAVE A DOCTOR SEW YOU UP? ",
                    offer.price
                ),
            ),
            move |state| {
                println!();
                match state.accept_doctor_offer(offer) {
                    DoctorTreatment::Healed => {
                        print!("{}", paint(GREEN, "THE DOCTOR PATCHES YOU UP !!"));
                    }
                    DoctorTreatment::CouldNotAfford => {
                        print!(
                            "{}",
                            paint(
                                RED,
                                sentence_lines("YOU CAN'T AFFORD IT MAN !! YOU'RE ON YOUR OWN !!"),
                            )
                        );
                    }
                }
                let _ = io::stdout().flush();
                state.wait_for_key_press();
            },
            |_| {},
        );
    }

    fn show_combat_event(&mut self, message: impl std::fmt::Display) {
        self.fight_hud();
        let formatted = sentence_lines(&message.to_string());
        print!("{formatted} ");
        let _ = io::stdout().flush();
        self.wait_for_key_press();
    }
}
