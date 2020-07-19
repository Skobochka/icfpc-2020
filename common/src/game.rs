#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Vec2 {
    pub x: isize,
    pub y: isize,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum GameStage {
    NotStarted = 0,
    Started = 1,
    Finished = 2,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Role {
    Attacker,
    Defender,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GameStaticInfo {
    // (x0 role x2 x3 x4)
    pub x0_raw: String,
    pub role: Role,
    pub x2_raw: String,
    pub x3_raw: String,
    pub x4_raw: String,
}


#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Ship {
    pub role: Role,
    pub ship_id: isize,
    pub position: Vec2,
    pub velocity: Vec2,
    pub x4_raw: String,
    pub x5_raw: String,
    pub x6_raw: String,
    pub x7_raw: String,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Command {
    Accelerate { ship_id: isize, vec: Vec2 }, // (0, shipId, vector)
    Detonate { ship_id: isize }, // (1, shipId)
    Shoot { ship_id: isize, target: Vec2, x3_raw: String }, // (2, shipId, target, x3)
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GameState {
    pub game_tick: usize,
    pub x1: String, //unknown
    pub ships_n_commands: Vec<(Ship, Vec<Command>)>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GameResponse {
    pub stage: GameStage,
    pub static_info: GameStaticInfo,
    pub state: GameState,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GameRound {
    pub player_key: usize,
    pub last_response: Option<GameResponse>

}

impl GameRound {
    pub fn new(player_key: &str) -> GameRound {
        GameRound {
            player_key: usize::from_str_radix(player_key, 10).unwrap(),
            last_response: None,
        }
    }
}
