use http_body::Body as _;
use hyper::{Client, Request, Method, Body, StatusCode};
use std::env;
use std::process;

use common::code::{
    EncodedNumber,
    PositiveNumber,
    Modulation,
    Number,
    make_dem_number,
};

use common::encoder::{
    ConsList,
    ListVal,
    Modulable,
};

fn make_join_request(key_str: &str) -> String {
    // JOIN request
    // (2, playerKey, (...unknown list...))
    let key = usize::from_str_radix(key_str, 10).unwrap();

    // assuming unknown list as nil for now
    let unknown_param = ListVal::Cons(Box::new(ConsList::Nil));

    let request = ConsList::Cons(
        ListVal::Number(make_dem_number(2)),
        ListVal::Cons(Box::new(ConsList::Cons(
            ListVal::Number(make_dem_number(key as isize)),
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

    // assuming unknown list as nil for now
    let unknown_param = ListVal::Cons(Box::new(ConsList::Nil));

    let request = ConsList::Cons(
        ListVal::Number(make_dem_number(3)),
        ListVal::Cons(Box::new(ConsList::Cons(
            ListVal::Number(make_dem_number(key as isize)),
            ListVal::Cons(Box::new(ConsList::Cons(
                unknown_param,
                ListVal::Cons(Box::new(ConsList::Nil)))))))));

    request.modulate_to_string()
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();

    let server_url = &args[1];
    let player_key = &args[2];

    println!("ServerUrl: {}; PlayerKey: {}", server_url, player_key);

    let client = Client::new();
    let req = Request::builder()
        .method(Method::POST)
        .uri(server_url)
        .body(Body::from(format!("{}", player_key)))?;

    match client.request(req).await {
        Ok(mut res) => {
            match res.status() {
                StatusCode::OK => {
                    print!("Server response: ");
                    while let Some(chunk) = res.body_mut().data().await {
                        match chunk {
                            Ok(content) => println!("{:?}", content),
                            Err(why) => println!("error reading body: {:?}", why)
                        }
                    }
                },
                _ => {
                    println!("Unexpected server response:");
                    println!("HTTP code: {}", res.status());
                    print!("Response body: ");
                    while let Some(chunk) = res.body_mut().data().await {
                        match chunk {
                            Ok(content) => println!("{:?}", content),
                            Err(why) => println!("error reading body: {:?}", why)
                        }
                    }
                    process::exit(2);
                }
            }
        },
        Err(err) => {
            println!("Unexpected server response:\n{}", err);
            process::exit(1);
        }
    }

    Ok(())
}
