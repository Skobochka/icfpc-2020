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
    Variable,
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
            _ => {
                println!("parse_number() fail {:?}", number.as_rule());
                unreachable!()
            }
        }
    }

    pub fn parse_func(&self, func: Pair<Rule>) -> Fun {
        match func.as_rule() {
            Rule::dec_ => Fun::Dec,
            Rule::inc_ => Fun::Inc,
            Rule::add_ => Fun::Sum,
            Rule::mul_ => Fun::Mul,
            Rule::div_ => Fun::Div,

            Rule::vec_ => Fun::Vec,
            Rule::cons_ => Fun::Cons,
            Rule::car_ => Fun::Car,
            Rule::cdr_ => Fun::Cdr,
            Rule::nil_ => Fun::Nil,
            _ => {
                println!("parse_func() fail {:?}", func.as_rule());
                unreachable!()
            }
        }
    }

    pub fn parse_expr(&self, expr: Pair<Rule>) -> Op {
        // println!("Expr: {:?}", expr.as_str());
        match expr.as_rule() {
            Rule::named_func => {
                let mut inner_rules = expr.into_inner();
                Op::Const(Const::Fun(self.parse_func(inner_rules.next().unwrap())))
            },
            Rule::unnamed_func => {
                let name: usize = expr.into_inner().next().unwrap().as_str().parse().unwrap();
                Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: name })})
            },
            Rule::variable => {
                let name: usize = expr.into_inner().next().unwrap().as_str().parse().unwrap();
                Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: name })})
            },
            Rule::ap_func => {
                Op::App
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

        println!("Statement: {:?}", statement.as_str());
        let mut part_iter = statement.into_inner();

        let mut in_left = true;
        let mut left = Vec::<Op>::new();
        let mut right = Vec::<Op>::new();
        
        loop {
            match part_iter.next() {
                Some(node) => {
                    match node.as_rule() {
                        Rule::equal_sign => {
                            in_left = false;
                        },
                        _ if in_left => left.push(self.parse_expr(node)),
                        _ => right.push(self.parse_expr(node)),
                    }
                },
                None => break,
            }
        }
        
        Statement::Equality ( Equality { left: Ops(left), right: Ops(right) } )
    }

    pub fn parse_script(&self, input: &str) -> Result<Script, Error> {
        let mut statements =  Vec::<Statement>::new();
        
        for line in input.trim().split('\n') {
            let res = AsmParser::parse(Rule::statement, line);
            match res {
                Ok(mut statement) => {
                    statements.push(self.parse_statement(statement.next().unwrap()));
                },
                Err(e) => {
                    return Err(Error::PestParsingError(e))
                },
            }
        }

        Ok(Script {
            statements: statements,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_00() {
        let parser = AsmParser::new();
        assert_eq!(
            parser.parse_script("dec = inc\n"),
            Ok(Script {
                statements: vec![
                    Statement::Equality(Equality {
                        left: Ops(vec![
                            Op::Const(Const::Fun(Fun::Dec))
                        ]),
                        right: Ops(vec![
                            Op::Const(Const::Fun(Fun::Inc))
                        ]),
                    }),
                ],
            }));
    }

    #[test]
    fn simple_multiline() {
        let parser = AsmParser::new();
        assert_eq!(
            parser.parse_script("dec = inc\ndec = inc\n"),
            Ok(Script {
                statements: vec![
                    Statement::Equality(Equality {
                        left: Ops(vec![
                            Op::Const(Const::Fun(Fun::Dec))
                        ]),
                        right: Ops(vec![
                            Op::Const(Const::Fun(Fun::Inc))
                        ]),
                    }),
                    Statement::Equality(Equality {
                        left: Ops(vec![
                            Op::Const(Const::Fun(Fun::Dec))
                        ]),
                        right: Ops(vec![
                            Op::Const(Const::Fun(Fun::Inc))
                        ]),
                    }),
                ],
            }));
    }

    #[test]
    fn simple_multiline_no_trail_newline() {
        let parser = AsmParser::new();
        assert_eq!(
            parser.parse_script("dec = inc\ndec = inc"),
            Ok(Script {
                statements: vec![
                    Statement::Equality(Equality {
                        left: Ops(vec![
                            Op::Const(Const::Fun(Fun::Dec))
                        ]),
                        right: Ops(vec![
                            Op::Const(Const::Fun(Fun::Inc))
                        ]),
                    }),
                    Statement::Equality(Equality {
                        left: Ops(vec![
                            Op::Const(Const::Fun(Fun::Dec))
                        ]),
                        right: Ops(vec![
                            Op::Const(Const::Fun(Fun::Inc))
                        ]),
                    }),
                ],
            }));
    }

    #[test]
    fn regression_nospace() {
        let parser = AsmParser::new();
        assert!(parser.parse_script("cc = ss").is_err());
    }

    // #[test]
    // fn galaxy_line1() {
    //     let parser = AsmParser::new();
    //     assert_eq!(
    //         parser.parse_script(":1029 = ap ap cons 7 ap ap cons 123229502148636 nil"),
    //         Ok(Script {
    //             statements: vec![
    //                 Statement::Equality(Equality {
    //                     left: Ops(vec![
    //                         Op::Const(Const::Fun(Fun::Dec))
    //                     ]),
    //                     right: Ops(vec![
    //                         Op::Const(Const::Fun(Fun::Inc))
    //                     ]),
    //                 }),
    //             ],
    //         }));
    // }
}
