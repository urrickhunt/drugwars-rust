use std::process;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

pub static MUSIC_HANDLE: OnceLock<Arc<Mutex<MusicPlayer>>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicState {
    Disabled,
    Playing,
    Paused,
}

enum Playback {
    Disabled,
    Playing(process::Child),
    Paused(process::Child),
}

pub struct MusicPlayer {
    playback: Playback,
    path: String,
}

impl MusicPlayer {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            playback: Playback::Disabled,
            path: path.into(),
        }
    }

    pub const fn state(&self) -> MusicState {
        match self.playback {
            Playback::Disabled => MusicState::Disabled,
            Playback::Playing(_) => MusicState::Playing,
            Playback::Paused(_) => MusicState::Paused,
        }
    }

    pub fn start(&mut self) {
        if !matches!(self.playback, Playback::Disabled) {
            return;
        }
        self.spawn();
    }

    pub fn toggle(&mut self) {
        match self.state() {
            MusicState::Disabled => self.start(),
            MusicState::Playing => self.pause(),
            MusicState::Paused => self.resume(),
        }
    }

    fn pause(&mut self) {
        let Playback::Playing(child) = std::mem::replace(&mut self.playback, Playback::Disabled)
        else {
            return;
        };

        Self::os_suspend(child.id());
        self.playback = Playback::Paused(child);
    }

    fn resume(&mut self) {
        let Playback::Paused(child) = std::mem::replace(&mut self.playback, Playback::Disabled)
        else {
            return;
        };

        Self::os_resume(child.id());
        self.playback = Playback::Playing(child);
    }

    fn kill(&mut self) {
        let playback = std::mem::replace(&mut self.playback, Playback::Disabled);
        if let Playback::Playing(mut child) | Playback::Paused(mut child) = playback {
            Self::os_resume(child.id());
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    pub fn respawn_if_needed(&mut self) {
        let dead = match &mut self.playback {
            Playback::Playing(child) => matches!(child.try_wait(), Ok(Some(_))),
            Playback::Disabled | Playback::Paused(_) => return,
        };

        if !dead {
            return;
        }

        let Playback::Playing(mut child) =
            std::mem::replace(&mut self.playback, Playback::Disabled)
        else {
            return;
        };

        let _ = child.wait();
        self.spawn();
    }

    fn spawn(&mut self) {
        match Self::launch_player(&self.path) {
            Some(child) => {
                self.playback = Playback::Playing(child);
            }
            None => {
                self.playback = Playback::Disabled;
            }
        }
    }

    #[cfg(unix)]
    fn os_suspend(pid: u32) {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        if let Ok(pid) = i32::try_from(pid) {
            let _ = kill(Pid::from_raw(pid), Signal::SIGSTOP);
        }
    }

    #[cfg(unix)]
    fn os_resume(pid: u32) {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        if let Ok(pid) = i32::try_from(pid) {
            let _ = kill(Pid::from_raw(pid), Signal::SIGCONT);
        }
    }

    #[cfg(windows)]
    fn os_suspend(pid: u32) {
        Self::call_nt_process_function(pid, c"NtSuspendProcess");
    }

    #[cfg(windows)]
    fn os_resume(pid: u32) {
        Self::call_nt_process_function(pid, c"NtResumeProcess");
    }

    #[cfg(windows)]
    fn call_nt_process_function(pid: u32, function_name: &'static std::ffi::CStr) {
        use winapi::shared::minwindef::FALSE;
        use winapi::shared::ntdef::NTSTATUS;
        use winapi::um::handleapi::CloseHandle;
        use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
        use winapi::um::processthreadsapi::OpenProcess;
        use winapi::um::winnt::{HANDLE, PROCESS_SUSPEND_RESUME};

        type NtProcessFunction = unsafe extern "system" fn(HANDLE) -> NTSTATUS;

        unsafe {
            let handle = OpenProcess(PROCESS_SUSPEND_RESUME, FALSE, pid);
            if !handle.is_null() {
                let ntdll = GetModuleHandleA(c"ntdll.dll".as_ptr());
                if !ntdll.is_null() {
                    let function = GetProcAddress(ntdll, function_name.as_ptr());
                    if !function.is_null() {
                        let function: NtProcessFunction = std::mem::transmute(function);
                        function(handle);
                    }
                }
                CloseHandle(handle);
            }
        }
    }

    #[cfg(unix)]
    fn launch_player(path: &str) -> Option<process::Child> {
        if let Ok(child) = process::Command::new("mpv")
            .args(["--no-video", "--loop=inf", "--really-quiet", path])
            .stdin(process::Stdio::null())
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .spawn()
        {
            return Some(child);
        }

        if let Ok(child) = process::Command::new("ffplay")
            .args([
                "-nodisp",
                "-autoexit",
                "-loop",
                "0",
                "-loglevel",
                "quiet",
                path,
            ])
            .stdin(process::Stdio::null())
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .spawn()
        {
            return Some(child);
        }

        None
    }

    #[cfg(windows)]
    fn launch_player(path: &str) -> Option<process::Child> {
        use std::os::windows::process::CommandExt;

        const CREATE_NO_WINDOW: u32 = 0x0800_0000;

        if let Ok(child) = process::Command::new("mpv")
            .args([
                "--no-video",
                "--loop=inf",
                "--really-quiet",
                "--no-terminal",
                path,
            ])
            .stdin(process::Stdio::null())
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
        {
            return Some(child);
        }

        if let Ok(child) = process::Command::new("ffplay")
            .args([
                "-nodisp",
                "-autoexit",
                "-loop",
                "0",
                "-loglevel",
                "quiet",
                path,
            ])
            .stdin(process::Stdio::null())
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
        {
            return Some(child);
        }

        None
    }
}

pub fn init_music() -> Arc<Mutex<MusicPlayer>> {
    let music = Arc::new(Mutex::new(MusicPlayer::new(extract_audio_file())));
    let _ = MUSIC_HANDLE.set(Arc::clone(&music));
    spawn_watchdog(&music);
    music
}

pub fn stop_music() {
    if let Some(music) = MUSIC_HANDLE.get() {
        if let Ok(mut player) = music.lock() {
            player.kill();
        }
    }
}

fn spawn_watchdog(music: &Arc<Mutex<MusicPlayer>>) {
    let music = Arc::clone(music);
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(500));
        if let Ok(mut player) = music.lock() {
            player.respawn_if_needed();
        }
    });
}

fn extract_audio_file() -> String {
    static MUSIC_BYTES: &[u8] = include_bytes!("../../assets/dopeman.opus");

    let path = std::env::temp_dir().join(format!("dopeman-{}.opus", env!("CARGO_PKG_VERSION")));
    let should_write = std::fs::metadata(&path)
        .map_or(true, |metadata| metadata.len() != MUSIC_BYTES.len() as u64);
    if should_write {
        if let Ok(mut file) = std::fs::File::create(&path) {
            use std::io::Write;
            let _ = file.write_all(MUSIC_BYTES);
        }
    }
    path.to_string_lossy().into_owned()
}
