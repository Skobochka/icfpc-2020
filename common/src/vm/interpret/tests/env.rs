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
    let x0_value = interpreter.lookup_env(&env, x0).unwrap();
    assert_eq!(x0_value, Some(one));
}

#[test]
fn galaxy_head() {
    // :1162 = cons
    let x1162_var = Op::Variable(Variable {
        name: Number::Positive(PositiveNumber {
            value: 1162,
        }),
    });
    let x1162 = Statement::Equality(Equality {
        left: Ops(vec![x1162_var.clone()]),
        right: Ops(vec![
            Op::Const(Const::Fun(Fun::Cons)),
        ]),
    });

    let num_7 = Op::Const(Const::EncodedNumber(EncodedNumber {
        number: Number::Positive(PositiveNumber {
            value: 7,
        }),
        modulation: Modulation::Demodulated,
    }));

    let num_123229502148636 = Op::Const(Const::EncodedNumber(EncodedNumber {
        number: Number::Positive(PositiveNumber {
            value: 123229502148636,
        }),
        modulation: Modulation::Demodulated,
    }));

    // :1029 = ap ap :1162 7 ap ap :1162 123229502148636 nil
    let x1029_rhs = Ops(vec![
        Op::App,
        Op::App,
        x1162_var.clone(),
        num_7.clone(),
        Op::App,
        Op::App,
        x1162_var.clone(),
        num_123229502148636.clone(),
        Op::Const(Const::Fun(Fun::Nil)),
    ]);
    let x1029_var = Op::Variable(Variable {
        name: Number::Positive(PositiveNumber {
            value: 1029,
    })});
    let x1029_lhs = Ops(vec![x1029_var.clone()]);
    let x1029 = Statement::Equality(Equality {
        left: x1029_lhs.clone(),
        right: x1029_rhs.clone(),
    });

    let script = Script {
        statements: vec![
            x1162,
            x1029,
        ]
    };

    let interpreter = Interpreter::new();
    let env = interpreter.eval_script(script).unwrap();
    let x1029_value_1 = interpreter.lookup_env(&env, x1029_lhs.clone()).unwrap();

    // :1029 = ap ap cons 7 ap ap cons 123229502148636 nil
    let x1029_rhs = Ops(vec![
        Op::App,
        Op::App,
        Op::Const(Const::Fun(Fun::Cons)),
        num_7.clone(),
        Op::App,
        Op::App,
        Op::Const(Const::Fun(Fun::Cons)),
        num_123229502148636.clone(),
        Op::Const(Const::Fun(Fun::Nil)),
    ]);
    let x1029 = Statement::Equality(Equality {
        left: x1029_lhs.clone(),
        right: x1029_rhs.clone(),
    });

    let script = Script {
        statements: vec![
            x1029,
        ]
    };

    let interpreter = Interpreter::new();
    let env = interpreter.eval_script(script).unwrap();
    let x1029_value_2 = interpreter.lookup_env(&env, x1029_lhs.clone()).unwrap();

    assert_eq!(x1029_value_1, x1029_value_2);
}
