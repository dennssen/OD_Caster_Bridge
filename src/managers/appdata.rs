use std::{fs, io};
use std::path::PathBuf;
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct AppData {
    pub camera_api_id: String,
    pub poll_interval_fps: u64,
    pub poll_game_data: bool,
    pub poll_gamemodes: bool,
    pub poll_cameras: bool,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            camera_api_id: "dennssen.caster".to_string(),
            poll_interval_fps: 60,
            poll_game_data: false,
            poll_gamemodes: true,
            poll_cameras: true
        }
    }
}

static DATA_PATH: OnceLock<PathBuf> = OnceLock::new();

impl AppData {
    pub fn get_data_path() -> &'static PathBuf {
        DATA_PATH.get_or_init(|| {
            dirs::data_dir().unwrap().join("ODCasterBridge")
        })
    }

    pub fn get_or_init() -> Self {
        let data_path = Self::get_data_path();
        if Self::try_create_data(data_path).is_err() {
            panic!("Could not create AppData. This is necessary for the app to work, please report this to a dev");
        }

        let app_data_string = match fs::read_to_string(data_path.join("data.json")) {
            Ok(string) => {
                string
            }
            Err(_) => {
                panic!("Could not read data.json.");
            }
        };

        match serde_json::from_str::<AppData>(app_data_string.as_str()) {
            Ok(app_data) => {
                app_data
            }
            Err(_) => {
                Self::default()
            }
        }
    }

    pub fn save(&self) {
        let app_data_path = Self::get_data_path().join("data.json");

        if let Ok(app_data_string) = serde_json::to_string(self) {
            if let Err(_) = fs::write(app_data_path, app_data_string) {
                println!("Failed to write to data.json");
            }
        } else {
            println!("Failed to serialize app data");
        }
    }

    fn try_create_data(data_path: &PathBuf) -> io::Result<()> {
        if !data_path.exists() {
            fs::create_dir_all(data_path)?;
        }

        let logos_path = data_path.join("logos");

        if !logos_path.exists() {
            fs::create_dir(&logos_path)?;
        }

        let home_path = logos_path.join("home");
        if !home_path.exists() {
            fs::create_dir(&home_path)?;
        }

        let away_path = logos_path.join("away");
        if !away_path.exists() {
            fs::create_dir(&away_path)?;
        }

        let app_data_file = data_path.join("data.json");
        if !app_data_file.exists() {
            fs::write(app_data_file, "")?;
        }

        Ok(())
    }
}