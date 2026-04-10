use std::fmt::{self, Display, Formatter};

pub const RESET: &str = "\x1B[0m";
pub const CLEAR_SCREEN: &str = "\x1B[0m\x1B[3J\x1B[2J\x1B[1;1H";

pub const RED: &str = "\x1B[31m";
pub const GREEN: &str = "\x1B[32m";
pub const YELLOW: &str = "\x1B[33m";
pub const BLUE: &str = "\x1B[34m";
pub const MAGENTA: &str = "\x1B[35m";
pub const CYAN: &str = "\x1B[36m";
pub const BRIGHT_BLACK: &str = "\x1B[90m";

pub const BOLD_GREEN: &str = "\x1B[1;32m";
pub const DEVILED_EGGS: &str = "\x1B[38;2;255;202;128m";
pub const ASTER: &str = "\x1B[38;2;121;112;169m";

pub struct Painted<T> {
    color: &'static str,
    content: T,
}

pub const fn paint<T: Display>(color: &'static str, content: T) -> Painted<T> {
    Painted { color, content }
}

impl<T: Display> Display for Painted<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", self.color, self.content, RESET)
    }
}
