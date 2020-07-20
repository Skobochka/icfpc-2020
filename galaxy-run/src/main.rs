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
    code::*,
};

#[derive(Debug)]
enum Error {
    Proto(common::proto::Error),
    Readline(ReadlineError),
    QuitTxTerminated,
}

#[allow(dead_code)]
fn ops2asm(ops: &Ops) -> String {
    let mut s = String::new();
    for op in ops.0.iter() {
        s += " ";
        match op {
            Op::App => s += "ap",
            Op::Const(Const::Picture(..)) => s += "[pic]",
            Op::Const(Const::Fun(Fun::Cons)) => s += "cons",
            Op::Const(Const::Fun(Fun::Inc)) => s += "inc",
            Op::Const(Const::Fun(Fun::Dec)) => s += "dec",
            Op::Const(Const::Fun(Fun::Sum)) => s += "add",
            Op::Const(Const::Fun(Fun::Mul)) => s += "mul",
            Op::Const(Const::Fun(Fun::Div)) => s += "div",
            Op::Const(Const::Fun(Fun::Eq)) => s += "eq",
            Op::Const(Const::Fun(Fun::Lt)) => s += "lt",
            Op::Const(Const::Fun(Fun::Mod)) => s += "mod",
            Op::Const(Const::Fun(Fun::Dem)) => s += "dem",
            Op::Const(Const::Fun(Fun::Send)) => s += "send",
            Op::Const(Const::Fun(Fun::Neg)) => s += "neg",
            Op::Const(Const::Fun(Fun::S)) => s += "s",
            Op::Const(Const::Fun(Fun::B)) => s += "b",
            Op::Const(Const::Fun(Fun::C)) => s += "c",
            Op::Const(Const::Fun(Fun::True)) => s += "t",
            Op::Const(Const::Fun(Fun::False)) => s += "f",
            Op::Const(Const::Fun(Fun::Pwr2)) => s += "pwr",
            Op::Const(Const::Fun(Fun::I)) => s += "i",
            Op::Const(Const::Fun(Fun::Car)) => s += "car",
            Op::Const(Const::Fun(Fun::Cdr)) => s += "cdr",
            Op::Const(Const::Fun(Fun::Nil)) => s += "nil",
            Op::Const(Const::Fun(Fun::IsNil)) => s += "isnil",
            Op::Const(Const::Fun(Fun::Vec)) => s += "vec",
            Op::Const(Const::Fun(Fun::Draw)) => s += "draw",
            Op::Const(Const::Fun(Fun::MultipleDraw)) => s += "multipledraw",
            Op::Const(Const::Fun(Fun::If0)) => s += "if0",
            Op::Const(Const::Fun(Fun::Interact)) => s += "interact",
            Op::Const(Const::Fun(Fun::Modem)) => s += "modem",
            Op::Const(Const::Fun(Fun::Galaxy)) => s += "galaxy",
            Op::Const(Const::Fun(Fun::Chkb)) |
            Op::Const(Const::Fun(Fun::Checkerboard)) => s += "checkerboard",
            Op::Const(Const::Fun(Fun::F38)) => s += "f38",
            Op::Const(Const::Fun(Fun::Render)) => s += "render",
            Op::Const(Const::EncodedNumber(EncodedNumber { number: Number::Positive(PositiveNumber { value: v }), .. })) => s += &v.to_string(),
            Op::Const(Const::EncodedNumber(EncodedNumber { number: Number::Negative(NegativeNumber { value: v }), .. })) => s += &v.to_string(),
            Op::Variable(Variable{ name: Number::Positive(PositiveNumber { value: v }) }) => s += &format!(":{}",v.to_string()),
            Op::Variable(Variable{ name: Number::Negative(NegativeNumber { value: v }) }) => s += &format!(":{}",v.to_string()),
            Op::Syntax(Syntax::LeftParen) => s += "(",
            Op::Syntax(Syntax::Comma) => s += ",",
            Op::Syntax(Syntax::RightParen) => s += ")",
        }
    }
    s
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (quit_tx, quit_rx) = oneshot::channel();

    let (outer_tx, mut outer_rx) = unbounded();

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
                OuterRequest::RenderPictures { pictures, } => {
                    println!("** >> render rq: {:?}", pictures);
                },
            }
        }

        println!("intercom task termination");
    });

    tokio::task::spawn_blocking(move || {
        let mut session = Session::with_interpreter(
            galaxy(),
            Interpreter::with_outer_channel(outer_tx),
        ).map_err(Error::Proto)?;

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
                    println!("   {:?}", ops);
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
