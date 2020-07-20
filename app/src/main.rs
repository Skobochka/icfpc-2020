use std::env;

use common::game::{
    GameStage,
    GameRound,
    GameRequest,
};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();

    let server_url = &args[1];
    let player_key = &args[2];

    println!("Starting round with server: {} and player_key: {}", server_url, player_key);

    let mut game = GameRound::new(&format!("{}/aliens/send", server_url), player_key);


    game.run_request(GameRequest::JOIN)?;
    let mut full_state = game.run_request(GameRequest::START { x0: 0, x1: 0, x2: 0, x3: 1 })?;

    for turn in 0..255 {
        println!("[DEBUG] TURN {}", turn);
        if full_state.stage == GameStage::Finished {
            println!("[DEBUG] Game finished stage detected");
            break;
        }

        full_state = game.run_request(GameRequest::COMMANDS(vec![]))?
    }

    Ok(())
}
