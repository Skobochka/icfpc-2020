use super::{
    Env,
    Interpreter,
    Op,
    Ops,
    Fun,
    Const,
    Syntax,
    Number,
    Modulation,
    EncodedNumber,
    PositiveNumber,
    NegativeNumber,
    super::list_val_to_ops,
};

use crate::encoder::{
    ListVal,
    ConsList,
};

#[test]
fn encode() {
    let interpreter = Interpreter::new();

    assert_eq!(
        interpreter.eval_ops_to_list_val(
            Ops(vec![
                Op::Syntax(Syntax::LeftParen),
                Op::Syntax(Syntax::RightParen),
            ]),
            &Env::new(),
        ).unwrap(),
        ListVal::Cons(Box::new(ConsList::Nil)),
    );

    assert_eq!(
        interpreter.eval_ops_to_list_val(
            Ops(vec![
                Op::Syntax(Syntax::LeftParen),
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 0,
                    }),
                    modulation: Modulation::Modulated,
                })),
                Op::Syntax(Syntax::RightParen),
            ]),
            &Env::new(),
        ).unwrap(),
        ListVal::Cons(Box::new(ConsList::Cons(
            ListVal::Number(EncodedNumber {
                number: Number::Positive(PositiveNumber { value: 0, }),
                modulation: Modulation::Modulated,
            }),
            ListVal::Cons(Box::new(ConsList::Nil)),
        ))),
    );

    assert_eq!(
        interpreter.eval_ops_to_list_val(
            Ops(vec![
                Op::Syntax(Syntax::LeftParen),
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 1,
                    }),
                    modulation: Modulation::Modulated,
                })),
                Op::Syntax(Syntax::Comma),
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 1,
                    }),
                    modulation: Modulation::Modulated,
                })),
                Op::Syntax(Syntax::RightParen),
            ]),
            &Env::new(),
        ).unwrap(),
        ListVal::Cons(Box::new(ConsList::Cons(
            ListVal::Number(EncodedNumber {
                number: Number::Positive(PositiveNumber { value: 1, }),
                modulation: Modulation::Modulated,
            }),
            ListVal::Cons(Box::new(ConsList::Cons(
                ListVal::Number(EncodedNumber {
                    number: Number::Positive(PositiveNumber { value: 1, }),
                    modulation: Modulation::Modulated,
                }),
                ListVal::Cons(Box::new(ConsList::Nil)),
            ))),
        ))),
    );

    assert_eq!(
        interpreter.eval_ops_to_list_val(
            Ops(vec![
                Op::Syntax(Syntax::LeftParen),
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 1,
                    }),
                    modulation: Modulation::Modulated,
                })),
                Op::Syntax(Syntax::Comma),
                Op::Syntax(Syntax::LeftParen),
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Negative(NegativeNumber {
                        value: -1,
                    }),
                    modulation: Modulation::Modulated,
                })),
                Op::Syntax(Syntax::RightParen),
                Op::Syntax(Syntax::RightParen),
            ]),
            &Env::new(),
        ).unwrap(),
        ListVal::Cons(Box::new(ConsList::Cons(
            ListVal::Number(EncodedNumber {
                number: Number::Positive(PositiveNumber { value: 1, }),
                modulation: Modulation::Modulated,
            }),
            ListVal::Cons(Box::new(ConsList::Cons(
                ListVal::Cons(Box::new(ConsList::Cons(
                    ListVal::Number(EncodedNumber {
                        number: Number::Negative(NegativeNumber { value: -1, }),
                        modulation: Modulation::Modulated,
                    }),
                    ListVal::Cons(Box::new(ConsList::Nil)),
                ))),
                ListVal::Cons(Box::new(ConsList::Nil))))),
        ))),
    );
}

#[test]
fn decode() {
    assert_eq!(
        list_val_to_ops(
            ListVal::Cons(Box::new(ConsList::Nil)),
        ),
        Ops(vec![
            Op::Const(Const::Fun(Fun::Nil)),
        ]),
    );

    let interpreter = Interpreter::new();

    let cons_list = ListVal::Cons(Box::new(ConsList::Cons(
        ListVal::Number(EncodedNumber {
            number: Number::Positive(PositiveNumber { value: 1, }),
            modulation: Modulation::Modulated,
        }),
        ListVal::Cons(Box::new(ConsList::Cons(
            ListVal::Number(EncodedNumber {
                number: Number::Positive(PositiveNumber { value: 1, }),
                modulation: Modulation::Modulated,
            }),
            ListVal::Cons(Box::new(ConsList::Nil)),
        ))),
    )));
    assert_eq!(
        interpreter.eval_ops_to_list_val(list_val_to_ops(cons_list.clone()), &Env::new()).unwrap(),
        cons_list,
    );

    let cons_list = ListVal::Cons(Box::new(ConsList::Cons(
        ListVal::Number(EncodedNumber {
            number: Number::Positive(PositiveNumber { value: 1, }),
            modulation: Modulation::Modulated,
        }),
        ListVal::Cons(Box::new(ConsList::Cons(
            ListVal::Cons(Box::new(ConsList::Cons(
                ListVal::Number(EncodedNumber {
                    number: Number::Negative(NegativeNumber { value: -1, }),
                    modulation: Modulation::Modulated,
                }),
                ListVal::Cons(Box::new(ConsList::Nil)),
            ))),
            ListVal::Cons(Box::new(ConsList::Nil))))),
    )));
    assert_eq!(
        interpreter.eval_ops_to_list_val(list_val_to_ops(cons_list.clone()), &Env::new()).unwrap(),
        cons_list,
    );
}
