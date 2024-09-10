use serde::{Deserialize, Serialize};
use serde_json::json;

use std::fs;
use std::path::PathBuf;
use tauri::api::path::app_data_dir;
use tauri_plugin_store::StoreBuilder;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    auto_start: bool,
    dark_theme: bool,
}
impl Default for Config {
    fn default() -> Self {
        auto_start: "en".to_string(),
        dark_theme: "dark".to_string(),
    }
}
impl Config {
    // デフォルトの設定
    pub fn new() -> Self {
        let config_file = get_config_root().join("config.json");
        if config
        Config {
            auto_start: false,
            dark_theme: false,
        }
    }
}

fn get_config_root() -> PathBuf {
    let appdata = PathBuf::from(std::env::var("APPDATA").unwrap());
    appdata.join("takanori").join("myapp")
}
fn save_config(config: &Config) -> Result<(), std::io::Error> {
    // APPDATA 配下のディレクトリを取得
    let config_dir = app_data_dir().expect("APPDATA ディレクトリが見つかりません");

    // 設定ファイルのパス
    let config_file_path = config_dir.join("config.json");

    // ディレクトリが存在しない場合は作成
    if !config_file_path.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    // 設定を JSON としてファイルに保存
    let config_json = serde_json::to_string_pretty(&config)?;
    fs::write(config_file_path, config_json)?;

    Ok(())
}

fn load_config() -> Result<Config, std::io::Error> {
    // APPDATA 配下のディレクトリを取得
    let config_dir = app_data_dir().expect("APPDATA ディレクトリが見つかりません");

    // 設定ファイルのパス
    let config_file_path = config_dir.join("config.json");

    // ファイルが存在すれば読み込む
    if config_file_path.exists() {
        let config_json = fs::read_to_string(config_file_path)?;
        let config: Config = serde_json::from_str(&config_json)?;
        Ok(config)
    } else {
        // 存在しない場合はデフォルトの設定を返す
        Ok(Config::default())
    }
}

#[tauri::command]
fn save_user_config(volume: u8) -> Result<(), String> {
    let config = Config { volume };
    save_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_user_config() -> Result<Config, String> {
    load_config().map_err(|e| e.to_string())
}
