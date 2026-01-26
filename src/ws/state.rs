use std::collections::HashMap;
use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Deserialize, Serialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Player {
    #[serde(rename = "playerId")]
    pub player_id: u16,
    #[serde(rename = "playerName")]
    pub player_name: String,
    pub head: Transform,
    pub velocity: Vec3,
    #[serde(rename = "teamIndex")]
    pub team_index: i8,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Spectator {
    #[serde(rename = "playerId")]
    pub player_id: u16,
    #[serde(rename = "playerName")]
    pub player_name: String,
    pub ping: u16,
    pub transform: Transform,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Ball {
    pub transform: Transform,
    pub velocity: Vec3,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GameData {
    pub players: Vec<Player>,
    pub spectators: Vec<Spectator>,
    pub balls: Vec<Ball>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SimpleGamemode {
    #[serde(rename = "slotId")]
    pub slot_id: String,
    pub position: Vec3,
    pub rotation: Quat,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SimplePlayer {
    #[serde(rename = "playerId")]
    pub player_id: u16,
    #[serde(rename = "playerName")]
    pub player_name: String,
    position: Vec3,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GamemodeTeam {
    pub score: i8,
    #[serde(rename = "roundsWon")]
    pub rounds_won: i16,
    pub players: Vec<SimplePlayer>,
    #[serde(rename = "teamColor")]
    pub team_color: TeamColor,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct TeamColor {
    pub primary: Color,
    pub secondary: Color,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GamemodeData {
    #[serde(rename = "slotId")]
    pub slot_id: String,
    #[serde(rename = "timeSeconds")]
    pub time_seconds: f32,
    #[serde(rename = "secondaryTimeSeconds")]
    pub secondary_time_seconds: f32,
    #[serde(rename = "isGameRunning")]
    pub is_game_running: bool,
    #[serde(rename = "totalRounds")]
    pub total_rounds: i8,
    #[serde(rename = "useBestOf")]
    pub use_best_of: bool,
    pub teams: Vec<GamemodeTeam>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Stats {
    pub goals: i32,
    pub saves: i32,
    pub assists: i32,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ShotInfo {
    pub shooter: String,
    pub assister: String,
    pub team: i32,
    #[serde(rename = "shotSpeed")]
    pub shot_speed: f64,
    #[serde(rename = "shotDistanceMeters")]
    pub shot_distance_meters: f64,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct Round {
    pub home: i32,
    pub away: i32,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct OverlayTeam {
    #[serde(deserialize_with = "deserialize_players")]
    pub players: HashMap<String, Stats>,
}

fn deserialize_players<'de, D>(deserializer: D) -> Result<HashMap<String, Stats>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum PlayerOrArray {
        Players(HashMap<String, Stats>),
        EmptyArray(Vec<()>),
    }

    match PlayerOrArray::deserialize(deserializer)? {
        PlayerOrArray::Players(map) => Ok(map),
        PlayerOrArray::EmptyArray(_vec) => Ok(HashMap::new()),
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CameraApi {
    #[serde(rename = "gamemodeId")]
    pub gamemode_id: String,
    pub home: OverlayTeam,
    pub away: OverlayTeam,
    #[serde(
        deserialize_with = "deserialize_rounds",
        serialize_with = "serialize_rounds"
    )]
    pub rounds: IndexMap<usize, Round>,
    #[serde(rename = "followedPlayer")]
    pub followed_player: String,
    #[serde(rename = "lastShotInfo")]
    pub last_shot_info: ShotInfo,
    #[serde(rename = "isGracePeriod")]
    pub is_grace_period: bool,
    #[serde(rename = "isOvertime")]
    pub is_overtime: bool,
    #[serde(rename = "bestOf")]
    pub best_of: i32,
    #[serde(rename = "matchLengthSeconds")]
    pub match_length_seconds: i32,
}

fn deserialize_rounds<'de, D>(deserializer: D) -> Result<IndexMap<usize, Round>, D::Error>
where
    D: Deserializer<'de>,
{
    let rounds = Vec::<Round>::deserialize(deserializer)?;
    Ok(rounds.into_iter().enumerate().collect())
}

fn serialize_rounds<S>(map: &IndexMap<usize, Round>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let items: Vec<&Round> = map.values().collect();
    items.serialize(serializer)
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CasterTeam {
    pub name: String,
    pub logo: String,
}

impl Default for CasterTeam {
    fn default() -> Self {
        Self {
            name: String::default(),
            logo: String::default(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CasterTeams {
    pub home: CasterTeam,
    pub away: CasterTeam,
}

impl Default for CasterTeams {
    fn default() -> Self {
        Self {
            home: CasterTeam::default(),
            away: CasterTeam::default(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GameState {
    #[serde(rename = "gameData")]
    pub game_data: Option<GameData>,
    pub gamemodes: Vec<SimpleGamemode>,
    #[serde(rename = "selectedGamemode")]
    pub selected_gamemode: Option<GamemodeData>,
    pub cameras: Vec<String>,
    #[serde(rename = "cameraApi")]
    pub camera_api: Option<CameraApi>,
    #[serde(rename = "casterTeams")]
    pub caster_teams: CasterTeams,
}