use std::{fs, io};
use std::path::PathBuf;
use std::sync::OnceLock;

pub struct AppData {

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
        if !data_path.exists() {
            if Self::create_data(data_path).is_err() {
                panic!("Could not create AppData. This is necessary for the app to work, please report this to a dev");
            }
        }

        Self {}
    }

    fn create_data(data_path: &PathBuf) -> io::Result<()> {
        fs::create_dir_all(data_path)?;

        fs::create_dir(data_path.join("logos"))?;

        Ok(())
    }
}