use super::{
    Env,
    Cache,
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
fn eval_huge() {
    let interpreter = Interpreter::new();

    // ap ap b ap b ap cons 2 ap ap c ap ap b b cons ap ap c cons nil
    // (ap (ap b (ap b (ap cons 2))) (ap (ap c (ap (ap b b) cons)) (ap (ap c cons) nil)))
    let tree = interpreter.build_tree(
        Ops(vec![
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::B)),
            Op::App,
            Op::Const(Const::Fun(Fun::B)),
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 2,
                }),
                modulation: Modulation::Demodulated,
            })),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::C)),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::B)),
            Op::Const(Const::Fun(Fun::B)),
            Op::Const(Const::Fun(Fun::Cons)),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::C)),
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::Fun(Fun::Nil)),
        ]),
    ).unwrap();

    assert_eq!(
        interpreter.eval(
            tree,
            &mut Env::new(),
        ),
        Ok(Ops(vec![
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::B)),
            Op::App,
            Op::Const(Const::Fun(Fun::B)),
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 2,
                }),
                modulation: Modulation::Demodulated,
            })),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::C)),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::B)),
            Op::Const(Const::Fun(Fun::B)),
            Op::Const(Const::Fun(Fun::Cons)),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::C)),
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::Fun(Fun::Nil)),
        ])),
    );

}

#[test]
fn eval_partial() {
    let interpreter = Interpreter::new();

    // ap add 1
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::Sum)),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 2,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                ]),
            ).unwrap(),
            &mut Env::new(),
        ),
        Ok(Ops(vec![
            Op::App,
            Op::Const(Const::Fun(Fun::Sum)),
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 2,
                }),
                modulation: Modulation::Demodulated,
            })),
        ])),
    );

    // ap cons nil
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::Cons)),
                    Op::Const(Const::Fun(Fun::Nil)),
                ]),
            ).unwrap(),
            &mut Env::new(),
        ),
        Ok(Ops(vec![
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::Fun(Fun::Nil)),
        ])),
    );
}

#[test]
fn eval_force_list() {
    let interpreter = Interpreter::new();

    assert_eq!(
        interpreter.eval_force_list(
            Ops(vec![
                Op::Syntax(Syntax::LeftParen),
                Op::App,
                Op::Const(Const::Fun(Fun::Inc)),
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 2,
                    }),
                    modulation: Modulation::Demodulated,
                })),
                Op::Syntax(Syntax::Comma),
                Op::App,
                Op::App,
                Op::Const(Const::Fun(Fun::Sum)),
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
                Op::Syntax(Syntax::RightParen),
            ]),
            &Env::new(),
            &mut Cache::new(),
        ),
        Ok(Ops(vec![
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 3,
                }),
                modulation: Modulation::Demodulated,
            })),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 7,
                }),
                modulation: Modulation::Demodulated,
            })),
            Op::Const(Const::Fun(Fun::Nil)),
        ])),
    );
}

#[test]
fn eval_force_list_x() {
    let interpreter = Interpreter::new();

    assert_eq!(
        interpreter.eval_force_list(
            Ops(vec![
                Op::App,
                Op::App,
                Op::Const(Const::Fun(Fun::Cons)),
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Positive(PositiveNumber { value: 5 }), modulation: Modulation::Demodulated,
                })),
                Op::App,
                Op::App,
                Op::Const(Const::Fun(Fun::Cons)),
                Op::App,
                Op::App,
                Op::Const(Const::Fun(Fun::Cons)),
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Positive(PositiveNumber { value: 2 }), modulation: Modulation::Demodulated,
                })),
                Op::App,
                Op::App,
                Op::Const(Const::Fun(Fun::Cons)),
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Positive(PositiveNumber { value: 0 }), modulation: Modulation::Demodulated,
                })),
                Op::App,
                Op::App,
                Op::Const(Const::Fun(Fun::Cons)),
                Op::Const(Const::Fun(Fun::Nil)),
                Op::App,
                Op::App,
                Op::Const(Const::Fun(Fun::Cons)),
                Op::Const(Const::Fun(Fun::Nil)),
                Op::App,
                Op::App,
                Op::Const(Const::Fun(Fun::Cons)),
                Op::Const(Const::Fun(Fun::Nil)),
                Op::App,
                Op::App,
                Op::Const(Const::Fun(Fun::Cons)),
                Op::Const(Const::Fun(Fun::Nil)),
                Op::App,
                Op::App,
                Op::Const(Const::Fun(Fun::Cons)),
                Op::Const(Const::Fun(Fun::Nil)),
                Op::App,
                Op::App,
                Op::Const(Const::Fun(Fun::Cons)),
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Positive(PositiveNumber { value: 12216 }), modulation: Modulation::Demodulated,
                })),
                Op::Const(Const::Fun(Fun::Nil)),
                Op::App,
                Op::App,
                Op::Const(Const::Fun(Fun::Cons)),
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Positive(PositiveNumber { value: 1 }), modulation: Modulation::Demodulated,
                })),
                Op::App,
                Op::App,
                Op::Const(Const::Fun(Fun::Cons)),
                Op::Const(Const::Fun(Fun::Nil)),
                Op::Const(Const::Fun(Fun::Nil)),
            ]),
            &Env::new(),
            &mut Cache::new(),
        ),
        Ok(Ops(vec![
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber { value: 5 }), modulation: Modulation::Demodulated,
            })),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber { value: 2 }), modulation: Modulation::Demodulated,
            })),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber { value: 0 }), modulation: Modulation::Demodulated,
            })),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::Fun(Fun::Nil)),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::Fun(Fun::Nil)),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::Fun(Fun::Nil)),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::Fun(Fun::Nil)),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::Fun(Fun::Nil)),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber { value: 12216 }), modulation: Modulation::Demodulated,
            })),
            Op::Const(Const::Fun(Fun::Nil)),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber { value: 1 }), modulation: Modulation::Demodulated,
            })),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::Fun(Fun::Nil)),
            Op::Const(Const::Fun(Fun::Nil)),
        ])),
    );
}
