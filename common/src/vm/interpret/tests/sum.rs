use super::{
    Env,
    Error,
    Interpreter,
    Op,
    Ops,
    Fun,
    Const,
    Number,
    Modulation,
    EncodedNumber,
    PositiveNumber,
    NegativeNumber,
};

#[test]
fn eval() {
    let interpreter = Interpreter::new();

    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
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

    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::Sum)),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Negative(NegativeNumber {
                            value: -1,
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
                    Op::App,
                    Op::Const(Const::Fun(Fun::Sum)),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                        modulation: Modulation::Modulated,
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
        Err(Error::TwoNumbersOpInDifferentModulation {
            number_a: EncodedNumber {
                number: Number::Positive(PositiveNumber { value: 1, }), modulation: Modulation::Modulated,
            },
            number_b: EncodedNumber {
                number: Number::Positive(PositiveNumber { value: 3, }), modulation: Modulation::Demodulated,
            },
        }),
    );
}

#[test]
fn eval_long() {
    let interpreter = Interpreter::new();

    // ap ap add ap ap add 2 3 4 = 9
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
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
                ]),
            ).unwrap(),
            &mut Env::new(),
        ),
        Ok(Ops(vec![
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 9,
                }),
                modulation: Modulation::Demodulated,
            })),
        ])),
    );
}
