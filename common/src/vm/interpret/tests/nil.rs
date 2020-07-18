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

    // ap nil x0   =   t
    assert_eq!(
        interpreter.eval(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::Nil)),
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
            Op::Const(Const::Fun(Fun::True)),
        ])),
    );
}
