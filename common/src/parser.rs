
use super::code::{
    Script,
};


#[derive(Parser)]
#[grammar = "asm.pest"]
pub struct AsmParser {

}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Error {
    Unknown,
}

impl AsmParser {
    pub fn new() -> AsmParser {
        AsmParser {

        }
    }

    pub fn parse_script(&self) -> Result<Script, Error> {
        Err(Error::Unknown)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
}
