use super::code::{
    Script,
    Op,
    Const,
    Number,
    EncodedNumber,
    PositiveNumber,
    NegativeNumber,
    Modulation,
    Statement,
    Ops,
    Equality,
    Fun,
};

use pest::{
    Parser,
    iterators::{
        Pair,
    }
};

#[derive(Parser)]
#[grammar = "asm.pest"]
pub struct AsmParser;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Error {
    PestParsingError(pest::error::Error<Rule>),
}

impl AsmParser {
    pub fn new() -> AsmParser {
        AsmParser {}
    }

    pub fn parse_number(&self, number: Pair<Rule>) -> EncodedNumber {
        match number.as_rule() {
            Rule::grid_positive_number_literal => EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: number.as_str().parse().unwrap()
                }),
                modulation: Modulation::Demodulated,
            },
            Rule::grid_negative_number_literal => EncodedNumber {
                number: Number::Negative(NegativeNumber {
                    value: number.as_str().parse().unwrap()
                }),
                modulation: Modulation::Demodulated,
            },
            _ => unreachable!()
        }
    }

    pub fn parse_func(&self, func: Pair<Rule>) -> Fun {
        // match number.as_rule() {
        //     _ => {
                println!("{:?}", func.as_rule());
                unreachable!()
        //     }
        // }
    }

    pub fn parse_expr(&self, expr: Pair<Rule>) -> Op {
        match expr.as_rule() {
            Rule::named_func => {
                let mut inner_rules = expr.into_inner();
                Op::Const(Const::Fun(self.parse_func(inner_rules.next().unwrap())))
            },
            Rule::grid_positive_number_literal | Rule::grid_negative_number_literal =>
                Op::Const(Const::EncodedNumber(self.parse_number(expr))),
            _ => {
                println!("LOL {:?}", expr.as_rule());
                unreachable!()
            }
        }
    }

    pub fn parse_statement(&self, statement: Pair<Rule>) -> Statement {
        // FIXME: this is just ugly :(
        // let mut inner_rules = statement.into_inner().collect::<Vec<&Pair<Rule>>>();
        let mut inner_rules = statement.into_inner();
        Statement::Equality ( Equality {

            left: Ops(inner_rules.clone().take_while(|expr| match expr.as_rule() {
                Rule::equal_sign => false,
                _ => true,
            }).map(|expr| self.parse_expr(expr)).collect()),

            right: Ops(inner_rules.clone().skip_while(|expr| match expr.as_rule() {
                Rule::equal_sign => false,
                _ => true,
            }).skip(1).map(|expr| self.parse_expr(expr)).collect()),
        })
    }

    pub fn parse_script(&self, input: &str) -> Result<Script, Error> {
        let res = AsmParser::parse(Rule::script, input);
        match res {
            Ok(lines) => Ok(Script {
                statements: lines.map(|statement| self.parse_statement(statement)).collect()
            }),
            Err(e) => Err(Error::PestParsingError(e)),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_test() {
        let parser = AsmParser::new();
        assert_eq!(
            parser.parse_script("dec = inc\nadd x0 = ap dec 1"),
            Ok(Script {
                statements: vec![]
            }));
    }
}
