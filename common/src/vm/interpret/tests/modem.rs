use super::{
    Env,
    Error,
    Interpreter,
    Op,
    Ops,
    Fun,
    Const,
    Number,
    Syntax,
    Modulation,
    EncodedNumber,
    PositiveNumber,
};

#[test]
fn eval_num() {
    let interpreter = Interpreter::new();

    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::Mod)),
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
                modulation: Modulation::Modulated,
            })),
        ])),
    );

    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::Dem)),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                        modulation: Modulation::Modulated,
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
                    Op::Const(Const::Fun(Fun::Dem)),
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
        Err(Error::DemOnDemodulatedNumber {
            number: EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 1,
                }),
                modulation: Modulation::Demodulated,
            },
        }),
    );
}

#[test]
fn eval_list() {
    let interpreter = Interpreter::new();

    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::Mod)),
                    Op::Syntax(Syntax::LeftParen),
                    Op::Syntax(Syntax::RightParen),
                ]),
            ).unwrap(),
            &mut Env::new(),
        ),
        Ok(Ops(vec![
            Op::Const(Const::Fun(Fun::Nil)),
        ])),
    );

    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::Mod)),
                    Op::Syntax(Syntax::LeftParen),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                    Op::Syntax(Syntax::RightParen),
                ]),
            ).unwrap(),
            &mut Env::new(),
        ),
        Ok(Ops(vec![
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 1,
                }),
                modulation: Modulation::Modulated,
            })),
            Op::Const(Const::Fun(Fun::Nil)),
        ])),
    );

    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::Modem)),
                    Op::Syntax(Syntax::LeftParen),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                    Op::Syntax(Syntax::Comma),
                    Op::Syntax(Syntax::LeftParen),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 2,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
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
            ).unwrap(),
            &mut Env::new(),
        ),
        Ok(Ops(vec![
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 1,
                }),
                modulation: Modulation::Demodulated,
            })),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 2,
                }),
                modulation: Modulation::Demodulated,
            })),
            Op::Const(Const::Fun(Fun::Nil)),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 3,
                }),
                modulation: Modulation::Demodulated,
            })),
            Op::Const(Const::Fun(Fun::Nil)),
        ])),
    );
}
