use super::{
    Env,
    Ast,
    Cache,
    AstNode,
    AstNodeH,
    Error,
    Interpreter,
    super::super::{
        code::{
            Op,
            Ops,
            Fun,
            Const,
            Coord,
            Number,
            Syntax,
            Picture,
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
mod nil;
mod is_nil;
mod if_zero;
mod list;
mod draw;
mod modem;
mod interact;
mod env;
mod various;
mod encoder;

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
        ).unwrap().render(),
        Ops(vec![
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 1,
                }),
                modulation: Modulation::Demodulated,
            })),
        ]),
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
        Err(Error::NoAppArgProvided {
            fun: std::rc::Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Inc)), })).render(),
        }),
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
            arg: std::rc::Rc::new(AstNodeH::new(AstNode::Literal {
                value: Op::Variable(Variable {
                    name: Number::Positive(PositiveNumber {
                        value: 1,
                    }),
                }),
            })).render(),
        }),
    );
}
