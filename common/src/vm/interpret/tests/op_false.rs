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

    // ap ap f x0 x1   =   x1
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::False)),
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

    // ap ap ap f inc dec 1   =   0
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::False)),
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
                    value: 0,
                }),
                modulation: Modulation::Demodulated,
            })),
        ])),
    );
}
