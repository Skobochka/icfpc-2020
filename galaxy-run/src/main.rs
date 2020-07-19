
use std::io::{
    self,BufReader,Read,
};

use common::{
    proto,
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
    let script = AsmParser.parse_script(proto::galaxy()).map_err(Error::Parse)?;
    let inter = Interpreter{};
    let env = inter.eval_script(script).map_err(Error::Vm)?;
    let oops = inter.eval(
        //ap ap ap interact x0 nil ap ap vec 0 0 = ( x16 , ap multipledraw x64 )
        inter.build_tree(Ops(vec![
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
