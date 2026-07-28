use std::io::{self, Write};

#[cfg(unix)]
use std::io::Read;

#[cfg(unix)]
use {
    nix::libc::{POLLIN, STDIN_FILENO, poll, pollfd},
    nix::sys::termios::{FlushArg, LocalFlags, SetArg, cfmakeraw, tcflush, tcgetattr, tcsetattr},
};

#[cfg(windows)]
use crate::audio;
use crate::model::GameState;

#[cfg(windows)]
use std::ptr::{null, null_mut};

#[cfg(windows)]
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

#[cfg(windows)]
static CTRL_C_HANDLED: AtomicBool = AtomicBool::new(false);

#[cfg(windows)]
static CTRL_HANDLER_REGISTERED: AtomicBool = AtomicBool::new(false);

#[cfg(windows)]
static CTRL_EVENT_HANDLE: AtomicPtr<winapi::ctypes::c_void> = AtomicPtr::new(null_mut());

#[cfg(windows)]
pub fn disable_input_echo() {
    use winapi::shared::minwindef::DWORD;
    use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::winbase::STD_INPUT_HANDLE;
    use winapi::um::wincon::ENABLE_ECHO_INPUT;

    unsafe {
        let handle = GetStdHandle(STD_INPUT_HANDLE);
        let mut mode: DWORD = 0;
        if GetConsoleMode(handle, &raw mut mode) != 0 {
            let _ = SetConsoleMode(handle, mode & !ENABLE_ECHO_INPUT);
        }
    }
}

#[cfg(unix)]
pub fn disable_input_echo() {
    if let Ok(mut termios) = tcgetattr(STDIN_FILENO) {
        termios.local_flags &= !(LocalFlags::ECHO | LocalFlags::ECHONL | LocalFlags::ISIG);
        let _ = tcsetattr(STDIN_FILENO, SetArg::TCSANOW, &termios);
    }
}

#[cfg(windows)]
pub fn enable_input_echo() {
    use winapi::shared::minwindef::DWORD;
    use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::winbase::STD_INPUT_HANDLE;
    use winapi::um::wincon::ENABLE_ECHO_INPUT;

    unsafe {
        let handle = GetStdHandle(STD_INPUT_HANDLE);
        let mut mode: DWORD = 0;
        if GetConsoleMode(handle, &raw mut mode) != 0 {
            let _ = SetConsoleMode(handle, mode | ENABLE_ECHO_INPUT);
        }
    }
}

#[cfg(unix)]
pub fn enable_input_echo() {
    if let Ok(mut termios) = tcgetattr(STDIN_FILENO) {
        termios.local_flags |= LocalFlags::ECHO | LocalFlags::ECHONL | LocalFlags::ISIG;
        let _ = tcsetattr(STDIN_FILENO, SetArg::TCSANOW, &termios);
    }
}

#[cfg(windows)]
pub fn drain_stdin() {
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::winbase::STD_INPUT_HANDLE;
    use winapi::um::wincon::FlushConsoleInputBuffer;

    unsafe {
        let handle = GetStdHandle(STD_INPUT_HANDLE);
        FlushConsoleInputBuffer(handle);
    }
}

#[cfg(unix)]
pub fn drain_stdin() {
    let _ = tcflush(STDIN_FILENO, FlushArg::TCIFLUSH);
}

#[cfg(windows)]
unsafe extern "system" fn ctrl_handler(event: u32) -> i32 {
    use winapi::um::synchapi::SetEvent;

    const CTRL_C_EVENT: u32 = 0;
    const CTRL_BREAK_EVENT: u32 = 1;

    if event == CTRL_C_EVENT || event == CTRL_BREAK_EVENT {
        CTRL_C_HANDLED.store(true, Ordering::SeqCst);
        let ctrl_event = CTRL_EVENT_HANDLE.load(Ordering::SeqCst);
        if !ctrl_event.is_null() {
            SetEvent(ctrl_event);
        }
        1
    } else {
        audio::stop_music();
        0
    }
}

#[cfg(windows)]
fn windows_ctrl_event_handle() -> io::Result<winapi::um::winnt::HANDLE> {
    use winapi::shared::minwindef::FALSE;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::synchapi::CreateEventW;

    unsafe {
        let existing = CTRL_EVENT_HANDLE.load(Ordering::SeqCst);
        if !existing.is_null() {
            return Ok(existing);
        }

        let event = CreateEventW(null_mut(), FALSE, FALSE, null());
        if event.is_null() {
            return Err(io::Error::last_os_error());
        }

        match CTRL_EVENT_HANDLE.compare_exchange(
            null_mut(),
            event,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => Ok(event),
            Err(existing) => {
                CloseHandle(event);
                Ok(existing)
            }
        }
    }
}

#[cfg(windows)]
fn ensure_windows_ctrl_handler() {
    use winapi::um::consoleapi::SetConsoleCtrlHandler;

    unsafe {
        if !CTRL_HANDLER_REGISTERED.swap(true, Ordering::SeqCst) {
            SetConsoleCtrlHandler(Some(ctrl_handler), 1);
        }
    }
}

fn push_number_digit(digits: &mut String, value: &mut i64, digit: char) -> bool {
    let digit_value = i64::from(
        digit
            .to_digit(10)
            .expect("push_number_digit should only be called with decimal digits"),
    );

    value
        .checked_mul(10)
        .and_then(|next| next.checked_add(digit_value))
        .is_some_and(|next| {
            digits.push(digit);
            *value = next;
            true
        })
}

fn pop_number_digit(digits: &mut String, value: &mut i64) -> bool {
    if digits.pop().is_none() {
        return false;
    }

    *value /= 10;
    true
}

impl GameState {
    pub(crate) fn prompt_number(&self, prompt: &str) -> i64 {
        print!("{prompt}");
        let _ = io::stdout().flush();
        self.read_number_input()
    }

    pub(crate) fn wait_for_key_press(&self) {
        let _ = self.getch();
    }

    pub(crate) fn getch_key(&self) -> char {
        match self.getch() {
            Ok(c) => c,
            Err(error) => {
                eprintln!("\r\nterminal error: {error}");
                Self::quit(1);
            }
        }
    }

    fn read_number_input(&self) -> i64 {
        let mut digits = String::new();
        let mut value = 0_i64;

        loop {
            match self.getch_key() {
                digit @ '0'..='9' => {
                    if push_number_digit(&mut digits, &mut value, digit) {
                        print!("{digit}");
                        let _ = io::stdout().flush();
                    }
                }
                '\r' | '\n' => {
                    println!();
                    break;
                }
                '\x08' | '\x7f' => {
                    if pop_number_digit(&mut digits, &mut value) {
                        print!("\x08 \x08");
                        let _ = io::stdout().flush();
                    }
                }
                _ => {}
            }
        }

        value
    }

    fn getch(&self) -> io::Result<char> {
        #[cfg(unix)]
        {
            self.getch_unix()
        }

        #[cfg(windows)]
        {
            self.getch_windows()
        }
    }

    #[cfg(unix)]
    fn getch_unix(&self) -> io::Result<char> {
        let old_termios = tcgetattr(STDIN_FILENO).map_err(io::Error::other)?;
        let mut raw_termios = old_termios.clone();
        cfmakeraw(&mut raw_termios);
        tcsetattr(STDIN_FILENO, SetArg::TCSANOW, &raw_termios).map_err(io::Error::other)?;

        let mut buffer = [0; 1];
        let result = (|| -> io::Result<()> {
            {
                let stdin = io::stdin();
                let mut handle = stdin.lock();
                handle.read_exact(&mut buffer)?;

                if buffer[0] == b'\x1b' {
                    Self::drain_unix_escape_sequence(&mut handle)?;
                }

                drop(handle);
            }

            Ok(())
        })();
        let _ = tcsetattr(STDIN_FILENO, SetArg::TCSANOW, &old_termios);

        result?;

        if buffer[0] == 3 {
            self.finish_game();
        }

        Ok(buffer[0] as char)
    }

    #[cfg(unix)]
    fn drain_unix_escape_sequence(stdin: &mut impl Read) -> io::Result<()> {
        let mut pending = pollfd {
            fd: STDIN_FILENO,
            events: POLLIN,
            revents: 0,
        };
        let mut buffer = [0; 8];

        loop {
            pending.revents = 0;

            let ready = unsafe { poll(&raw mut pending, 1, 0) };
            if ready < 0 {
                return Err(io::Error::last_os_error());
            }
            if ready == 0 || pending.revents & POLLIN == 0 {
                return Ok(());
            }

            let read = stdin.read(&mut buffer)?;
            if read == 0 || read < buffer.len() {
                return Ok(());
            }
        }
    }

    #[cfg(windows)]
    fn getch_windows(&self) -> io::Result<char> {
        use winapi::shared::minwindef::DWORD;
        use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
        use winapi::um::handleapi::INVALID_HANDLE_VALUE;
        use winapi::um::processenv::GetStdHandle;
        use winapi::um::winbase::STD_INPUT_HANDLE;
        use winapi::um::wincon::{ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT};

        ensure_windows_ctrl_handler();
        let ctrl_event = windows_ctrl_event_handle()?;

        unsafe {
            let handle = GetStdHandle(STD_INPUT_HANDLE);
            if handle == INVALID_HANDLE_VALUE {
                return Err(io::Error::last_os_error());
            }

            let mut old_mode: DWORD = 0;
            if GetConsoleMode(handle, &raw mut old_mode) == 0 {
                return Err(io::Error::last_os_error());
            }

            let new_mode = old_mode & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT);
            if SetConsoleMode(handle, new_mode) == 0 {
                return Err(io::Error::last_os_error());
            }

            self.read_windows_key(handle, old_mode, ctrl_event)
        }
    }

    #[cfg(windows)]
    unsafe fn read_windows_key(
        &self,
        handle: winapi::um::winnt::HANDLE,
        old_mode: winapi::shared::minwindef::DWORD,
        ctrl_event: winapi::um::winnt::HANDLE,
    ) -> io::Result<char> {
        use winapi::shared::minwindef::FALSE;
        use winapi::um::consoleapi::{ReadConsoleInputW, SetConsoleMode};
        use winapi::um::synchapi::WaitForMultipleObjects;
        use winapi::um::winbase::{INFINITE, WAIT_FAILED, WAIT_OBJECT_0};
        use winapi::um::wincon::{INPUT_RECORD, KEY_EVENT, KEY_EVENT_RECORD};

        const WAIT_HANDLE_COUNT: u32 = 2;

        let mut input_record: INPUT_RECORD = std::mem::zeroed();
        let mut events_read = 0;
        let wait_handles = [handle, ctrl_event];

        loop {
            if CTRL_C_HANDLED.load(Ordering::SeqCst) {
                let _ = SetConsoleMode(handle, old_mode);
                self.finish_game();
            }

            match WaitForMultipleObjects(WAIT_HANDLE_COUNT, wait_handles.as_ptr(), FALSE, INFINITE)
            {
                WAIT_OBJECT_0 => {}
                signal if signal == WAIT_OBJECT_0 + 1 => {
                    let _ = SetConsoleMode(handle, old_mode);
                    self.finish_game();
                }
                WAIT_FAILED => {
                    let _ = SetConsoleMode(handle, old_mode);
                    return Err(io::Error::last_os_error());
                }
                _ => continue,
            }

            if ReadConsoleInputW(handle, &raw mut input_record, 1, &raw mut events_read) == 0 {
                let _ = SetConsoleMode(handle, old_mode);
                return Err(io::Error::last_os_error());
            }

            if input_record.EventType == KEY_EVENT && input_record.Event.KeyEvent().bKeyDown != 0 {
                let key_event: KEY_EVENT_RECORD = *input_record.Event.KeyEvent();
                #[allow(clippy::cast_possible_truncation)]
                let char_code = *key_event.uChar.UnicodeChar() as u8 as char;

                let _ = SetConsoleMode(handle, old_mode);

                if char_code == '\u{3}' {
                    self.finish_game();
                }

                return Ok(char_code);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{pop_number_digit, push_number_digit};

    #[test]
    fn number_input_ignores_digits_that_would_overflow_i64() {
        let mut digits = String::new();
        let mut value = 0_i64;

        for digit in i64::MAX.to_string().chars() {
            assert!(push_number_digit(&mut digits, &mut value, digit));
        }

        assert_eq!(value, i64::MAX);
        assert!(!push_number_digit(&mut digits, &mut value, '9'));
        assert_eq!(digits, i64::MAX.to_string());
        assert_eq!(value, i64::MAX);
    }

    #[test]
    fn number_input_backspace_keeps_value_and_digits_in_sync() {
        let mut digits = String::new();
        let mut value = 0_i64;

        for digit in ['0', '0', '7'] {
            assert!(push_number_digit(&mut digits, &mut value, digit));
        }

        assert_eq!(digits, "007");
        assert_eq!(value, 7);

        assert!(pop_number_digit(&mut digits, &mut value));
        assert_eq!(digits, "00");
        assert_eq!(value, 0);
    }
}
