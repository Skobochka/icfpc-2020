use std::env;
use tokio::runtime::Runtime;

use common::send::{
    Intercom,
};

use common::code::{
    make_mod_number,
};

use common::encoder::{
    ConsList,
    ListVal,
    Modulable,
    PrettyPrintable,
};

fn make_join_request(key_str: &str) -> String {
    // JOIN request
    // (2, playerKey, (...unknown list...))
    let key = usize::from_str_radix(key_str, 10).unwrap();

    // assuming unknown list as nil for now
    let unknown_param = ListVal::Cons(Box::new(ConsList::Nil));

    let request = ConsList::Cons(
        ListVal::Number(make_mod_number(2)),
        ListVal::Cons(Box::new(ConsList::Cons(
            ListVal::Number(make_mod_number(key as isize)),
            ListVal::Cons(Box::new(ConsList::Cons(
                unknown_param,
                ListVal::Cons(Box::new(ConsList::Nil)))))))));

    request.modulate_to_string()
}

fn make_start_request(key_str: &str, x0: isize, x1: isize, x2: isize, x3: isize) -> String {
    // START
    // (3, playerKey, (x0, x1, x2, x3))
    // The third item of this list is always a list of 4 numbers – it’s the initial ship parameters.
    // We noticed, that START doesn’t finish successfully when x3 is 0 or xi’s are too large.

    let key = usize::from_str_radix(key_str, 10).unwrap();

    let request = ConsList::Cons(
        ListVal::Number(make_mod_number(3)),

        ListVal::Cons(Box::new(ConsList::Cons(
            ListVal::Number(make_mod_number(key as isize)),

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



fn make_empty_commands_request(key_str: &str) -> String {
    // COMMANDS
    // (4, playerKey, commands)
    // commands is the list of issued commands.
    // Each item has format (type, shipId, ...), where ... denotes command-specific parameters.
    // THIS IMPLEMENTATION SENDS nil AS COMMANDS LIST
    let key = usize::from_str_radix(key_str, 10).unwrap();

    // assuming unknown list as nil for now
    let request = ConsList::Cons(
        ListVal::Number(make_mod_number(4)),

        ListVal::Cons(Box::new(ConsList::Cons(
            ListVal::Number(make_mod_number(key as isize)),

            ListVal::Cons(Box::new(ConsList::Cons(
                ListVal::Cons(Box::new(ConsList::Nil)),
                ListVal::Cons(Box::new(ConsList::Nil)))))))));

    request.modulate_to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();

    let server_url = &args[1];
    let player_key = &args[2];

    let intercom = Intercom::new(format!("{}/aliens/send", server_url));
    let mut runtime = Runtime::new().unwrap();

    let join_request = make_join_request(player_key);
    println!("Sending JOIN request: {}", &join_request);
    let join_response = intercom.send(join_request.clone(), &mut runtime).unwrap();
    println!("JOIN response: {}", &join_response);
    let join_response_pretty = ConsList::demodulate_from_string(&join_response).unwrap().to_pretty_string();
    println!("JOIN response pretty: {}", &join_response_pretty);


    let start_request = make_start_request(player_key, 0, 0, 0, 1);
    println!("Sending START request: {}", &start_request);
    let start_response = intercom.send(start_request.clone(), &mut runtime).unwrap();
    println!("START response: {}", &start_response);
    let start_response_pretty = ConsList::demodulate_from_string(&start_response).unwrap().to_pretty_string();
    println!("START response pretty: {}", &start_response_pretty);

    for turn in 0..255 {
        println!("++++ TURN {}", turn);

        let commands_request = make_empty_commands_request(player_key);
        println!("Sending COMMANDS request: {}", &start_request);
        let commands_response = intercom.send(commands_request.clone(), &mut runtime).unwrap();
        println!("COMMANDS response: {}", &commands_response);
        let commands_response_pretty = ConsList::demodulate_from_string(&commands_response).unwrap().to_pretty_string();
        println!("COMMANDS response pretty: {}", &commands_response_pretty);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_request() {
        assert_eq!(make_join_request("1"), "11011000101101100001110000");
        /* this is NOT our API key, but something similiar */
        assert_eq!(make_join_request("7551862922895789501"),
                   "11011000101101111111111111111100110100011001101100110110101000100011111101001011000110110111101110000");
    }

    #[test]
    fn start_request() {
        assert_eq!(make_start_request("1", 0, 0, 0, 0), "1101100011110110000111110101101011010110100000");
    }

    #[test]
    fn empty_commands_request() {
        assert_eq!(make_empty_commands_request("1"), "11011001001101100001110000");
    }

}
