use crate::{
    code::Ops,
    parser::{self,AsmParser},
    vm::interpret::{self,Env,Interpreter},
};

mod galaxy;

pub fn galaxy() -> &'static str {
    galaxy::GALAXY
}

#[derive(Debug)]
pub enum Error {
    Parse(parser::Error),
    Vm(interpret::Error),
}

pub struct Session {
    inter: Interpreter,
    env: Env,
}
impl Session {
    pub fn galaxy() -> Result<Session,Error> {
        Session::new(galaxy())
    }

    pub fn new(proto: &str) -> Result<Session,Error> {
        Session::with_interpreter(proto, Interpreter::new())
    }

    pub fn with_interpreter(proto: &str, inter: Interpreter) -> Result<Session,Error> {
        let script = AsmParser.parse_script(proto).map_err(Error::Parse)?;
        let env = inter.eval_script(script).map_err(Error::Vm)?;
        Ok(Session {
            inter: inter,
            env: env,
        })
    }

    pub fn eval_asm(&mut self, asm: &str) -> Result<Ops,Error> {
        self.eval_ops(AsmParser.parse_expression(&asm).map_err(Error::Parse)?)
    }
    pub fn eval_ops(&mut self, ops: Ops) -> Result<Ops,Error> {
        self.inter.eval(
            self.inter.build_tree(ops).map_err(Error::Vm)?,
            &self.env,
        ).map_err(Error::Vm)
    }
}
