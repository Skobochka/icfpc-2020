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
}
