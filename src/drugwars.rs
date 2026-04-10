// drugwars.rs 🦀
// 40th Anniversary Drugwars in Rust
// urrick hunt

use std::env;
use std::process;

pub mod ansi;
pub mod audio;
pub mod balance;
mod combat;
pub mod engine;
mod events;
mod flow;
pub mod input;
mod market;
pub mod model;
mod paths;
pub mod scoreboard;
pub mod state;
#[cfg(test)]
pub mod test_support;
pub mod ui;
pub mod view;

use self::model::GameState;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(windows)]
fn enable_ansi_support() {
    use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::winbase::STD_OUTPUT_HANDLE;

    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut mode: u32 = 0;
        if GetConsoleMode(handle, &raw mut mode) != 0 {
            let _ = SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
        }
    }
}

pub fn run() {
    #[cfg(windows)]
    enable_ansi_support();

    let args: Vec<String> = env::args().collect();
    if args.get(1).is_some_and(|arg| arg == "--version") {
        println!("drugwars.rs {VERSION}");
        process::exit(0);
    }

    let mut game = GameState::new();
    game.roll_prices();
    game.start_game();
}
