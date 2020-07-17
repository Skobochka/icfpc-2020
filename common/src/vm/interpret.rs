use super::{
    super::code::{
        Op,
        Ops,
        Fun,
        Const,
        Number,
        EncodedNumber,
        PositiveNumber,
        NegativeNumber,
    },
    Env,
};

pub struct Interpreter {

}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Error {
    NoAppFunProvided,
    NoAppArgProvided { fun: AstNode, },
    EvalEmptyTree,
    AppOnNumber { number: EncodedNumber, arg: AstNode, },
    AppIncOnFun { fun: EvalFun, },
    AppDecOnFun { fun: EvalFun, },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ast {
    Empty,
    Tree(AstNode),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum AstNode {
    Literal { value: Op, },
    App { fun: Box<AstNode>, arg: Box<AstNode>, },
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {

        }
    }

    pub fn build_tree(&self, Ops(ops): Ops) -> Result<Ast, Error> {
        enum State {
            AwaitAppFun,
            AwaitAppArg { fun: AstNode, },
        }

        let mut states = vec![];
        let mut ops_iter = ops.into_iter();
        loop {
            let mut maybe_node: Option<AstNode> = match ops_iter.next() {
                None =>
                    None,
                Some(value @ Op::Const(..)) |
                Some(value @ Op::Variable(..)) =>
                    Some(AstNode::Literal { value: value, }),
                Some(Op::App) => {
                    states.push(State::AwaitAppFun);
                    continue;
                },
            };

            loop {
                match (states.pop(), maybe_node) {
                    (None, None) =>
                        return Ok(Ast::Empty),
                    (None, Some(node)) =>
                        return Ok(Ast::Tree(node)),
                    (Some(State::AwaitAppFun), None) =>
                        return Err(Error::NoAppFunProvided),
                    (Some(State::AwaitAppFun), Some(node)) => {
                        states.push(State::AwaitAppArg { fun: node, });
                        break;
                    },
                    (Some(State::AwaitAppArg { fun, }), None) =>
                        return Err(Error::NoAppArgProvided { fun, }),
                    (Some(State::AwaitAppArg { fun, }), Some(node)) => {
                        maybe_node = Some(AstNode::App {
                            fun: Box::new(fun),
                            arg: Box::new(node),
                        });
                    },
                }
            }
        }
    }

    pub fn eval(&self, ast: Ast, env: &mut Env) -> Result<Ops, Error> {
        match ast {
            Ast::Empty =>
                Err(Error::EvalEmptyTree),
            Ast::Tree(node) =>
                self.eval_tree(node, env),
        }
    }

    fn eval_tree(&self, mut ast_node: AstNode, _env: &mut Env) -> Result<Ops, Error> {
        enum State {
            EvalAppFun { arg: AstNode, },
            EvalAppArg { fun: EvalFun, },
        }

        let mut states = vec![];
        loop {
            let mut eval_op = match ast_node {
                AstNode::Literal { value, } =>
                    EvalOp::new(value),

                AstNode::App { fun, arg, } => {
                    states.push(State::EvalAppFun { arg: *arg, });
                    ast_node = *fun;
                    continue;
                },
            };

            loop {
                match (states.pop(), eval_op) {
                    (None, EvalOp::Number { number, }) =>
                        return Ok(Ops(vec![Op::Const(Const::EncodedNumber(number))])),
                    (None, EvalOp::Fun(EvalFun::Inc0)) =>
                        return Ok(Ops(vec![Op::Const(Const::Fun(Fun::Inc))])),
                    (None, EvalOp::Fun(EvalFun::Dec0)) =>
                        return Ok(Ops(vec![Op::Const(Const::Fun(Fun::Dec))])),

                    (Some(State::EvalAppFun { arg, }), EvalOp::Number { number, }) =>
                        return Err(Error::AppOnNumber { number, arg, }),
                    (Some(State::EvalAppFun { arg, }), EvalOp::Fun(fun)) => {
                        states.push(State::EvalAppArg { fun, });
                        ast_node = arg;
                        break;
                    },

                    // inc on positive number
                    (
                        Some(State::EvalAppArg { fun: EvalFun::Inc0, }),
                        EvalOp::Number {
                            number: EncodedNumber {
                                number: Number::Positive(PositiveNumber { value, }),
                                modulation,
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Number {
                            number: EncodedNumber {
                                number: Number::Positive(PositiveNumber { value: value + 1, }),
                                modulation,
                            },
                        },
                    // inc on negative number
                    (
                        Some(State::EvalAppArg { fun: EvalFun::Inc0, }),
                        EvalOp::Number {
                            number: EncodedNumber {
                                number: Number::Negative(NegativeNumber { value, }),
                                modulation,
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Number {
                            number: EncodedNumber {
                                number: if value + 1 < 0 {
                                    Number::Negative(NegativeNumber { value: value + 1, })
                                } else {
                                    Number::Positive(PositiveNumber { value: (value + 1) as usize, })
                                },
                                modulation,
                            },
                        },
                    // inc on fun
                    (Some(State::EvalAppArg { fun: EvalFun::Inc0, }), EvalOp::Fun(fun)) =>
                        return Err(Error::AppIncOnFun { fun, }),
                    // dec on positive number
                    (
                        Some(State::EvalAppArg { fun: EvalFun::Dec0, }),
                        EvalOp::Number {
                            number: EncodedNumber {
                                number: Number::Positive(PositiveNumber { value, }),
                                modulation,
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Number {
                            number: EncodedNumber {
                                number: if value == 0 {
                                    Number::Negative(NegativeNumber { value: -1, })
                                } else {
                                    Number::Positive(PositiveNumber { value: value - 1, })
                                },
                                modulation,
                            },
                        },
                    // dec on negative number
                    (
                        Some(State::EvalAppArg { fun: EvalFun::Dec0, }),
                        EvalOp::Number {
                            number: EncodedNumber {
                                number: Number::Negative(NegativeNumber { value, }),
                                modulation,
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Number {
                            number: EncodedNumber {
                                number: Number::Negative(NegativeNumber { value: value - 1, }),
                                modulation,
                            },
                        },
                    // dec on fun
                    (Some(State::EvalAppArg { fun: EvalFun::Dec0, }), EvalOp::Fun(fun)) =>
                        return Err(Error::AppDecOnFun { fun, }),
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum EvalOp {
    Number { number: EncodedNumber, },
    Fun(EvalFun),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EvalFun {
    Inc0,
    Dec0,
}

impl EvalOp {
    fn new(op: Op) -> EvalOp {
        match op {
            Op::Const(Const::EncodedNumber(number)) =>
                EvalOp::Number { number, },
            Op::Const(Const::Fun(Fun::Inc)) =>
                EvalOp::Fun(EvalFun::Inc0),
            Op::Const(Const::Fun(Fun::Dec)) =>
                EvalOp::Fun(EvalFun::Dec0),
            Op::Const(Const::Fun(Fun::Sum)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Mul)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Div)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Eq)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Lt)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Mod)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Dem)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Send)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Neg)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::S)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::C)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::B)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::True)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::False)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Pwr2)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::I)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Cons)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Car)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Cdr)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Nil)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::IsNil)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::LeftParen)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Comma)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::RightParen)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Vec)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Draw)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Chkb)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::MultipleDraw)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::If0)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Interact)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Modem)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Galaxy)) =>
                unimplemented!(),
            Op::Variable(var) =>
                unimplemented!(),
            Op::App =>
                unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Env,
        Ast,
        AstNode,
        Error,
        Interpreter,
        EvalFun,
        super::super::{
            code::{
                Op,
                Ops,
                Fun,
                Const,
                Number,
                Variable,
                Modulation,
                EncodedNumber,
                PositiveNumber,
                NegativeNumber,
            },
        },
    };

    #[test]
    fn ast_tree_basic() {
        let interpreter = Interpreter::new();

        assert_eq!(
            interpreter.build_tree(
                Ops(vec![]),
            ),
            Ok(Ast::Empty),
        );

        assert_eq!(
            interpreter.build_tree(
                Ops(vec![
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                ]),
            ),
            Ok(Ast::Tree(AstNode::Literal {
                value: Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 1,
                    }),
                    modulation: Modulation::Demodulated,
                })),
            })),
        );

        assert_eq!(
            interpreter.build_tree(
                Ops(vec![Op::App]),
            ),
            Err(Error::NoAppFunProvided),
        );

        assert_eq!(
            interpreter.build_tree(
                Ops(vec![Op::App, Op::App]),
            ),
            Err(Error::NoAppFunProvided),
        );

        assert_eq!(
            interpreter.build_tree(
                Ops(vec![Op::App, Op::Const(Const::Fun(Fun::Inc))]),
            ),
            Err(Error::NoAppArgProvided { fun: AstNode::Literal { value: Op::Const(Const::Fun(Fun::Inc)), }, }),
        );

        assert_eq!(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                    }),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                ]),
            ),
            Ok(Ast::Tree(AstNode::App {
                fun: Box::new(AstNode::Literal {
                    value: Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                    }),
                }),
                arg: Box::new(AstNode::Literal {
                    value: Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                }),
            })),
        );

        assert_eq!(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::App,
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                    }),
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 2,
                        }),
                    }),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                ]),
            ),
            Ok(Ast::Tree(AstNode::App {
                fun: Box::new(AstNode::App {
                    fun: Box::new(AstNode::Literal {
                        value: Op::Variable(Variable {
                            name: Number::Positive(PositiveNumber {
                                value: 1,
                            }),
                        }),
                    }),
                    arg: Box::new(AstNode::Literal {
                        value: Op::Variable(Variable {
                            name: Number::Positive(PositiveNumber {
                                value: 2,
                            }),
                        }),
                    }),
                }),
                arg: Box::new(AstNode::Literal {
                    value: Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                }),
            })),
        );
    }

    #[test]
    fn eval_basic() {
        let interpreter = Interpreter::new();

        assert_eq!(
            interpreter.eval(
                interpreter.build_tree(
                    Ops(vec![]),
                ).unwrap(),
                &mut Env::new(),
            ),
            Err(Error::EvalEmptyTree),
        );

        assert_eq!(
            interpreter.eval(
                interpreter.build_tree(
                    Ops(vec![
                        Op::Const(Const::EncodedNumber(EncodedNumber {
                            number: Number::Positive(PositiveNumber {
                                value: 1,
                            }),
                            modulation: Modulation::Demodulated,
                        })),
                    ]),
                ).unwrap(),
                &mut Env::new(),
            ),
            Ok(Ops(vec![
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 1,
                    }),
                    modulation: Modulation::Demodulated,
                })),
            ])),
        );

        assert_eq!(
            interpreter.eval(
                interpreter.build_tree(
                    Ops(vec![
                        Op::App,
                        Op::Const(Const::EncodedNumber(EncodedNumber {
                            number: Number::Positive(PositiveNumber {
                                value: 1,
                            }),
                            modulation: Modulation::Demodulated,
                        })),
                        Op::Variable(Variable {
                            name: Number::Positive(PositiveNumber {
                                value: 1,
                            }),
                        }),
                    ]),
                ).unwrap(),
                &mut Env::new(),
            ),
            Err(Error::AppOnNumber {
                number: EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 1,
                    }),
                    modulation: Modulation::Demodulated,
                },
                arg: AstNode::Literal {
                    value: Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                    }),
                },
            }),
        );
    }

    #[test]
    fn eval_fun_inc() {
        let interpreter = Interpreter::new();

        assert_eq!(
            interpreter.eval(
                interpreter.build_tree(
                    Ops(vec![
                        Op::App,
                        Op::Const(Const::Fun(Fun::Inc)),
                        Op::Const(Const::EncodedNumber(EncodedNumber {
                            number: Number::Positive(PositiveNumber {
                                value: 1,
                            }),
                            modulation: Modulation::Demodulated,
                        })),
                    ]),
                ).unwrap(),
                &mut Env::new(),
            ),
            Ok(Ops(vec![
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 2,
                    }),
                    modulation: Modulation::Demodulated,
                })),
            ])),
        );

        assert_eq!(
            interpreter.eval(
                interpreter.build_tree(
                    Ops(vec![
                        Op::App,
                        Op::Const(Const::Fun(Fun::Inc)),
                        Op::Const(Const::EncodedNumber(EncodedNumber {
                            number: Number::Negative(NegativeNumber {
                                value: -1,
                            }),
                            modulation: Modulation::Demodulated,
                        })),
                    ]),
                ).unwrap(),
                &mut Env::new(),
            ),
            Ok(Ops(vec![
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 0,
                    }),
                    modulation: Modulation::Demodulated,
                })),
            ])),
        );

        assert_eq!(
            interpreter.eval(
                interpreter.build_tree(
                    Ops(vec![
                        Op::App,
                        Op::Const(Const::Fun(Fun::Inc)),
                        Op::Const(Const::EncodedNumber(EncodedNumber {
                            number: Number::Negative(NegativeNumber {
                                value: -2,
                            }),
                            modulation: Modulation::Demodulated,
                        })),
                    ]),
                ).unwrap(),
                &mut Env::new(),
            ),
            Ok(Ops(vec![
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Negative(NegativeNumber {
                        value: -1,
                    }),
                    modulation: Modulation::Demodulated,
                })),
            ])),
        );

        assert_eq!(
            interpreter.eval(
                interpreter.build_tree(
                    Ops(vec![
                        Op::App,
                        Op::Const(Const::Fun(Fun::Inc)),
                        Op::Const(Const::Fun(Fun::Inc)),
                    ]),
                ).unwrap(),
                &mut Env::new(),
            ),
            Err(Error::AppIncOnFun { fun: EvalFun::Inc0, }),
        );
    }

    #[test]
    fn eval_fun_dec() {
        let interpreter = Interpreter::new();

        assert_eq!(
            interpreter.eval(
                interpreter.build_tree(
                    Ops(vec![
                        Op::App,
                        Op::Const(Const::Fun(Fun::Dec)),
                        Op::Const(Const::EncodedNumber(EncodedNumber {
                            number: Number::Positive(PositiveNumber {
                                value: 1,
                            }),
                            modulation: Modulation::Demodulated,
                        })),
                    ]),
                ).unwrap(),
                &mut Env::new(),
            ),
            Ok(Ops(vec![
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 0,
                    }),
                    modulation: Modulation::Demodulated,
                })),
            ])),
        );

        assert_eq!(
            interpreter.eval(
                interpreter.build_tree(
                    Ops(vec![
                        Op::App,
                        Op::Const(Const::Fun(Fun::Dec)),
                        Op::Const(Const::EncodedNumber(EncodedNumber {
                            number: Number::Negative(NegativeNumber {
                                value: -1,
                            }),
                            modulation: Modulation::Demodulated,
                        })),
                    ]),
                ).unwrap(),
                &mut Env::new(),
            ),
            Ok(Ops(vec![
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Negative(NegativeNumber {
                        value: -2,
                    }),
                    modulation: Modulation::Demodulated,
                })),
            ])),
        );

        assert_eq!(
            interpreter.eval(
                interpreter.build_tree(
                    Ops(vec![
                        Op::App,
                        Op::Const(Const::Fun(Fun::Dec)),
                        Op::Const(Const::EncodedNumber(EncodedNumber {
                            number: Number::Positive(PositiveNumber {
                                value: 0,
                            }),
                            modulation: Modulation::Demodulated,
                        })),
                    ]),
                ).unwrap(),
                &mut Env::new(),
            ),
            Ok(Ops(vec![
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Negative(NegativeNumber {
                        value: -1,
                    }),
                    modulation: Modulation::Demodulated,
                })),
            ])),
        );

        assert_eq!(
            interpreter.eval(
                interpreter.build_tree(
                    Ops(vec![
                        Op::App,
                        Op::Const(Const::Fun(Fun::Dec)),
                        Op::Const(Const::Fun(Fun::Dec)),
                    ]),
                ).unwrap(),
                &mut Env::new(),
            ),
            Err(Error::AppDecOnFun { fun: EvalFun::Dec0, }),
        );
    }
}
