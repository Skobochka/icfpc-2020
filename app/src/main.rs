use std::env;
use tokio::runtime::Runtime;

use common::game::{
    GameRound,
};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();

    let server_url = &args[1];
    let player_key = &args[2];

    println!("Starting round with server: {} and player_key: {}", server_url, player_key);

    let mut game = GameRound::new(&format!("{}/aliens/send", server_url), player_key);


    println!("[DEBUG] JOIN");
    let join_request = game.make_join_request();
    game.run_request(join_request);

    println!("[DEBUG] START");
    let start_request = game.make_start_request(0, 0, 0, 1);
    game.run_request(start_request);

    for turn in 0..255 {
        println!("[DEBUG] COMMAND {}", turn);
        
        let commands_request = game.make_commands_request(vec![]);
        game.run_request(commands_request);
    }

    Ok(())
}
