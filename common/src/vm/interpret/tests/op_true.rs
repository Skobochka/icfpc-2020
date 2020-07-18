use super::{
    Env,
    Error,
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

    // ap ap t x0 x1   =   x0
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::True)),
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
                    value: 0,
                }),
            }),
        ])),
    );

    // ap ap ap t inc x1 1   =   2
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::True)),
                    Op::Const(Const::Fun(Fun::Inc)),
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

}
