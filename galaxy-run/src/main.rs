use std::io;

use rustyline::{
    error::ReadlineError,
    Editor,
};

use common::proto::{
    Session,
};

#[derive(Debug)]
enum Error {
    Proto(common::proto::Error),
    Readline(ReadlineError),
}

fn main() -> Result<(), Error> {

    let mut rl = Editor::<()>::new();
    match rl.load_history("./galaxy-run-history.txt") {
        Ok(()) =>
            (),
        Err(ReadlineError::Io(ref e)) if e.kind() == io::ErrorKind::NotFound => {
            println!("no previous history in current dir");
        },
        Err(e) =>
            return Err(Error::Readline(e)),
    }

    let mut session = Session::galaxy().map_err(Error::Proto)?;

    println!("Enter a command to run\nFor example: ap galaxy 0\nOr 'exit' to exit...\n");
    loop {
        let readline = rl.readline(">>> ");
        let asm = match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                line
            },
            Err(ReadlineError::Interrupted) => {
                println!("Exit on <CTRL-C>");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("Exit on <CTRL-D>");
                break
            },
            Err(err) => {
                println!("Read rrror: {:?}", err);
                break
            }
        };
        match &asm[..] {
            "exit" => {
                println!("Bye...");
                break;
            },
            "" =>
                continue,
            _ =>
                (),
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
    }
    rl.save_history("./galaxy-run-history.txt").unwrap();
    Ok(())
}
