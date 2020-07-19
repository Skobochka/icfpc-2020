use super::{
    Env,
    Interpreter,
    Op,
    Ops,
    Fun,
    Const,
    Coord,
    Syntax,
    Number,
    Picture,
    Modulation,
    EncodedNumber,
    PositiveNumber,
};

#[test]
fn eval_single() {
    let interpreter = Interpreter::new();

    // ap draw ( )   =   |picture1|
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::Draw)),
                    Op::Syntax(Syntax::LeftParen),
                    Op::Syntax(Syntax::RightParen),
                ]),
            ).unwrap(),
            &mut Env::new(),
        ),
        Ok(Ops(vec![
            Op::Const(Const::Picture(Picture { points: vec![], })),
        ])),
    );

    // ap draw ( ap ap vec 1 1 )   =   |picture2|
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::Draw)),
                    Op::Syntax(Syntax::LeftParen),
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::Vec)),
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
                    Op::Syntax(Syntax::RightParen),
                ]),
            ).unwrap(),
            &mut Env::new(),
        ),
        Ok(Ops(vec![
            Op::Const(Const::Picture(Picture {
                points: vec![
                    Coord {
                        x: EncodedNumber { number: Number::Positive(PositiveNumber { value: 1, }), modulation: Modulation::Demodulated },
                        y: EncodedNumber { number: Number::Positive(PositiveNumber { value: 1, }), modulation: Modulation::Demodulated },
                    },
                ],
            })),
        ])),
    );

    // ap draw ( ap ap vec 1 2 , ap ap vec 3 1 )   =   |picture5|
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::Draw)),
                    Op::Syntax(Syntax::LeftParen),
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::Vec)),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 2,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                    Op::Syntax(Syntax::Comma),
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::Vec)),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 3,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
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
            Op::Const(Const::Picture(Picture {
                points: vec![
                    Coord {
                        x: EncodedNumber { number: Number::Positive(PositiveNumber { value: 1, }), modulation: Modulation::Demodulated },
                        y: EncodedNumber { number: Number::Positive(PositiveNumber { value: 2, }), modulation: Modulation::Demodulated },
                    },
                    Coord {
                        x: EncodedNumber { number: Number::Positive(PositiveNumber { value: 3, }), modulation: Modulation::Demodulated },
                        y: EncodedNumber { number: Number::Positive(PositiveNumber { value: 1, }), modulation: Modulation::Demodulated },
                    },
                ],
            })),
        ])),
    );

}

#[test]
fn eval_multi() {
    let interpreter = Interpreter::new();

    // ap multipledraw ( )   =   |picture1|
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::MultipleDraw)),
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

    // ap car ap cdr ap multipledraw ( ( ap ap vec 1 1 ), ( ap ap vec 1 2 , ap ap vec 3 1 ) )   =  |picture5|
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::Car)),
                    Op::App,
                    Op::Const(Const::Fun(Fun::Cdr)),

                    Op::App,
                    Op::Const(Const::Fun(Fun::MultipleDraw)),
                    Op::Syntax(Syntax::LeftParen),

                    Op::Syntax(Syntax::LeftParen),
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::Vec)),
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
                    Op::Syntax(Syntax::RightParen),

                    Op::Syntax(Syntax::Comma),

                    Op::Syntax(Syntax::LeftParen),
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::Vec)),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 2,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                    Op::Syntax(Syntax::Comma),
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::Vec)),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 3,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                    Op::Syntax(Syntax::RightParen),

                    Op::Syntax(Syntax::RightParen),
                ]),
            ).unwrap(),
            &mut Env::new(),
        ),
        Ok(Ops(vec![
            Op::Const(Const::Picture(Picture {
                points: vec![
                    Coord {
                        x: EncodedNumber { number: Number::Positive(PositiveNumber { value: 1, }), modulation: Modulation::Demodulated },
                        y: EncodedNumber { number: Number::Positive(PositiveNumber { value: 2, }), modulation: Modulation::Demodulated },
                    },
                    Coord {
                        x: EncodedNumber { number: Number::Positive(PositiveNumber { value: 3, }), modulation: Modulation::Demodulated },
                        y: EncodedNumber { number: Number::Positive(PositiveNumber { value: 1, }), modulation: Modulation::Demodulated },
                    },
                ],
            })),
        ])),
    );

}
