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
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(x0).unwrap(),
            &env,
        ).unwrap(),
        one,
    );
}

#[test]
fn subst_arg_inc() {
    // :1162 = 1
    // ap inc :1162 = :0
    // :0 = 2

    let result = Ops(vec![
        Op::Variable(Variable {
            name: Number::Positive(PositiveNumber {
                value: 0,
            }),
        }),
    ]);
    let script = Script {
        statements: vec![
            Statement::Equality(Equality {
                left: Ops(vec![
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 1162,
                        }),
                    }),
                ]),
                right: Ops(vec![
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                ]),
            }),
            Statement::Equality(Equality {
                left: Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::Inc)),
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 1162,
                        }),
                    }),
                ]),
                right: result.clone(),
            }),
        ],
    };

    let interpreter = Interpreter::new();
    let env = interpreter.eval_script(script).unwrap();
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(result).unwrap(),
            &env,
        ).unwrap(),
        Ops(vec![
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 2,
                }),
                modulation: Modulation::Demodulated,
            })),
        ]),
    );
}

#[test]
fn subst_arg_is_nil() {
    // :1162 = nil
    // ap isnil :1162 = :0
    // :0 = t

    let result = Ops(vec![
        Op::Variable(Variable {
            name: Number::Positive(PositiveNumber {
                value: 0,
            }),
        }),
    ]);
    let script = Script {
        statements: vec![
            Statement::Equality(Equality {
                left: Ops(vec![
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 1162,
                        }),
                    }),
                ]),
                right: Ops(vec![
                    Op::Const(Const::Fun(Fun::Nil)),
                ]),
            }),
            Statement::Equality(Equality {
                left: Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::IsNil)),
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 1162,
                        }),
                    }),
                ]),
                right: result.clone(),
            }),
        ],
    };

    let interpreter = Interpreter::new();
    let env = interpreter.eval_script(script).unwrap();
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(result).unwrap(),
            &env,
        ).unwrap(),
        Ops(vec![
            Op::Const(Const::Fun(Fun::True)),
        ]),
    );
}

#[test]
fn two_subst_arg_is_nil() {
    // :1162 = nil
    // :0 = ap isnil :1162
    // :1 = :0
    // :1 = t

    let result = Ops(vec![
        Op::Variable(Variable {
            name: Number::Positive(PositiveNumber {
                value: 1,
            }),
        }),
    ]);
    let script = Script {
        statements: vec![
            Statement::Equality(Equality {
                left: Ops(vec![
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 1162,
                        }),
                    }),
                ]),
                right: Ops(vec![
                    Op::Const(Const::Fun(Fun::Nil)),
                ]),
            }),
            Statement::Equality(Equality {
                left: Ops(vec![
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 0,
                        }),
                    }),
                ]),
                right: Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::IsNil)),
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 1162,
                        }),
                    }),
                ]),
            }),
            Statement::Equality(Equality {
                left: result.clone(),
                right: Ops(vec![
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 0,
                        }),
                    }),
                ]),
            }),
        ],
    };

    let interpreter = Interpreter::new();
    let env = interpreter.eval_script(script).unwrap();
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(result).unwrap(),
            &env,
        ).unwrap(),
        Ops(vec![
            Op::Const(Const::Fun(Fun::True)),
        ]),
    );
}

#[test]
fn eval_video_1() {
    // :0 = ap :2048 42
    // :2048 = ap f :2048
    // result: 42

    let result = Ops(vec![
        Op::Variable(Variable {
            name: Number::Positive(PositiveNumber {
                value: 0,
            }),
        }),
    ]);
    let script = Script {
        statements: vec![
            Statement::Equality(Equality {
                left: result.clone(),
                right: Ops(vec![
                    Op::App,
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 2048,
                        }),
                    }),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 42,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                ]),
            }),
            Statement::Equality(Equality {
                left: Ops(vec![
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 2048,
                        }),
                    }),
                ]),
                right: Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::False)),
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 2048,
                        }),
                    }),
                ]),
            }),
        ],
    };

    let interpreter = Interpreter::new();
    let env = interpreter.eval_script(script).unwrap();
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(result).unwrap(),
            &env,
        ).unwrap(),
        Ops(vec![
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 42,
                }),
                modulation: Modulation::Demodulated,
            })),
        ]),
    );
}
