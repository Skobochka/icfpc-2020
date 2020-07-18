use crate::code::{
    Op,
    Ops,
    Fun,
    Const,
    Number,
    Variable,
    PositiveNumber,
    Script,
    Statement,
    Equality,
    Modulation,
    EncodedNumber,
};
use crate::vm::{
    interpret::{
        Interpreter,
        Env,
    },
};

#[test]
fn simple() {
    // x0
    let x0 = Ops(vec![
        Op::Variable(Variable {
            name: Number::Positive(PositiveNumber {
                value: 0,
            }),
        }),
    ]);

    // 1
    let one = Ops(vec![
        Op::Const(Const::EncodedNumber(EncodedNumber {
            number: Number::Positive(PositiveNumber {
                value: 1,
            }),
            modulation: Modulation::Demodulated,
        })),
    ]);

    let script = Script {
        statements: vec![
            // 1 = x0
            Statement::Equality(Equality {
                left: one.clone(),
                right: x0.clone(),
            }),
        ]
    };

    let interpreter = Interpreter::new();
    let env = interpreter.eval_script(script).unwrap();
    let x0_value = env.lookup(x0);
    assert_eq!(x0_value, Some(one));
}
