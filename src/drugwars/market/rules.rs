use crate::engine::RuleError;
use crate::model::{Drug, GamePhase, GameState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketEntry {
    LoanSharkVisit,
    Trading,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketMenuAction {
    Buy,
    Sell,
    Jet,
    ToggleMusic,
}

impl MarketMenuAction {
    pub const fn from_key(key: char) -> Option<Self> {
        match key {
            'b' | 'B' => Some(Self::Buy),
            's' | 'S' => Some(Self::Sell),
            'j' | 'J' => Some(Self::Jet),
            'm' | 'M' => Some(Self::ToggleMusic),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoanAction {
    Repay(i64),
    Borrow(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BankAction {
    Deposit(i64),
    Withdraw(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StashAction {
    Store { drug: Drug, amount: i64 },
    Take { drug: Drug, amount: i64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeAction {
    Buy { drug: Drug, amount: i64 },
    Sell { drug: Drug, amount: i64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuyQuote {
    pub affordable_units: i64,
    pub hold_limit: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SellQuote {
    pub carried_units: i64,
}

impl GameState {
    pub(crate) const fn market_entry(&self) -> MarketEntry {
        match self.phase() {
            GamePhase::Begin | GamePhase::BronxDo => MarketEntry::LoanSharkVisit,
            GamePhase::Normal => MarketEntry::Trading,
        }
    }

    pub(crate) fn prepare_market_phase(&mut self) {
        match self.phase() {
            GamePhase::Normal => {}
            GamePhase::Begin | GamePhase::BronxDo => {
                self.set_phase(GamePhase::Normal);
            }
        }
    }

    pub(crate) const fn buy_quote(&self, drug: Drug) -> BuyQuote {
        BuyQuote {
            affordable_units: self.affordable_units(drug),
            hold_limit: self.hold(),
        }
    }

    pub(crate) const fn sell_quote(&self, drug: Drug) -> SellQuote {
        SellQuote {
            carried_units: self.carried_units(drug),
        }
    }

    pub(crate) fn apply_loan_action(&mut self, action: LoanAction) -> Result<(), RuleError> {
        match action {
            LoanAction::Repay(amount) => self.repay_debt(amount),
            LoanAction::Borrow(amount) => self.borrow_from_shark(amount),
        }
    }

    pub(crate) fn apply_bank_action(&mut self, action: BankAction) -> Result<(), RuleError> {
        match action {
            BankAction::Deposit(amount) => self.deposit_bank_funds(amount),
            BankAction::Withdraw(amount) => self.withdraw_bank_funds(amount),
        }
    }

    pub(crate) fn apply_stash_action(&mut self, action: StashAction) -> Result<(), RuleError> {
        match action {
            StashAction::Store { drug, amount } => self.stash_drug_units(drug, amount),
            StashAction::Take { drug, amount } => self.take_stashed_drug_units(drug, amount),
        }
    }

    pub(crate) fn apply_trade_action(&mut self, action: TradeAction) -> Result<(), RuleError> {
        match action {
            TradeAction::Buy { drug, amount } => self.buy_drug_units(drug, amount),
            TradeAction::Sell { drug, amount } => self.sell_drug_units(drug, amount),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Borough, Drug, GamePhase};
    use crate::test_support::game_builder;

    use super::{BankAction, LoanAction, MarketEntry, MarketMenuAction, StashAction, TradeAction};

    #[test]
    fn market_entry_encodes_required_loan_visits() {
        let mut game = game_builder().build();
        assert_eq!(game.market_entry(), MarketEntry::LoanSharkVisit);
        game.prepare_market_phase();
        assert_eq!(game.phase(), GamePhase::Normal);

        let mut game = game_builder().with_phase(GamePhase::BronxDo).build();
        assert_eq!(game.market_entry(), MarketEntry::LoanSharkVisit);
        game.prepare_market_phase();
        assert_eq!(game.phase(), GamePhase::Normal);

        let mut game = game_builder().with_phase(GamePhase::Normal).build();
        assert_eq!(game.market_entry(), MarketEntry::Trading);
        game.prepare_market_phase();
        assert_eq!(game.phase(), GamePhase::Normal);
    }

    #[test]
    fn typed_market_actions_delegate_to_existing_rules() {
        let mut game = game_builder().with_price(Drug::Weed, 100).build();

        game.apply_loan_action(LoanAction::Borrow(1_000))
            .expect("borrow should succeed");
        game.apply_bank_action(BankAction::Deposit(500))
            .expect("deposit should succeed");
        game.apply_trade_action(TradeAction::Buy {
            drug: Drug::Weed,
            amount: 5,
        })
        .expect("buy should succeed");
        game.apply_stash_action(StashAction::Store {
            drug: Drug::Weed,
            amount: 2,
        })
        .expect("stash should succeed");

        assert_eq!(game.debt(), 6_500);
        assert_eq!(game.bank(), 500);
        assert_eq!(game.carried_units(Drug::Weed), 3);
        assert_eq!(game.stashed_units(Drug::Weed), 2);
    }

    #[test]
    fn menu_and_travel_choices_are_typed() {
        assert_eq!(MarketMenuAction::from_key('b'), Some(MarketMenuAction::Buy));
        assert_eq!(Borough::from_travel_key('6'), Some(Borough::Brooklyn));

        let mut game = game_builder().build();
        game.travel_to(Borough::Manhattan);
        assert_eq!(game.borough(), Borough::Manhattan);
    }
}
