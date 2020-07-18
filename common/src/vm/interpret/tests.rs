use super::{
    Env,
    Ast,
    AstNode,
    Error,
    Interpreter,
    EvalFun,
    EvalFunNum,
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
        Err(Error::AppExpectsNumButFunProvided { fun: EvalFun::ArgNum(EvalFunNum::Inc0), }),
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
        Err(Error::AppExpectsNumButFunProvided { fun: EvalFun::ArgNum(EvalFunNum::Dec0), }),
    );
}

#[test]
fn eval_fun_sum() {
    let interpreter = Interpreter::new();

    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::Sum)),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 3,
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
                    value: 4,
                }),
                modulation: Modulation::Demodulated,
            })),
        ])),
    );
}
