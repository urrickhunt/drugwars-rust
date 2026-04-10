use crate::input::{disable_input_echo, enable_input_echo};
use crate::model::GameState;

use super::rules::{FlowStep, InstructionChoice};

impl GameState {
    pub(crate) fn start_game(&mut self) {
        self.term_info();

        if let Ok(mut player) = self.music.lock() {
            player.start();
        }

        let mut step = FlowStep::initial();
        loop {
            step = match step {
                FlowStep::Intro => {
                    disable_input_echo();
                    self.show_startup_screen();
                    enable_input_echo();
                    FlowStep::after_intro()
                }
                FlowStep::InstructionPrompt => match self.prompt_instruction_choice() {
                    InstructionChoice::Show => {
                        FlowStep::after_instruction_prompt(InstructionChoice::Show)
                    }
                    InstructionChoice::Skip => {
                        FlowStep::after_instruction_prompt(InstructionChoice::Skip)
                    }
                },
                FlowStep::Instructions => {
                    disable_input_echo();
                    self.show_instructions();
                    enable_input_echo();
                    FlowStep::after_instructions()
                }
                FlowStep::OpeningEvent => {
                    self.roll_event();
                    FlowStep::after_opening_event()
                }
                FlowStep::Market => {
                    self.run_market_phase();
                    FlowStep::after_market()
                }
                FlowStep::DayAdvance => FlowStep::after_day_advance(self.advance_day()),
                FlowStep::GunOffer => {
                    if self.should_offer_guns() {
                        self.gunsale();
                    }
                    FlowStep::after_gun_offer()
                }
                FlowStep::DailyEvent => {
                    self.roll_event();
                    FlowStep::after_daily_event()
                }
                FlowStep::Fight => {
                    self.roll_fight();
                    FlowStep::after_fight()
                }
                FlowStep::GameOver => self.finish_game(),
            };
        }
    }

    fn prompt_instruction_choice(&self) -> InstructionChoice {
        disable_input_echo();

        loop {
            match self.getch_key().to_ascii_lowercase() {
                'y' => {
                    enable_input_echo();
                    return InstructionChoice::Show;
                }
                'n' => {
                    enable_input_echo();
                    return InstructionChoice::Skip;
                }
                _ => {}
            }
        }
    }
}
