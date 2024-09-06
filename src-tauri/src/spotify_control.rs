use log::{debug, error, info, trace, warn};
use windows::core::*;

use std::ffi::OsStr;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::*;
pub struct SpotifyControls {
    spotify_hwnd: HWND,
    system: System,
}
struct WindowData {
    target_pid: Vec<u32>,
    hwnd_found: HWND,
}
impl SpotifyControls {
    pub const SPOTIFY_PAUSED_WINDOW_TITLE: &'static str = "Spotify";
    const SPOTIFY_WINDOW_CLASS_NAME: &'static str = "Chrome_WidgetWin_1";

    pub fn new() -> Self {
        SpotifyControls {
            spotify_hwnd: HWND::default(),
            system: System::new(),
        }
    }

    pub fn get_spotify_hwnd(&mut self) {
        let mut s = System::new();

        s.refresh_specifics(
            RefreshKind::new().with_processes(
                ProcessRefreshKind::everything()
                    .without_cpu()
                    .without_memory()
                    .without_disk_usage(),
            ),
        );
        let spotify_process_name: &OsStr = OsStr::new("Spotify.exe");
        let mut spotify_pids: Vec<u32> = Vec::<u32>::new();
        for p in s.processes_by_name(spotify_process_name) {
            spotify_pids.push(p.pid().as_u32());
        }
        info!("Spotify PIDs: {:?}", spotify_pids);
        let mut window_data = WindowData {
            target_pid: spotify_pids,
            hwnd_found: self.spotify_hwnd,
        };
        let lparam = &mut window_data as *mut _ as isize;

        unsafe {
            let _ = EnumWindows(Some(Self::enum_windows_proc), LPARAM(lparam));
        };

        self.spotify_hwnd = window_data.hwnd_found;
        println!("{:?}", self.spotify_hwnd);
    }

    extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let spotify_window_class_name = "Chrome_WidgetWin_1".to_string();
        let windows_data = unsafe { &mut *(lparam.0 as *mut WindowData) };
        let target_pid = &windows_data.target_pid;
        let mut pid: u32 = 0;
        let mut class_name: Vec<u8> = vec![0; 256];

        unsafe {
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            GetClassNameA(hwnd, &mut class_name);
        }
        let mut title: [u16; 256] = [0; 256];
        let length = unsafe { GetWindowTextW(hwnd, &mut title) };

        if length > 0 {
            let title_str = String::from_utf16_lossy(&title[..length as usize]);
            let class_name_string = String::from_utf8_lossy(&class_name).to_string();
            debug!(
                "Found window with HWND: {:?}, Title: {} PID: {} Class: {}",
                hwnd, title_str, pid, class_name_string
            );

            if target_pid.contains(&pid) && unsafe { IsWindowVisible(hwnd).as_bool() } {
                let title_str = String::from_utf16_lossy(&title[..length as usize]);
                println!(
                    "Found Spotify window with HWND: {:?}, Title: {} PID: {}",
                    hwnd, title_str, pid
                );

                windows_data.hwnd_found = hwnd;

                return BOOL(0); // 見つけたので列挙を中断
            }
        }

        BOOL(1) // 列挙を続行
    }

    pub fn play(&self) -> bool {
        self.send_message_to_spotify(46 * 65536) // APPCOMMAND_MEDIA_PLAY
    }

    pub fn pause(&self) -> bool {
        // let value = (47 * 65536) as usize; 0x0000C001
        println!("{:?}", LPARAM(47 * 65536 as isize));
        self.send_message_to_spotify(47 * 65536) // APPCOMMAND_MEDIA_PAUSE
    }

    fn send_message_to_spotify(&self, command: i32) -> bool {
        if self.spotify_hwnd.is_invalid() {
            error!("spotify hwnd is invalid");
            return false;
        }
        info!("Sending message to Spotify {}", command);
        unsafe {
            SendMessageW(
                self.spotify_hwnd,
                0x0319,
                WPARAM(0),
                LPARAM(command as isize),
            );
        }
        true
    }

    fn get_title(&self, hwnd: HWND) -> String {
        let length = unsafe { GetWindowTextLengthW(hwnd) } + 1;
        let mut buffer: Vec<u16> = vec![0; length as usize];
        unsafe { GetWindowTextW(hwnd, &mut buffer) };
        let title = String::from_utf16_lossy(&buffer[..length as usize - 1]);
        let mut buf = vec![0u16; 256];
        unsafe {
            GetClassNameW(hwnd, &mut buf);
        }
        buf.retain(|&x| x != 0);
        let class_name = String::from_utf16_lossy(&buf);
        println!("title:{:?} | class name:{:?}", title, class_name);
        title
    }
}
pub fn is_spotify_running() -> bool {
    //sysinfoをつかってspotifyが起動しているか確認する
    let s = System::new_all();
    let spotify_process_name: &OsStr = OsStr::new("Spotify.exe");
    for _ in s.processes_by_name(spotify_process_name) {
        return true;
    }
    false
}
