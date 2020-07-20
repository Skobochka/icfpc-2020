use super::encoder::{
    ConsList,
    ListVal,
    Modulable,
    PrettyPrintable,
    Error as EncoderError,
};

use super::code::{
    make_mod_number,
};

use super::send::{
    Intercom,
};
use tokio::runtime::Runtime;

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
    pub x4_raw: String, // looks like (x0 x1 x2 x3) parameters to START!
    pub x5_raw: String,
    pub x6_raw: String,
    pub x7_raw: String,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Command {
    Accelerate { vec: Vec2 }, // (0, shipId, vector)
    Detonate, // (1, shipId)
    Shoot { target: Vec2, x3_raw: String }, // (2, shipId, target, x3)
    UnknownCommand { cmd_id: isize, raw: String },
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

pub struct GameRound {
    intercom: Intercom,
    runtime: Runtime,
    pub player_key: isize,
    pub last_response: Option<GameResponse>

}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Error {
    ParsingError(EncoderError),
    ServerErrorReply,
}

impl GameRound {
    pub fn new(api_url: &str, player_key: &str) -> GameRound {
        GameRound {
            player_key: isize::from_str_radix(player_key, 10).unwrap(),
            last_response: None,
            intercom: Intercom::new(api_url.to_string()),
            runtime: Runtime::new().unwrap(),
        }
    }

    pub fn run_request(&mut self, request: String) -> String {
        println!("-> {}", &request);
        println!("[DEBUG] => : {}", GameRound::format_modulated(&request));
        let response = self.intercom.send(request, &mut self.runtime).unwrap();
        println!("<- {}", &response);
        println!("[DEBUG] <= : {}", GameRound::format_modulated(&response));
        response
    }
    
    pub fn format_modulated(modulated: &str) -> String {
        ConsList::demodulate_from_string(&modulated).unwrap().to_pretty_string()
    }

    pub fn make_join_request(&self) -> String {
        // JOIN request
        // (2, playerKey, (...unknown list...))

        // assuming unknown list as nil for now
        let unknown_param = ListVal::Cons(Box::new(ConsList::Nil));
        
        let request = ConsList::Cons(
            ListVal::Number(make_mod_number(2)),
            ListVal::Cons(Box::new(ConsList::Cons(
                ListVal::Number(make_mod_number(self.player_key)),
                ListVal::Cons(Box::new(ConsList::Cons(
                    unknown_param,
                    ListVal::Cons(Box::new(ConsList::Nil)))))))));

        request.modulate_to_string()
    }

    pub fn make_start_request(&self, x0: isize, x1: isize, x2: isize, x3: isize) -> String {
        // START
        // (3, playerKey, (x0, x1, x2, x3))
        // The third item of this list is always a list of 4 numbers – it’s the initial ship parameters.
        // We noticed, that START doesn’t finish successfully when x3 is 0 or xi’s are too large.
                
        let request = ConsList::Cons(
            ListVal::Number(make_mod_number(3)),
            
            ListVal::Cons(Box::new(ConsList::Cons(
                ListVal::Number(make_mod_number(self.player_key)),
                
                ListVal::Cons(Box::new(ConsList::Cons(
                    
                    ListVal::Cons(Box::new(ConsList::Cons(
                        ListVal::Number(make_mod_number(x0)),
                        ListVal::Cons(Box::new(ConsList::Cons(
                            ListVal::Number(make_mod_number(x1)),
                            ListVal::Cons(Box::new(ConsList::Cons(
                                ListVal::Number(make_mod_number(x2)),
                                ListVal::Cons(Box::new(ConsList::Cons(
                                    ListVal::Number(make_mod_number(x3)),
                                    ListVal::Cons(Box::new(ConsList::Nil)))))))))))))),

                    ListVal::Cons(Box::new(ConsList::Nil)))))))));

        request.modulate_to_string()
    }

    pub fn make_commands_request(&self, _cmds: Vec<(&Ship, Command)>) -> String {
        // COMMANDS
        // (4, playerKey, commands)
        // commands is the list of issued commands.
        // Each item has format (type, shipId, ...), where ... denotes command-specific parameters.

        // assuming unknown list as nil for now
        let request = ConsList::Cons(
            ListVal::Number(make_mod_number(4)),

            ListVal::Cons(Box::new(ConsList::Cons(
                ListVal::Number(make_mod_number(self.player_key)),

                ListVal::Cons(Box::new(ConsList::Cons(

                    ListVal::Cons(Box::new(ConsList::Nil)), // here comes command list
                    
                    ListVal::Cons(Box::new(ConsList::Nil)))))))));

        request.modulate_to_string()
    }

    pub fn parse_ship(resp: &ConsList) -> Result<Ship, Error> {
        let role_int = resp.car().as_encoded_number().as_isize();
        let ship_id = resp.cdr().as_cons().car().as_encoded_number().as_isize();
        let position_enc = resp.cdr().as_cons().cdr().as_cons().car().as_tuple();
        let velocity_enc = resp.cdr().as_cons().cdr().as_cons().cdr().as_cons().car().as_tuple();

        let x4_raw = resp.cdr().as_cons().cdr().as_cons().cdr().as_cons().cdr().as_cons().car().to_pretty_string();
        let x5_raw = resp.cdr().as_cons().cdr().as_cons().cdr().as_cons().cdr().as_cons().cdr().as_cons().car().to_pretty_string();
        let x6_raw = resp.cdr().as_cons().cdr().as_cons().cdr().as_cons().cdr().as_cons().cdr().as_cons().cdr().as_cons().car().to_pretty_string();
        let x7_raw = resp.cdr().as_cons().cdr().as_cons().cdr().as_cons().cdr().as_cons().cdr().as_cons().cdr().as_cons().cdr().as_cons().car().to_pretty_string();

        Ok(Ship {
            role: match role_int {
                0 => Role::Attacker,
                1 => Role::Defender,
                _ => unreachable!(),
            },
            ship_id,
            position: Vec2 { x: position_enc.0.as_isize(), y: position_enc.1.as_isize() },
            velocity: Vec2 { x: velocity_enc.0.as_isize(), y: velocity_enc.1.as_isize() },
            x4_raw,
            x5_raw,
            x6_raw,
            x7_raw,
        })
    }

    pub fn parse_command(resp: &ConsList) -> Result<Command, Error> {
        let cmd_id = resp.car().as_encoded_number().as_isize();
        match cmd_id {
            0 => {
                let vec_enc = resp.cdr().as_cons().car().as_tuple();

                Ok(Command::Accelerate { vec: Vec2 { x: vec_enc.0.as_isize(), y: vec_enc.1.as_isize() } })
            },
            1 => Ok(Command::Detonate),
            2 => {
                let vec_enc = resp.cdr().as_cons().car().as_tuple();
                let x3_raw = resp.cdr().as_cons().cdr().as_cons().cdr().as_cons().car().to_pretty_string();

                Ok(Command::Shoot { target: Vec2 { x: vec_enc.0.as_isize(), y: vec_enc.1.as_isize() },
                                    x3_raw })
            },
            _ => Ok(Command::UnknownCommand { cmd_id, raw: resp.to_pretty_string() }),
        }
    }

    pub fn parse_ships_n_commands(resp: &ConsList) -> Result<Vec<(Ship, Vec<Command>)>, Error> {
        let mut result = Vec::<(Ship, Vec<Command>)>::new();
        let mut iter = resp;

        loop {
            match iter {
                ConsList::Nil => break,
                ConsList::Cons(head, tail) => {
                    let ship = GameRound::parse_ship(head.as_cons().car().as_cons())?;
                    let mut commands = Vec::<Command>::new();
                    let mut cmd_iter = head.as_cons().cdr().as_cons().car().as_cons();
                    loop {
                        match cmd_iter {
                            ConsList::Nil => break,
                            ConsList::Cons(cmd_head, cmd_tail) => {
                                let command = GameRound::parse_command(cmd_head.as_cons())?;
                                commands.push(command);
                                cmd_iter = cmd_tail.as_cons();
                            }
                        }
                    }
                    result.push((ship, commands));
                    iter = tail.as_cons();
                }
            }
        }
        Ok(result)
    }

    pub fn parse_game_state(resp: &ConsList) -> Result<GameState, Error> {
        let game_tick = resp.car().as_encoded_number().as_isize() as usize;
        let x1_raw = resp.cdr().as_cons().car().to_pretty_string();
        let ships_n_commands = GameRound::parse_ships_n_commands(resp.cdr().as_cons().cdr().as_cons().car().as_cons())?;
        Ok(GameState {
            game_tick,
            x1_raw,
            ships_n_commands,
        })
    }

    pub fn parse_static_game_info(resp: &ConsList) -> Result<GameStaticInfo, Error> {
        let x0_raw = resp.car().to_pretty_string();

        let role_int = resp.cdr().as_cons().car().as_encoded_number().as_isize();

        let x2_raw = resp.cdr().as_cons().cdr().as_cons().car().to_pretty_string();
        let x3_raw = resp.cdr().as_cons().cdr().as_cons().cdr().as_cons().car().to_pretty_string();
        let x4_raw = resp.cdr().as_cons().cdr().as_cons().cdr().as_cons().cdr().as_cons().car().to_pretty_string();

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
        let resp2 = "1101100001110110000111110111100001000000001101100001111101111000011100000011011000011101110010000000011110111000010000110111010000000001100001111011000111111011100001000011011101000000000111111110110000111010111110110001010101010111011110110001101011110101101011010110110000100110101101110010000001101100001001100001111110101101100001111101110001001000111000010100111110100110011000111111011100011110111011100100000011011010101101100001001101011011100100000011011000010011111101011110110000110100001000000000000";
        assert_eq!(GameRound::parse_game_response_from_string(resp2).is_ok(), true);
    }

    #[test]
    fn round_requests_composer() {
        let round = GameRound::new("", "1");
        
        assert_eq!(round.make_join_request(), "11011000101101100001110000");
        assert_eq!(round.make_start_request(0, 0, 0, 0), "1101100011110110000111110101101011010110100000");
        assert_eq!(round.make_commands_request(vec![]), "11011001001101100001110000");
    }
}
