
use std::io::{
    self,BufReader,Read,
};

use common::{
    code::*,
    parser::{self,AsmParser},
    vm::interpret::{Interpreter,self},
};

#[derive(Debug)]
enum Error {
    Read(io::Error),
    Parse(parser::Error),
    Vm(interpret::Error),
}

fn main() -> Result<(),Error> {
    let mut buffer = String::new();
    BufReader::new(std::io::stdin()).read_to_string(&mut buffer).map_err(Error::Read)?;

    let script = AsmParser.parse_script(&buffer).map_err(Error::Parse)?;
    let inter = Interpreter{};
    let env = inter.eval_script(script).map_err(Error::Vm)?;
    let oops = inter.eval(
        //ap ap ap interact x0 nil ap ap vec 0 0 = ( x16 , ap multipledraw x64 )
        inter.build_tree(Ops(vec![
            Op::App,
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Interact)),
            // this is "galaxy"
            Op::Variable(Variable {
                name: Number::Negative(NegativeNumber {
                    value: -1,
                }),
            }),
            Op::Const(Const::Fun(Fun::Nil)),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 0,
                }),
                modulation: Modulation::Demodulated,
            })),
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 0,
                }),
                modulation: Modulation::Demodulated,
            })),
        ])).map_err(Error::Vm)?,
        &env,
    ).map_err(Error::Vm)?;

    println!("{:#?}",oops);
    //Const(Fun(Galaxy))

    Ok(())
}
