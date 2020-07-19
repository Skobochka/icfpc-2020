use super::{
    Env,
    Interpreter,
    Op,
    Ops,
    Fun,
    Const,
    Number,
    Syntax,
    Variable,
    PositiveNumber,
};

#[test]
fn eval_zero() {
    let interpreter = Interpreter::new();

    // ( )   =   nil
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::Syntax(Syntax::LeftParen),
                    Op::Syntax(Syntax::RightParen),
                ]),
            ).unwrap(),
            &mut Env::new(),
        ),
        Ok(Ops(vec![
            Op::Const(Const::Fun(Fun::Nil)),
        ])),
    );
}

#[test]
fn eval_one() {
    let interpreter = Interpreter::new();

    // ( x0 )   =   ap ap cons x0 nil
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::Syntax(Syntax::LeftParen),
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 0,
                        }),
                    }),
                    Op::Syntax(Syntax::RightParen),
                ]),
            ).unwrap(),
            &mut Env::new(),
        ),
        Ok(Ops(vec![
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Variable(Variable {
                name: Number::Positive(PositiveNumber {
                    value: 0,
                }),
            }),
            Op::Const(Const::Fun(Fun::Nil)),
        ])),
    );
}

#[test]
fn eval_two() {
    let interpreter = Interpreter::new();

    // ( x0 , x1 )   =   ap ap cons x0 ap ap cons x1 nil
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::Syntax(Syntax::LeftParen),
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 0,
                        }),
                    }),
                    Op::Syntax(Syntax::Comma),
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                    }),
                    Op::Syntax(Syntax::RightParen),
                ]),
            ).unwrap(),
            &mut Env::new(),
        ),
        Ok(Ops(vec![
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Variable(Variable {
                name: Number::Positive(PositiveNumber {
                    value: 0,
                }),
            }),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Variable(Variable {
                name: Number::Positive(PositiveNumber {
                    value: 1,
                }),
            }),
            Op::Const(Const::Fun(Fun::Nil)),
        ])),
    );
}

#[test]
fn eval_three() {
    let interpreter = Interpreter::new();

    // ( x0 , x1 , x2 )   =   ap ap cons x0 ap ap cons x1 ap ap cons x2 nil
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::Syntax(Syntax::LeftParen),
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 0,
                        }),
                    }),
                    Op::Syntax(Syntax::Comma),
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                    }),
                    Op::Syntax(Syntax::Comma),
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 2,
                        }),
                    }),
                    Op::Syntax(Syntax::RightParen),
                ]),
            ).unwrap(),
            &mut Env::new(),
        ),
        Ok(Ops(vec![
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Variable(Variable {
                name: Number::Positive(PositiveNumber {
                    value: 0,
                }),
            }),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Variable(Variable {
                name: Number::Positive(PositiveNumber {
                    value: 1,
                }),
            }),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Variable(Variable {
                name: Number::Positive(PositiveNumber {
                    value: 2,
                }),
            }),
            Op::Const(Const::Fun(Fun::Nil)),
        ])),
    );
}
