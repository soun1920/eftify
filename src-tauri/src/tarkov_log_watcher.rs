use crate::spotify_control::SpotifyControls;
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use log::{debug, info};
use regex::Regex;
use std::{
    ffi::OsStr,
    fs::OpenOptions,
    io::{self, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::mpsc::{self, Sender, TryRecvError},
    sync::{Arc, Mutex},
    {fs, thread, time},
};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use winreg::{enums::*, RegKey};

pub(crate) struct TarkovLogWatcher {
    eft_logs_location: String,
    app_new_log_tx: Sender<String>,
    app_new_log_rx: mpsc::Receiver<String>,
    backend_new_log_tx: Sender<String>,
    backend_new_log_rx: mpsc::Receiver<String>,
    app_log_bytes_read: Arc<Mutex<u64>>,
    backend_log_bytes_read: Arc<Mutex<u64>>,
    spotify: SpotifyControls,
    eft_exit_flag: bool,
}

impl TarkovLogWatcher {
    pub fn new() -> Self {
        let eft_logs_location = Self::get_log_path();
        let (app_new_log_tx, app_new_log_rx) = mpsc::channel();
        let (backend_new_log_tx, backend_new_log_rx) = mpsc::channel();
        let app_log_bytes_read: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
        let backend_log_bytes_read: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
        let spotify = SpotifyControls::new();
        let eft_exit_flag = false;

        TarkovLogWatcher {
            eft_logs_location,
            app_new_log_tx,
            app_new_log_rx,
            backend_new_log_tx,
            backend_new_log_rx,
            app_log_bytes_read,
            backend_log_bytes_read,
            spotify,
            eft_exit_flag,
        }
    }
    fn get_log_path() -> String {
        let reg_path: &str =
        "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\EscapeFromTarkov";
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let tarkov = hklm
            .open_subkey(&reg_path)
            .unwrap_or_else(|e| match e.kind() {
                io::ErrorKind::NotFound => panic!("EFT is not installed"),
                io::ErrorKind::PermissionDenied => panic!("Access denied"),
                _ => panic!("{:?}", e),
            });

        let path: String = tarkov.get_value("InstallLocation").unwrap_or_else(|e| {
            panic!("Failed to get InstallLocation: {:?}", e);
        });

        return path + "\\logs";
    }

    fn get_latest_log_folder(path: &String) -> Result<PathBuf, std::io::Error> {
        let logs_path = path.clone();
        let re = Regex::new(r"log_(?P<timestamp>\d+\.\d+\.\d+_\d+-\d+-\d+)").unwrap();
        let mut latest_date: i64 = 0;
        let mut latest_log_folder = String::new();
        let folders = fs::read_dir(logs_path)?;
        for log_folder in folders {
            let dir = log_folder?;
            let path = dir.path();
            let folder_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name,
                None => continue,
            };

            let captures = match re.captures(&folder_name) {
                Some(caps) => caps,
                None => continue,
            };

            let date_time_str = match captures.name("timestamp") {
                Some(timestamp) => timestamp.as_str(),
                None => continue,
            };

            let log_date = match NaiveDateTime::parse_from_str(date_time_str, "%Y.%m.%d_%H-%M-%S") {
                Ok(date) => DateTime::<Utc>::from_naive_utc_and_offset(date, Utc),
                Err(_) => continue,
            };

            let unix_time = log_date.timestamp();
            if unix_time > latest_date {
                latest_date = unix_time;
                latest_log_folder = folder_name.to_string();
            }
        }
        let mut latest_log_path = PathBuf::from(path);
        latest_log_path.push(latest_log_folder);
        return Ok(latest_log_path);
    }
    fn get_log_files<P: AsRef<Path>>(path: P) -> Result<(String, String), std::io::Error> {
        let files = fs::read_dir(path)?;
        let (mut application_log_name, mut backend_log_name) = (String::new(), String::new());
        for file in files {
            let file = file?;
            let path: PathBuf = file.path();
            let file_name = match path.file_name().and_then(|t| t.to_str()) {
                Some(name) => name,
                None => continue,
            };
            if file_name.contains("application.log") {
                application_log_name = file_name.to_string()
            }
            if file_name.contains("backend.log") {
                backend_log_name = file_name.to_string()
            }
        }
        Ok((application_log_name, backend_log_name))
    }

    fn read_log_file(path: PathBuf, new_log_tx: Sender<String>, file_bytes: Arc<Mutex<u64>>) {
        let log_path = path;
        let new_log_data: Sender<String> = new_log_tx;

        let file_bytes_rea = Arc::clone(&file_bytes);
        thread::spawn(move || {
            let mut file_bytes_read = file_bytes_rea.lock().unwrap();
            let meta = match fs::metadata(&log_path) {
                Ok(metadata) => metadata,
                Err(e) => panic!("{}", e),
            };
            let file_size = meta.len();
            debug!("file size: {}", file_size);
            debug!("file bytes read: {:?}", *file_bytes_read);
            if file_size > *file_bytes_read {
                match OpenOptions::new()
                    .read(true)
                    .open(&log_path)
                    .and_then(|mut f| {
                        f.seek(SeekFrom::Start(*file_bytes_read))?;
                        let mut buffer = vec![0; 1024];
                        let mut chunks = Vec::new();
                        let mut new_bytes_read = 0;
                        loop {
                            match f.read(&mut buffer) {
                                Ok(0) => break, // end of file
                                Ok(bytes_read) => {
                                    new_bytes_read += bytes_read as u64;
                                    chunks.push(
                                        String::from_utf8_lossy(&buffer[..bytes_read]).to_string(),
                                    );
                                }
                                Err(e) => return Err(e),
                            }
                        }
                        *file_bytes_read += new_bytes_read;
                        let data = chunks.concat();
                        let _ = new_log_data.send(data);
                        // Here you would invoke the InitialReadComplete event if necessary
                        Ok(())
                    }) {
                    Ok(_) => (),
                    Err(e) => panic!("{}", e),
                }
            }
        });
    }

    pub fn watch_logs(&mut self) -> std::io::Result<()> {
        let mut system = System::new();
        system.refresh_specifics(
            RefreshKind::new().with_processes(
                ProcessRefreshKind::everything()
                    .without_cpu()
                    .without_memory()
                    .without_disk_usage(),
            ),
        );
        let is_eft_running = self.check_eft_running(&system);
        if is_eft_running || self.check_spotify_running(&system) {
            info!("EFT is running");
            self.spotify.get_spotify_hwnd();

            if self.eft_exit_flag {
                info!("eft exit flag is true");
                self.init_bytes_read();
                self.eft_exit_flag = false;
                thread::sleep(time::Duration::from_secs(15));
            }
            let latest_log_path = Self::get_latest_log_folder(&self.eft_logs_location)?;
            let (app_log_name, backend_log_name) = Self::get_log_files(latest_log_path.clone())?;
            info!("latest log path: {:?}", latest_log_path);
            self.process_log(
                latest_log_path.clone(),
                app_log_name,
                latest_log_path,
                backend_log_name,
            );
            info!(
                "app bytes read {}",
                *self.app_log_bytes_read.lock().unwrap()
            );
            info!(
                "backend bytes read {}",
                *self.backend_log_bytes_read.lock().unwrap()
            );
        }
        if !is_eft_running {
            self.eft_exit_flag = true;
        }

        Ok(())
    }
    fn init_bytes_read(&self) {
        let mut app_log_bytes_read = self.app_log_bytes_read.lock().unwrap();
        *app_log_bytes_read = 0;
        let mut backend_log_bytes_read = self.backend_log_bytes_read.lock().unwrap();
        *backend_log_bytes_read = 0;
    }
    fn check_eft_running(&self, s: &System) -> bool {
        let eft_process_name: &OsStr = OsStr::new("EscapeFromTarkov.exe");
        for _ in s.processes_by_name(eft_process_name) {
            return true;
        }
        false
    }
    fn check_spotify_running(&self, s: &System) -> bool {
        let eft_process_name: &OsStr = OsStr::new("EscapeFromTarkov.exe");
        for _ in s.processes_by_name(eft_process_name) {
            return true;
        }
        false
    }

    fn process_log(
        &self,
        mut app_path: PathBuf,
        app_log_name: String,
        mut backend_path: PathBuf,
        backend_log_name: String,
    ) {
        let app_log_bytes = self.app_log_bytes_read.clone();
        app_path.push(app_log_name);
        Self::read_log_file(app_path, self.app_new_log_tx.clone(), app_log_bytes);
        let new_data_app = match self.app_new_log_rx.try_recv() {
            Ok(new_data_app) => new_data_app,
            Err(TryRecvError::Empty) => "".to_string(),
            Err(TryRecvError::Disconnected) => {
                panic!("channel disconnected")
            }
        };
        let backend_log_bytes = self.backend_log_bytes_read.clone();
        backend_path.push(backend_log_name);
        Self::read_log_file(
            backend_path,
            self.backend_new_log_tx.clone(),
            backend_log_bytes,
        );
        let new_data_backend = match self.backend_new_log_rx.try_recv() {
            Ok(new_data_backend) => new_data_backend,
            Err(TryRecvError::Empty) => "".to_string(),
            Err(TryRecvError::Disconnected) => {
                panic!("channel disconnected")
            }
        };
        let dt = Local::now();
        if new_data_app.contains("application|Application awaken") {
            info!("spotify control skipped");
            return;
        }
        if new_data_app.contains("application|GameStarted") {
            info!("spotify paused");
            self.spotify.pause();
        }
        if new_data_backend
            .contains("escapefromtarkov.com/client/putMetrics, crc: , responseText: .")
        {
            info!("spotify playing");
            self.spotify.play();
        }
    }
}
