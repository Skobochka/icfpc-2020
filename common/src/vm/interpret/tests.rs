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

mod inc;
mod dec;
mod sum;
mod mul;
mod div;
mod op_true;
mod op_false;
mod eq;
mod lt;
mod neg;
mod op_i;
mod op_c;
mod op_b;
mod op_s;
mod cons;
mod car;
mod cdr;

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
