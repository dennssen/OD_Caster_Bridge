use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Deserialize, Serialize)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Deserialize, Serialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Deserialize, Serialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat
}

#[derive(Deserialize, Serialize)]
pub struct PlayerColor {
    pub primary: Color,
    pub secondary: Color,
}

#[derive(Deserialize, Serialize)]
pub struct Player {
    #[serde(rename = "playerId")]
    pub player_id: u16,
    #[serde(rename = "playerName")]
    pub player_name: String,
    pub root: Transform,
    pub head: Transform,
    #[serde(rename = "leftHand")]
    pub left_hand: Transform,
    #[serde(rename = "rightHand")]
    pub right_hand: Transform,
    pub velocity: Vec3,
    #[serde(rename = "teamIndex")]
    pub team_index: i8,
    #[serde(rename = "playerColor")]
    pub player_color: PlayerColor,
}

#[derive(Deserialize, Serialize)]
pub struct Spectator {
    #[serde(rename = "playerId")]
    pub player_id: u16,
    #[serde(rename = "playerName")]
    pub player_name: String,
    pub ping: u16,
    pub transform: Transform,
}

#[derive(Deserialize, Serialize)]
pub struct Ball {
    pub transform: Transform,
    pub velocity: Vec3,
}

#[derive(Deserialize, Serialize)]
pub struct GameState {
    pub players: Vec<Player>,
    pub spectators: Vec<Spectator>,
    pub balls: Vec<Ball>
}