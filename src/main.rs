pub(crate) mod drugwars;

#[cfg(test)]
pub(crate) use drugwars::test_support;
pub(crate) use drugwars::{
    ansi, audio, balance, engine, input, model, scoreboard, state, ui, view,
};

fn main() {
    drugwars::run();
}
