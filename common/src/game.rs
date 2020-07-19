use super::encoder::{
    ConsList,
    Modulable,
    Error as EncoderError,
};


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
    pub x1_raw: String, //unknown
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Error {
    ParsingError(EncoderError),
    ServerErrorReply,
}

impl GameRound {
    pub fn new(player_key: &str) -> GameRound {
        GameRound {
            player_key: usize::from_str_radix(player_key, 10).unwrap(),
            last_response: None,
        }
    }

    pub fn parse_game_state(resp: &ConsList) -> Result<GameState, Error> {
        let game_tick = resp.car().as_encoded_number().as_isize() as usize;
        let x1_raw = format!("{:?}", resp.cdr().as_cons().car());
        Ok(GameState {
            game_tick,
            x1_raw,
            ships_n_commands: vec![],
        })
    }

    pub fn parse_static_game_info(resp: &ConsList) -> Result<GameStaticInfo, Error> {
        let x0_raw = format!("{:?}", resp.car());

        let role_int = resp.cdr().as_cons().car().as_encoded_number().as_isize();

        let x2_raw = format!("{:?}", resp.cdr().as_cons().cdr().as_cons().car());
        let x3_raw = format!("{:?}", resp.cdr().as_cons().cdr().as_cons().cdr().as_cons().car());
        let x4_raw = format!("{:?}", resp.cdr().as_cons().cdr().as_cons().cdr().as_cons().cdr().as_cons().car());

        Ok(GameStaticInfo {
            x0_raw,
            role: match role_int {
                0 => Role::Attacker,
                1 => Role::Defender,
                _ => unreachable!(),
            },
            x2_raw,
            x3_raw,
            x4_raw,
        })
    }

    pub fn parse_game_response_from_string(input: &str) -> Result<GameResponse, Error> {
        let resp = ConsList::demodulate_from_string(input).map_err(|e| Error::ParsingError(e))?;

        GameRound::parse_game_response(&resp)
    }

    pub fn parse_game_response(resp: &ConsList) -> Result<GameResponse, Error> {
        let status = resp.car().as_encoded_number().as_isize();
        if status != 1 {
            return Err(Error::ServerErrorReply)
        }

        let stage_int = resp.cdr().as_cons().car().as_encoded_number().as_isize();
        let static_info = GameRound::parse_static_game_info(resp.cdr().as_cons().cdr().as_cons().car().as_cons())?;
        let state = GameRound::parse_game_state(resp.cdr().as_cons().cdr().as_cons().cdr().as_cons().car().as_cons())?;

        Ok(GameResponse {
            stage: match stage_int {
                0 => GameStage::NotStarted,
                1 => GameStage::Started,
                2 => GameStage::Finished,
                _ => unreachable!(),
            },
            state,
            static_info,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_response_parse_smoke() {
        let resp1 = "110110000111011000011111011110000100000000110110000111110111100001110000001101100001110111001000000001111011100001000011011101000000000110000111101011110111000010000110111010000000001111111101100001110101111011100011000010110001010001111010010111101011010110101101100001001101011011100100000011011000010011000011111101011011000011111101100011000001110001010001111010010111101110110010001101101010110110101011011000010011010110111001000000110110000100110000000000";
        assert_eq!(GameRound::parse_game_response_from_string(resp1).is_ok(), true);
    }
}
