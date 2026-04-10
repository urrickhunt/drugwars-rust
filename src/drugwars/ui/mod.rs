mod dashboard;
mod startup;

use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use crate::ansi::CLEAR_SCREEN;

pub fn sentence_lines(text: &str) -> String {
    let mut formatted = String::with_capacity(text.len() + 4);
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        formatted.push(ch);

        if matches!(ch, '!' | '?' | '.') {
            while chars.next_if_eq(&ch).is_some() {
                formatted.push(ch);
            }

            let mut saw_spaces = false;
            while chars.next_if_eq(&' ').is_some() {
                saw_spaces = true;
            }

            if saw_spaces {
                match chars.peek() {
                    Some('\n') | None => {}
                    Some(_) => formatted.push('\n'),
                }
            }
        }
    }

    formatted
}

pub fn clear_screen() {
    print!("{CLEAR_SCREEN}");
    let _ = io::stdout().flush();
}

fn typewrite_line(text: &str, pad: usize, color_on: &str, color_off: &str, delay_ms: u64) {
    print!("{}", " ".repeat(pad));
    let _ = io::stdout().flush();

    for c in text.chars() {
        print!("{color_on}{c}{color_off}");
        let _ = io::stdout().flush();
        thread::sleep(Duration::from_millis(delay_ms));
    }

    println!();
    let _ = io::stdout().flush();
}

#[cfg(test)]
mod tests {
    use super::sentence_lines;

    #[test]
    fn sentence_lines_breaks_multi_sentence_dialogue() {
        assert_eq!(
            sentence_lines("SOMEONE TRIED TO MUG YOU !! BUT YOU FUCKED THEM WITH YOUR SHOTGUN !!"),
            "SOMEONE TRIED TO MUG YOU !!\nBUT YOU FUCKED THEM WITH YOUR SHOTGUN !!"
        );
        assert_eq!(
            sentence_lines(
                "COLOMBIAN FREIGHTER DUSTED THE COAST GUARD !!  WEED PRICES HAVE BOTTOMED OUT !!"
            ),
            "COLOMBIAN FREIGHTER DUSTED THE COAST GUARD !!\nWEED PRICES HAVE BOTTOMED OUT !!"
        );
        assert_eq!(
            sentence_lines("THERE IS SOME WEED THAT SMELLS LIKE PARAQUAT HERE !! IT LOOKS GOOD !!"),
            "THERE IS SOME WEED THAT SMELLS LIKE PARAQUAT HERE !!\nIT LOOKS GOOD !!"
        );
        assert_eq!(sentence_lines("CAFÉ !! ADIÓS !!"), "CAFÉ !!\nADIÓS !!");
    }
}
