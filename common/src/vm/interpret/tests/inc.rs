use super::{
    Env,
    Error,
    Interpreter,
    EvalFun,
    EvalFunNum,
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

    // assert_eq!(
    //     interpreter.eval(
    //         interpreter.build_tree(
    //             Ops(vec![
    //                 Op::App,
    //                 Op::Const(Const::Fun(Fun::Inc)),
    //                 Op::Variable(Variable {
    //                     name: Number::Positive(PositiveNumber {
    //                         value: 1,
    //                     }),
    //                 }),
    //             ]),
    //         ).unwrap(),
    //         &mut Env::new(),
    //     ),
    //     Ok(Ops(vec![])),
    // );
}
