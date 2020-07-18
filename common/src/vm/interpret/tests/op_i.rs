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

    // ap i x0   =   x0
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::I)),
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 0,
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

    // ap i ap add 1   =   ap add 1
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::I)),
                    Op::App,
                    Op::Const(Const::Fun(Fun::Sum)),
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
            Op::App,
            Op::Const(Const::Fun(Fun::Sum)),
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 1,
                }),
                modulation: Modulation::Demodulated,
            })),
        ])),
    );
}
