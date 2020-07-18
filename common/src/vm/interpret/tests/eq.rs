use super::{
    Env,
    Interpreter,
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
};

#[test]
fn eval() {
    let interpreter = Interpreter::new();

    // // temporary off
    // // ap ap eq x0 x0   =   t
    // assert_eq!(
    //     interpreter.eval(
    //         interpreter.build_tree(
    //             Ops(vec![
    //                 Op::App,
    //                 Op::App,
    //                 Op::Const(Const::Fun(Fun::Eq)),
    //                 Op::Variable(Variable {
    //                     name: Number::Positive(PositiveNumber {
    //                         value: 0,
    //                     }),
    //                 }),
    //                 Op::Variable(Variable {
    //                     name: Number::Positive(PositiveNumber {
    //                         value: 0,
    //                     }),
    //                 }),
    //             ]),
    //         ).unwrap(),
    //         &mut Env::new(),
    //     ),
    //     Ok(Ops(vec![
    //         Op::Const(Const::Fun(Fun::True)),
    //     ])),
    // );

    // ap ap eq 0 0   =   t
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::Eq)),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 0,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
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
            Op::Const(Const::Fun(Fun::True)),
        ])),
    );

    // ap ap ap eq 0 -1 x0 x1
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::App,
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::Eq)),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 0,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Negative(NegativeNumber {
                            value: -1,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 0,
                        }),
                    }),
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                    }),
                ]),
            ).unwrap(),
            &mut Env::new(),
        ),
        Ok(Ops(vec![
            Op::Variable(Variable {
                name: Number::Positive(PositiveNumber {
                    value: 1,
                }),
            }),
        ])),
    );
}
