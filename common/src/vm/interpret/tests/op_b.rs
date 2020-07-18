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
};

#[test]
fn eval() {
    let interpreter = Interpreter::new();

    // ap ap ap b inc dec 1   =   1
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::B)),
                    Op::Const(Const::Fun(Fun::Inc)),
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
                    value: 1,
                }),
                modulation: Modulation::Demodulated,
            })),
        ])),
    );

    // ap ap ap b x0 x1 x2   =   ap x0 ap x1 x2
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::B)),
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
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 2,
                        }),
                    }),
                ]),
            ).unwrap(),
            &mut Env::new(),
        ),
        Ok(Ops(vec![
            Op::App,
            Op::Variable(Variable {
                name: Number::Positive(PositiveNumber {
                    value: 0,
                }),
            }),
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
        ])),
    );
}
