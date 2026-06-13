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
    pub position: Vec3,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct TeamColor {
    pub primary: Color,
    pub secondary: Color,
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

impl GamemodeTeam {
    fn default_team(team_color: TeamColor) -> Self {
        Self {
            score: 0,
            rounds_won: 0,
            players: vec![],
            team_color
        }
    }

    pub fn home_team() -> Self {
        Self::default_team(TeamColor {
            primary: Color {
                r: 223,
                g: 45,
                b: 82,
                a: 255
            },
            secondary: Color {
                r: 223,
                g: 45,
                b: 82,
                a: 255
            }
        })
    }

    pub fn away_team() -> Self {
        Self::default_team(TeamColor {
            primary: Color {
                r: 9,
                g: 114,
                b: 213,
                a: 255
            },
            secondary: Color {
                r: 9,
                g: 114,
                b: 213,
                a: 255
            }
        })
    }
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

impl Default for GamemodeData {
    fn default() -> Self {
        Self {
            slot_id: String::from("gamma01"),
            time_seconds: 300.0,
            secondary_time_seconds: 0.0,
            is_game_running: false,
            total_rounds: 3,
            use_best_of: true,
            teams: vec![GamemodeTeam::home_team(), GamemodeTeam::away_team()]
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Stats {
    pub goals: i32,
    pub saves: i32,
    pub assists: i32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            goals: 0,
            saves: 0,
            assists: 0
        }
    }
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

impl Default for ShotInfo {
    fn default() -> Self {
        Self {
            shooter: String::new(),
            assister: String::new(),
            team: -1,
            shot_speed: 0.0,
            shot_distance_meters: 0.0
        }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct Round {
    pub home: i32,
    pub away: i32,
}

impl Default for Round {
    fn default() -> Self {
        Self {
            home: 0,
            away: 0
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct OverlayTeam {
    #[serde(deserialize_with = "deserialize_players")]
    pub players: IndexMap<String, Stats>,
}


impl OverlayTeam {
    pub fn home_team() -> Self {
        Self {
            players: IndexMap::from([
                ("Player1".to_string(), Stats::default()),
                ("Player2".to_string(), Stats::default()),
                ("Player3".to_string(), Stats::default()),
                ("Player4".to_string(), Stats::default()),
            ]),
        }
    }

    pub fn away_team() -> Self {
        Self {
            players: IndexMap::from([
                ("PlayerWithLongNameAsh".to_string(), Stats::default()),
                ("Player6".to_string(), Stats::default()),
                ("Player7".to_string(), Stats::default()),
                ("Player8".to_string(), Stats::default()),
            ]),
        }
    }
}

fn deserialize_players<'de, D>(deserializer: D) -> Result<IndexMap<String, Stats>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum PlayerOrArray {
        Players(IndexMap<String, Stats>),
        EmptyArray(Vec<()>),
    }

    match PlayerOrArray::deserialize(deserializer)? {
        PlayerOrArray::Players(map) => Ok(map),
        PlayerOrArray::EmptyArray(_vec) => Ok(IndexMap::new()),
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

impl Default for CameraApi {
    fn default() -> Self {
        Self {
            gamemode_id: String::new(),
            home: OverlayTeam::home_team(),
            away: OverlayTeam::away_team(),
            rounds: IndexMap::new(),
            followed_player: String::new(),
            last_shot_info: ShotInfo::default(),
            is_grace_period: false,
            is_overtime: false,
            best_of: 3,
            match_length_seconds: 300
        }
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct CameraApiUpdate {
    #[serde(rename = "gamemodeId", skip_serializing_if = "Option::is_none")]
    pub gamemode_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub home: Option<OverlayTeam>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub away: Option<OverlayTeam>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_rounds_opt",
        default
    )]
    pub rounds: Option<IndexMap<usize, Round>>,

    #[serde(rename = "followedPlayer", skip_serializing_if = "Option::is_none")]
    pub followed_player: Option<String>,

    #[serde(rename = "lastShotInfo", skip_serializing_if = "Option::is_none")]
    pub last_shot_info: Option<ShotInfo>,

    #[serde(rename = "isGracePeriod", skip_serializing_if = "Option::is_none")]
    pub is_grace_period: Option<bool>,

    #[serde(rename = "isOvertime", skip_serializing_if = "Option::is_none")]
    pub is_overtime: Option<bool>,

    #[serde(rename = "bestOf", skip_serializing_if = "Option::is_none")]
    pub best_of: Option<i32>,

    #[serde(rename = "matchLengthSeconds", skip_serializing_if = "Option::is_none")]
    pub match_length_seconds: Option<i32>,
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

fn serialize_rounds_opt<S>(
    rounds: &Option<IndexMap<usize, Round>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match rounds {
        Some(r) => serialize_rounds(r, serializer),
        None => unreachable!(), // skip_serializing_if handles this case
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CasterTeam {
    pub name: String,
    #[serde(rename = "logoUrl")]
    pub logo_url: String,
}

impl Default for CasterTeam {
    fn default() -> Self {
        Self {
            name: String::default(),
            logo_url: String::default(),
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
pub struct MatchData {
    pub host: String,
    pub name: String,
    pub stage: String,
}

impl Default for MatchData {
    fn default() -> Self {
        Self {
            host: String::default(),
            name: String::default(),
            stage: String::default()
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
    #[serde(rename = "matchData")]
    pub match_data: MatchData,
}