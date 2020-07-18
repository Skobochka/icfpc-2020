use super::{
    Env,
    Interpreter,
    Op,
    Ops,
    Fun,
    Const,
    Number,
    Variable,
    PositiveNumber,
};

#[test]
fn eval() {
    let interpreter = Interpreter::new();

    // ap ap ap cons x0 x1 x2   =   ap ap x2 x0 x1
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::Cons)),
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
            Op::App,
            Op::Variable(Variable {
                name: Number::Positive(PositiveNumber {
                    value: 2,
                }),
            }),
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
        ])),
    );

    // ap ap ap vec x0 x1 x2   =   ap ap x2 x0 x1
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::Vec)),
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
            Op::App,
            Op::Variable(Variable {
                name: Number::Positive(PositiveNumber {
                    value: 2,
                }),
            }),
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
        ])),
    );

}
