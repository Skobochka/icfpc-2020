
use std::io::{
    BufReader,BufRead,Write,
};

use common::proto::{Session,Error};


fn main() -> Result<(),Error> {
    let mut session = Session::galaxy()?;

    println!("Enter a command to run\nFor example: ap galaxy 0\nOr 'exit' to exit...\n");
    print!(">>> "); std::io::stdout().flush().ok();
    for row in BufReader::new(std::io::stdin()).lines() {
        let asm = match row {
            Err(e) => {
                println!("Read error: {:?}\n",e);
                print!(">>> "); std::io::stdout().flush().ok();
                continue;
            },
            Ok(asm) => asm,
        };
        match &asm[..] {
            "exit" => {
                println!("Bye...");
                return Ok(());
            },
            "" => {
                print!(">>> "); std::io::stdout().flush().ok();
                continue;
            },
            _ => {},
        }
        match session.eval_asm(&asm) {
            Ok(ops) => {
                println!("Ok:");
                for op in ops.0 {
                    println!("   {:?}",op);
                }
                println!("");
            },
            Err(e) => {
                println!("Error: {:?}",e);
            },
        }
        print!(">>> "); std::io::stdout().flush().ok();
    }
    Ok(())
}
