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
    Syntax,
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
    Unknown,
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
                unimplemented!("parse_number() fail {:?}", number.as_rule())
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
            Rule::eq_ => Fun::Eq,
            Rule::lt_ => Fun::Lt,

            Rule::vec_ => Fun::Vec,
            Rule::cons_ => Fun::Cons,
            Rule::car_ => Fun::Car,
            Rule::cdr_ => Fun::Cdr,
            Rule::nil_ => Fun::Nil,
            Rule::isnil_ => Fun::IsNil,

            Rule::neg_ => Fun::Neg,
            Rule::mod_ => Fun::Mod,
            Rule::dem_ => Fun::Dem,

            Rule::s_ => Fun::S,
            Rule::c_ => Fun::C,
            Rule::b_ => Fun::B,
            Rule::i_ => Fun::I,
            Rule::true_ => Fun::True,
            Rule::false_ => Fun::False,

            Rule::draw_ => Fun::Draw,
            Rule::multipledraw_ => Fun::MultipleDraw,

            Rule::galaxy_ => Fun::Galaxy,

            _ => {
                unimplemented!("parse_func() {:?}", func.as_rule());
            }
        }
    }

    pub fn parse_expr(&self, expr: Pair<Rule>) -> Op {
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
            Rule::ap_func => Op::App,
            Rule::left_paren => Op::Syntax(Syntax::LeftParen),
            Rule::right_paren => Op::Syntax(Syntax::RightParen),
            Rule::comma => Op::Syntax(Syntax::Comma),
            Rule::grid_positive_number_literal | Rule::grid_negative_number_literal =>
                Op::Const(Const::EncodedNumber(self.parse_number(expr))),
            _ => {
                unimplemented!("parse_expr() fail {:?}", expr.as_rule());
            }
        }
    }

    fn parse_list_construction_recursive(&self, node: Pair<Rule>, into: &mut Vec<Op>) {
        match node.as_rule() {
            Rule::list_construction => {
                for expr in node.into_inner() {
                    self.parse_list_construction_recursive(expr, into)
                }
            }
            _ => into.push(self.parse_expr(node)),
        }
    }

    pub fn parse_statement(&self, statement: Pair<Rule>) -> Statement {

        let mut part_iter = statement.into_inner();

        let mut in_left = true;
        let mut left = Vec::<Op>::new();
        let mut right = Vec::<Op>::new();

        loop {
            match part_iter.next() {
                Some(node) => {
                    // println!("parse_statement(): {:?} {:?}", node.as_rule(), node.as_str());

                    match node.as_rule() {
                        Rule::equal_sign => {
                            in_left = false;
                        },
                        Rule::list_construction if in_left => {
                            self.parse_list_construction_recursive(node, &mut left);
                        }
                        Rule::list_construction => {
                            self.parse_list_construction_recursive(node, &mut right);
                        }
                        _ if in_left => left.push(self.parse_expr(node)),
                        _ => right.push(self.parse_expr(node)),
                    }
                },
                None => break,
            }
        }

        Statement::Equality ( Equality { left: Ops(left), right: Ops(right) } )
    }

    pub fn parse_expression(&self, input: &str) -> Result<Ops, Error> {
        let res = AsmParser::parse(Rule::expr, input);
        match res {
            Ok(mut exprs) => {
                let expr = exprs.next().unwrap();

                let mut ops = Vec::<Op>::new();
                for expr_part in expr.into_inner() {
                    match expr_part.as_rule() {
                        Rule::list_construction => {
                            for node in expr_part.into_inner() {
                                ops.push(self.parse_expr(node))
                            }
                        },
                        _ => ops.push(self.parse_expr(expr_part)),
                    }
                }

                Ok(Ops(ops))
            },
            Err(e) => {
                return Err(Error::PestParsingError(e))
            },
        }
    }

    pub fn parse_script(&self, input: &str) -> Result<Script, Error> {
        let mut statements = Vec::<Statement>::new();

        for line in input.trim().lines() {
            let res = AsmParser::parse(Rule::statement, line);
            match res {
                Ok(mut statement) => {
                    match statement.next() {
                        Some(statement_node) => {
                            statements.push(self.parse_statement(statement_node));
                        },
                        None => {
                            return Err(Error::Unknown)
                        }
                    }
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
    fn list_constructions() {
        let parser = AsmParser::new();
        assert_eq!(
            parser.parse_script(":1234 = ap cons (1, 2, 3)"),
            Ok(Script {
                statements: vec![
                    Statement::Equality(Equality {
                        left: Ops(vec![
                            Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 1234 })})
                        ]),
                        right: Ops(vec![
                            Op::App,
                            Op::Const(Const::Fun(Fun::Cons)),
                            Op::Syntax(Syntax::LeftParen),
                            Op::Const(Const::EncodedNumber(EncodedNumber {
                                number: Number::Positive(PositiveNumber {
                                    value: 1,
                                }),
                                modulation: Modulation::Demodulated,
                            })),
                            Op::Syntax(Syntax::Comma),
                            Op::Const(Const::EncodedNumber(EncodedNumber {
                                number: Number::Positive(PositiveNumber {
                                    value: 2,
                                }),
                                modulation: Modulation::Demodulated,
                            })),
                            Op::Syntax(Syntax::Comma),
                            Op::Const(Const::EncodedNumber(EncodedNumber {
                                number: Number::Positive(PositiveNumber {
                                    value: 3,
                                }),
                                modulation: Modulation::Demodulated,
                            })),
                            Op::Syntax(Syntax::RightParen),
                        ]),
                    }),
                ],
            }));
    }

    #[test]
    fn list_constructions2() {
        let parser = AsmParser::new();
        assert_eq!(
            parser.parse_script(":1234 = ap cons ( 1, x1, 3)"),
            Ok(Script {
                statements: vec![
                    Statement::Equality(Equality {
                        left: Ops(vec![
                            Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 1234 })})
                        ]),
                        right: Ops(vec![
                            Op::App,
                            Op::Const(Const::Fun(Fun::Cons)),
                            Op::Syntax(Syntax::LeftParen),
                            Op::Const(Const::EncodedNumber(EncodedNumber {
                                number: Number::Positive(PositiveNumber {
                                    value: 1,
                                }),
                                modulation: Modulation::Demodulated,
                            })),
                            Op::Syntax(Syntax::Comma),
                            Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 1 })}),
                            Op::Syntax(Syntax::Comma),
                            Op::Const(Const::EncodedNumber(EncodedNumber {
                                number: Number::Positive(PositiveNumber {
                                    value: 3,
                                }),
                                modulation: Modulation::Demodulated,
                            })),
                            Op::Syntax(Syntax::RightParen),
                        ]),
                    }),
                ],
            }));
    }

    #[test]
    fn list_constructions_invalid() {
        let parser = AsmParser::new();
        assert!(parser.parse_script(":1234 = ap cons (").is_err());
        assert!(parser.parse_script(":1234 = ap cons )").is_err());
        assert!(parser.parse_script(":1234 = ap cons ,").is_err());
        assert!(parser.parse_script(":1234 = ap cons (,)").is_err());
        assert!(parser.parse_script(":1234 = ap cons (,,)").is_err());
    }

    #[test]
    fn list_constructions_nested() {
        let parser = AsmParser::new();
        assert_eq!(
            parser.parse_script(":1234 = ap cons ((1, x1), 3)"),
            Ok(Script {
                statements: vec![
                    Statement::Equality(Equality {
                        left: Ops(vec![
                            Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 1234 })})
                        ]),
                        right: Ops(vec![
                            Op::App,
                            Op::Const(Const::Fun(Fun::Cons)),
                            Op::Syntax(Syntax::LeftParen),
                            Op::Syntax(Syntax::LeftParen),
                            Op::Const(Const::EncodedNumber(EncodedNumber {
                                number: Number::Positive(PositiveNumber {
                                    value: 1,
                                }),
                                modulation: Modulation::Demodulated,
                            })),
                            Op::Syntax(Syntax::Comma),
                            Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 1 })}),
                            Op::Syntax(Syntax::RightParen),
                            Op::Syntax(Syntax::Comma),
                            Op::Const(Const::EncodedNumber(EncodedNumber {
                                number: Number::Positive(PositiveNumber {
                                    value: 3,
                                }),
                                modulation: Modulation::Demodulated,
                            })),
                            Op::Syntax(Syntax::RightParen),
                        ]),
                    }),
                ],
            }));
    }

    #[test]
    fn parse_expression() {
        let parser = AsmParser::new();
        assert_eq!(parser.parse_expression("ap inc 1"),
                   Ok(Ops(vec![
                          Op::App,
                          Op::Const(Const::Fun(Fun::Inc)),
                          Op::Const(Const::EncodedNumber(EncodedNumber {
                              number: Number::Positive(PositiveNumber {
                                  value: 1,
                              }),
                              modulation: Modulation::Demodulated,
                           }))
                           ])));

        assert_eq!(parser.parse_expression("ap ap add ap ap add 2 3 4"), Ok(Ops(vec![
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::Sum)),
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::Sum)),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 2,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 3,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 4,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                ])));
    }


    #[test]
    fn regression_nospace() {
        let parser = AsmParser::new();
        assert!(parser.parse_script("cc = ss").is_err());
    }

    #[test]
    fn parse_draw() {
        let parser = AsmParser::new();
        assert_eq!(parser.parse_expression("ap draw ap ap cons ap ap cons 1 1 nil"),
                   Ok(Ops(vec![
                          Op::App,
                          Op::Const(Const::Fun(Fun::Draw)),
                          Op::App,
                          Op::App,
                          Op::Const(Const::Fun(Fun::Cons)),
                          Op::App,
                          Op::App,
                          Op::Const(Const::Fun(Fun::Cons)),
                          Op::Const(Const::EncodedNumber(EncodedNumber {
                              number: Number::Positive(PositiveNumber {
                                  value: 1,
                              }),
                              modulation: Modulation::Demodulated,
                           })),
                          Op::Const(Const::EncodedNumber(EncodedNumber {
                              number: Number::Positive(PositiveNumber {
                                  value: 1,
                              }),
                              modulation: Modulation::Demodulated,
                          })),
                          Op::Const(Const::Fun(Fun::Nil)),
                       ])));
    }

    #[test]
    fn parse_multipledraw() {
        let parser = AsmParser::new();
        assert_eq!(parser.parse_expression("ap multipledraw ap ap cons ap ap cons 1 1 nil"),
                   Ok(Ops(vec![
                          Op::App,
                          Op::Const(Const::Fun(Fun::MultipleDraw)),
                          Op::App,
                          Op::App,
                          Op::Const(Const::Fun(Fun::Cons)),
                          Op::App,
                          Op::App,
                          Op::Const(Const::Fun(Fun::Cons)),
                          Op::Const(Const::EncodedNumber(EncodedNumber {
                              number: Number::Positive(PositiveNumber {
                                  value: 1,
                              }),
                              modulation: Modulation::Demodulated,
                           })),
                          Op::Const(Const::EncodedNumber(EncodedNumber {
                              number: Number::Positive(PositiveNumber {
                                  value: 1,
                              }),
                              modulation: Modulation::Demodulated,
                          })),
                          Op::Const(Const::Fun(Fun::Nil)),
                       ])));
    }

    #[test]
    fn galaxy_smoke_test() {
        let parser = AsmParser::new();
        let galaxy = include_str!("../../problems/galaxy.txt");

        assert!(parser.parse_script(galaxy).is_ok())
    }


    #[test]
    fn simple_unnamed_functions_lightning() {
        let parser = AsmParser::new();
        assert_eq!(
            parser.parse_script(":1234 = inc"),
            Ok(Script {
                statements: vec![
                    Statement::Equality(Equality {
                        left: Ops(vec![
                            Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 1234 })})
                        ]),
                        right: Ops(vec![
                            Op::Const(Const::Fun(Fun::Inc))
                        ]),
                    }),
                ],
            }));
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
