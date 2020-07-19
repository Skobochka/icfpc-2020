use std::{
    io,
};

use futures::{
    channel::{
        oneshot,
        mpsc::unbounded,
    },
    StreamExt,
};

use rustyline::{
    error::ReadlineError,
    Editor,
};

use common::{
    vm::interpret::{
        Interpreter,
        OuterRequest,
    },
    proto::{
        galaxy,
        Session,
    },
    send::Intercom,
};

#[derive(Debug)]
enum Error {
    Proto(common::proto::Error),
    Readline(ReadlineError),
    QuitTxTerminated,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (quit_tx, quit_rx) = oneshot::channel();

    let (outer_tx, mut outer_rx) = unbounded();

    let mut session = Session::with_interpreter(
        galaxy(),
        Interpreter::with_outer_channel(outer_tx),
    ).map_err(Error::Proto)?;

    tokio::spawn(async move {
        let intercom = Intercom::proxy();

        while let Some(request) = outer_rx.next().await {
            match request {
                OuterRequest::ProxySend { modulated_req, modulated_rep, } => {
                    println!("** >> transmission rq: {:?}", modulated_req);
                    match intercom.async_send(modulated_req).await {
                        Ok(response) => {
                            println!("** << transmission rp: {:?}", response);
                            if let Err(..) = modulated_rep.send(response) {
                                println!("interpreter has gone, quitting");
                                break;
                            }
                        },
                        Err(error) => {
                            println!("intercom send failed: {:?}, quitting", error);
                            break;
                        },
                    }
                },
            }
        }

        println!("intercom task termination");
    });

    tokio::task::spawn_blocking(move || {
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

        quit_tx.send(()).ok();
        Ok(())
    });

    quit_rx.await.map_err(|_| Error::QuitTxTerminated)
}
