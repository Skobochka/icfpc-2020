use std::{
    sync::{
        Arc,
        Mutex,
    },
    thread,
};

use rand::Rng;

use common::{
    vm::interpret::{
        Interpreter,
    },
    proto::{
        galaxy,
        Session,
    },
    parser::AsmParser,
    code::{
        Op,
        Ops,
        Fun,
        Const,
        Number,
        Modulation,
        EncodedNumber,
        PositiveNumber,
    },
    encoder::{
        self,
        Modulable,
        PrettyPrintable,
    },
};

#[derive(Debug)]
enum Error {
    Proto(common::proto::Error),
    InitStateParse(common::parser::Error),
    ResultsListGetterParse(common::parser::Error),
}

fn main() -> Result<(), Error> {
    let parser = AsmParser::new();
    let state_asm = "(4, (1, (122, 203, 410, 164, 444, 484, 202, 77, 251, 56, 456, 435, 28, 329, 257, 265, 501, 18, 190, 423, 384, 434, 266, 69, 34, 437, 203, 152, 160, 425, 245, 428, 99, 107, 192, 372, 346, 344, 169, 478, 393, 502, 201, 497, 313, 32, 281, 510, 436, 22, 237, 80, 325, 405, 184, 358, 57, 276, 359, 189, 284, 277, 198, 244), -1, 0, nil), 0, (103652820))";
    let init_state_ops = parser.parse_expression(state_asm)
        .map_err(Error::InitStateParse)?;
    let points: Vec<_> = (0 .. 8)
        .flat_map(|row| (0 .. 8).map(move |col| (row, col)))
        .collect();
    let mut pairs = vec![];
    for i in 0 .. points.len() - 1 {
        for j in (i + 1) .. points.len() {
            pairs.push((points[i], points[j]));
        }
    }
    let sync_pairs = Arc::new(Mutex::new(pairs));

    let mut slaves = vec![];
    for _ in 0 .. 4 {
        let sync_pairs = sync_pairs.clone();
        let init_state_ops = init_state_ops.clone();

        slaves.push(thread::spawn(move || {
            let mut session = Session::with_interpreter(
                galaxy(),
                Interpreter::new(),
            ).map_err(Error::Proto)?;
            let parser = AsmParser::new();

            let mut state_ops = init_state_ops.clone();

            let mut rng = rand::thread_rng();
            loop {
                let pair = {
                    let mut pairs = sync_pairs.lock().unwrap();
                    if pairs.is_empty() {
                        break;
                    }
                    let index = rng.gen_range(0, pairs.len());
                    pairs.swap_remove(index)
                };

                state_ops = run_on_state_coords(&mut session, state_ops, pair.0)?;
                state_ops = run_on_state_coords(&mut session, state_ops, pair.1)?;
                let results = results_list(&mut session, &parser, &state_ops)?;
                match &*results.0 {
                    [Op::Const(Const::ModulatedBits(results_bits))] => {
                        if results_bits.len() > 2 {
                            println!(
                                " ;; a new pair is found @ {:?}! {:?}",
                                pair,
                                encoder::ConsList::demodulate_from_string(results_bits)
                                    .unwrap()
                                    .to_pretty_string(),
                            );
                        } else {
                            println!(" ;; rejecting pair {:?}, restoring previous session ({} pairs left)", pair, sync_pairs.lock().unwrap().len());
                            state_ops = init_state_ops.clone();
                        }
                    },
                    _ =>
                        panic!("unexpected values result = {:?}", results),
                }
            }
            Ok(())
        }));
    }

    while let Some(slave) = slaves.pop() {
        slave.join().unwrap()?;
    }

    Ok(())
}

fn run_on_state_coords(session: &mut Session, state_ops: Ops, pair: (usize, usize)) -> Result<Ops, Error> {
    let coord_x = pair.0 * 6;
    let coord_y = pair.1 * 6;

    let mut ops = Ops(vec![]);
    ops.0.extend(vec![
        Op::App,
        Op::App,
        Op::App,
        Op::Const(Const::Fun(Fun::Interact)),
        Op::Const(Const::Fun(Fun::Galaxy)),
    ]);
    ops.0.extend(state_ops.0);
    ops.0.extend(vec![
        Op::App,
        Op::App,
        Op::Const(Const::Fun(Fun::Vec)),
        Op::Const(Const::EncodedNumber(EncodedNumber {
            number: Number::Positive(PositiveNumber{ value: coord_x, }),
            modulation: Modulation::Demodulated,
        })),
        Op::Const(Const::EncodedNumber(EncodedNumber {
            number: Number::Positive(PositiveNumber{ value: coord_y, }),
            modulation: Modulation::Demodulated,
        })),
    ]);

    let out_ops = session.eval_ops(ops).map_err(Error::Proto)?;

    let mut ops = Ops(vec![]);
    ops.0.extend(vec![
        Op::App,
        Op::Const(Const::Fun(Fun::Car)),
    ]);
    ops.0.extend(out_ops.0);

    session.eval_force_list(ops).map_err(Error::Proto)
}

fn results_list(session: &mut Session, parser: &AsmParser, state: &Ops) -> Result<Ops, Error> {
    let mut ops = parser.parse_expression("ap car ap cdr ap cdr ap cdr ap cdr ap car ap cdr")
        .map_err(Error::ResultsListGetterParse)?;
    ops.0.extend(state.0.iter().cloned());

    session.eval_ops(ops).map_err(Error::Proto)
}
