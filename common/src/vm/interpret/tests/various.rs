use super::{
    Env,
    Interpreter,
    Op,
    Ops,
    Fun,
    Const,
    Number,
    Modulation,
    EncodedNumber,
    PositiveNumber,
};

#[test]
fn eval() {
    let interpreter = Interpreter::new();

    // ap ap b ap b ap cons 2 ap ap c ap ap b b cons ap ap c cons nil
    // (ap (ap b (ap b (ap cons 2))) (ap (ap c (ap (ap b b) cons)) (ap (ap c cons) nil)))
    let tree = interpreter.build_tree(
        Ops(vec![
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::B)),
            Op::App,
            Op::Const(Const::Fun(Fun::B)),
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 2,
                }),
                modulation: Modulation::Demodulated,
            })),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::C)),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::B)),
            Op::Const(Const::Fun(Fun::B)),
            Op::Const(Const::Fun(Fun::Cons)),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::C)),
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::Fun(Fun::Nil)),
        ]),
    ).unwrap();

    assert_eq!(
        interpreter.eval(
            tree,
            &mut Env::new(),
        ),
        Ok(Ops(vec![
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::B)),
            Op::App,
            Op::Const(Const::Fun(Fun::B)),
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::EncodedNumber(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 2,
                }),
                modulation: Modulation::Demodulated,
            })),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::C)),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::B)),
            Op::Const(Const::Fun(Fun::B)),
            Op::Const(Const::Fun(Fun::Cons)),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::C)),
            Op::Const(Const::Fun(Fun::Cons)),
            Op::Const(Const::Fun(Fun::Nil)),
        ])),
    );

}
