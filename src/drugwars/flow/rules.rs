use crate::model::DayAdvance;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FlowStep {
    Intro,
    InstructionPrompt,
    Instructions,
    OpeningEvent,
    Market,
    DayAdvance,
    GunOffer,
    DailyEvent,
    Fight,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum InstructionChoice {
    Show,
    Skip,
}

impl FlowStep {
    pub(super) const fn initial() -> Self {
        Self::Intro
    }

    pub(super) const fn after_intro() -> Self {
        Self::InstructionPrompt
    }

    pub(super) const fn after_instruction_prompt(choice: InstructionChoice) -> Self {
        match choice {
            InstructionChoice::Show => Self::Instructions,
            InstructionChoice::Skip => Self::OpeningEvent,
        }
    }

    pub(super) const fn after_instructions() -> Self {
        Self::OpeningEvent
    }

    pub(super) const fn after_opening_event() -> Self {
        Self::Market
    }

    pub(super) const fn after_market() -> Self {
        Self::DayAdvance
    }

    pub(super) const fn after_day_advance(outcome: DayAdvance) -> Self {
        match outcome {
            DayAdvance::Continue => Self::GunOffer,
            DayAdvance::GameOver => Self::GameOver,
        }
    }

    pub(super) const fn after_gun_offer() -> Self {
        Self::DailyEvent
    }

    pub(super) const fn after_daily_event() -> Self {
        Self::Fight
    }

    pub(super) const fn after_fight() -> Self {
        Self::Market
    }
}

#[cfg(test)]
mod tests {
    use crate::model::DayAdvance;

    use super::{FlowStep, InstructionChoice};

    #[test]
    fn intro_path_can_skip_or_show_instructions() {
        assert_eq!(FlowStep::initial(), FlowStep::Intro);
        assert_eq!(FlowStep::after_intro(), FlowStep::InstructionPrompt);
        assert_eq!(
            FlowStep::after_instruction_prompt(InstructionChoice::Show),
            FlowStep::Instructions
        );
        assert_eq!(
            FlowStep::after_instruction_prompt(InstructionChoice::Skip),
            FlowStep::OpeningEvent
        );
        assert_eq!(FlowStep::after_instructions(), FlowStep::OpeningEvent);
    }

    #[test]
    fn gameplay_path_advances_through_day_states() {
        assert_eq!(FlowStep::after_opening_event(), FlowStep::Market);
        assert_eq!(FlowStep::after_market(), FlowStep::DayAdvance);
        assert_eq!(
            FlowStep::after_day_advance(DayAdvance::Continue),
            FlowStep::GunOffer
        );
        assert_eq!(FlowStep::after_gun_offer(), FlowStep::DailyEvent);
        assert_eq!(FlowStep::after_daily_event(), FlowStep::Fight);
        assert_eq!(FlowStep::after_fight(), FlowStep::Market);
        assert_eq!(
            FlowStep::after_day_advance(DayAdvance::GameOver),
            FlowStep::GameOver
        );
    }
}
