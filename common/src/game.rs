#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Vec2 {
    pub x: isize;
    pub y: isize;
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum GameStage : u8 {
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
pub struct StaticGameInfo {
    // (x0 role x2 x3 x4)
    x0_raw: String,
    role: Role,
    x2_raw: String,
    x3_raw: String,
    x4_raw: String,
}

pub struct ShipsAndCommands {
    // TODO:
}

pub struct GameState {
    gameTick: usize,
    x1: String, //unknown
    shipsAndCommands: //
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GameResponse {
    
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GameRound {
    pub player_key: usize,
}

impl GameRound {
    pub fn new(player_key: &str) -> GameRound {
        GameRound {
            player_key: usize::from_str_radix(player_key, 10).unwrap(),
        }
    }
}
