use super::{
    Env,
    Interpreter,
    Op,
    Ops,
    Const,
    Syntax,
    Number,
    Modulation,
    EncodedNumber,
    PositiveNumber,
    NegativeNumber,
};

use crate::encoder::{
    ListVal,
    ConsList,
};

#[test]
fn eval() {
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
                        value: 1,
                    }),
                    modulation: Modulation::Demodulated,
                })),
                Op::Syntax(Syntax::Comma),
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 1,
                    }),
                    modulation: Modulation::Demodulated,
                })),
                Op::Syntax(Syntax::RightParen),
            ]),
            &Env::new(),
        ).unwrap(),
        ListVal::Cons(Box::new(ConsList::Cons(
            ListVal::Number(EncodedNumber {
                number: Number::Positive(PositiveNumber { value: 1, }),
                modulation: Modulation::Demodulated,
            }),
            ListVal::Cons(Box::new(ConsList::Cons(
                ListVal::Number(EncodedNumber {
                    number: Number::Positive(PositiveNumber { value: 1, }),
                    modulation: Modulation::Demodulated,
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
                    modulation: Modulation::Demodulated,
                })),
                Op::Syntax(Syntax::Comma),
                Op::Syntax(Syntax::LeftParen),
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Negative(NegativeNumber {
                        value: -1,
                    }),
                    modulation: Modulation::Demodulated,
                })),
                Op::Syntax(Syntax::RightParen),
                Op::Syntax(Syntax::RightParen),
            ]),
            &Env::new(),
        ).unwrap(),
        ListVal::Cons(Box::new(ConsList::Cons(
            ListVal::Number(EncodedNumber {
                number: Number::Positive(PositiveNumber { value: 1, }),
                modulation: Modulation::Demodulated,
            }),
            ListVal::Cons(Box::new(ConsList::Cons(
                ListVal::Cons(Box::new(ConsList::Cons(
                    ListVal::Number(EncodedNumber {
                        number: Number::Negative(NegativeNumber { value: -1, }),
                        modulation: Modulation::Demodulated,
                    }),
                    ListVal::Cons(Box::new(ConsList::Nil)),
                ))),
                ListVal::Cons(Box::new(ConsList::Nil))))),
        ))),
    );
}
