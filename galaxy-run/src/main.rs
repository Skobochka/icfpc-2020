
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
    let oops = env.lookup(Ops(vec![Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 1338 }) })]));
    println!("{:?}",oops);
    //Const(Fun(Galaxy))

    Ok(())
}
