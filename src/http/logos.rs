use std::{fs, io};
use std::path::{Path, PathBuf};
use crate::gui::app::AppState;
use crate::managers::appdata::AppData;
use crate::http;

pub enum Team {
    Home(String),
    Away(String)
}

pub fn get_and_upload_logo(team: Team, state: &mut AppState) {
    let logo = get_logo(&team);

    if logo.is_none() {
        return;
    }

    let logo_path = logo.unwrap();
    let logo_extension = if logo_path.extension().is_some() && logo_path.extension().unwrap().to_str().is_some() {
        logo_path.extension().unwrap().to_str().unwrap()
    } else {
        return;
    };

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let destination_filename = match &team {
        Team::Home(_) => {
            format!("home_{}.{}", timestamp, logo_extension)
        }
        Team::Away(_) => {
            format!("away_{}.{}", timestamp, logo_extension)
        }
    };

    let upload_destination_path = match &team {
        Team::Home(_) => {
            AppData::get_data_path().join("logos").join("home")
        }
        Team::Away(_) => {
            AppData::get_data_path().join("logos").join("away")
        }
    };

    if let Err(_) = empty_directory(&upload_destination_path) {
        return;
    }

    if let Err(_) = fs::copy(&logo_path, &upload_destination_path.join(&destination_filename)) {
        return;
    }

    match &team {
        Team::Home(_) => {
            state.game_state.caster_teams.home.logo_url = format!("http://{}/logos/home/{}", http::server::HTTP_ADDRESS, destination_filename)
        }
        Team::Away(_) => {
            state.game_state.caster_teams.away.logo_url = format!("http://{}/logos/away/{}", http::server::HTTP_ADDRESS, destination_filename)
        }
    };
}

fn empty_directory<P: AsRef<Path>>(path: P) -> io::Result<()> {
    fs::remove_dir_all(&path)?;
    fs::create_dir(&path)?;

    Ok(())
}

fn get_logo(team: &Team) -> Option<PathBuf> {
    let team_name = match team {
        Team::Home(name) => {
            if name.is_empty() {
                "home"
            } else {
                name.as_str()
            }
        }
        Team::Away(name) => {
            if name.is_empty() {
                "away"
            } else {
                name.as_str()
            }
        }
    };

    rfd::FileDialog::new()
        .add_filter("Logo", &["png", "jpg", "jpeg"])
        .set_title(format!("Pick {} logo", team_name))
        .pick_file()
}