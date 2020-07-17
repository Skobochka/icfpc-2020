use super::{
    super::code::{
        Op,
        Ops,
    },
    Env,
};

pub struct Interpreter {

}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Error {
    EmptyInput,
    NoAppFunProvided,
    NoAppArgProvided { fun: Ops, },
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {

        }
    }

    pub fn interpret(&self, Ops(ops): Ops, _env: &mut Env) -> Result<Ops, Error> {
        enum State {
            AwaitAppFun,
            AwaitAppArg { fun: Ops, },
        }

        let mut states = vec![];
        let mut ops_iter = ops.into_iter();
        loop {
            let mut result: Option<Ops> = match ops_iter.next() {
                None =>
                    None,
                Some(value @ Op::Const(..)) |
                Some(value @ Op::Variable(..)) =>
                    Some(Ops(vec![value])),
                Some(Op::App) => {
                    states.push(State::AwaitAppFun);
                    continue;
                },
            };

            loop {
                match (states.pop(), result) {
                    (None, None) =>
                        return Err(Error::EmptyInput),
                    (None, Some(ops)) =>
                        return Ok(ops),
                    (Some(State::AwaitAppFun), None) =>
                        return Err(Error::NoAppFunProvided),
                    (Some(State::AwaitAppFun), Some(ops)) => {
                        states.push(State::AwaitAppArg { fun: ops, });
                        break;
                    },
                    (Some(State::AwaitAppArg { fun, }), None) =>
                        return Err(Error::NoAppArgProvided { fun, }),
                    (Some(State::AwaitAppArg { fun, }), Some(ops)) => {
                        let mut app_ops = Vec::with_capacity(1 + fun.0.len() + ops.0.len());
                        app_ops.push(Op::App);
                        app_ops.extend(fun.0);
                        app_ops.extend(ops.0);
                        result = Some(Ops(app_ops));
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Env,
        Interpreter,
        Error,
        super::super::{
            code::{
                Op,
                Ops,
                Fun,
                Const,
                Number,
                Modulation,
                EncodedNumber,
                PositiveNumber,
            },
        },
    };

    #[test]
    fn interpret_basic() {
        let interpreter = Interpreter::new();

        assert_eq!(
            interpreter.interpret(
                Ops(vec![]),
                &mut Env::new(),
            ),
            Err(Error::EmptyInput),
        );

        assert_eq!(
            interpreter.interpret(
                Ops(vec![
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                ]),
                &mut Env::new(),
            ),
            Ok(Ops(vec![
                Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 1,
                    }),
                    modulation: Modulation::Demodulated,
                })),
            ])),
        );

        assert_eq!(
            interpreter.interpret(
                Ops(vec![Op::App]),
                &mut Env::new(),
            ),
            Err(Error::NoAppFunProvided),
        );

        assert_eq!(
            interpreter.interpret(
                Ops(vec![Op::App, Op::App]),
                &mut Env::new(),
            ),
            Err(Error::NoAppFunProvided),
        );

        assert_eq!(
            interpreter.interpret(
                Ops(vec![Op::App, Op::Const(Const::Fun(Fun::Inc))]),
                &mut Env::new(),
            ),
            Err(Error::NoAppArgProvided { fun: Ops(vec![Op::Const(Const::Fun(Fun::Inc))]), }),
        );
    }
}
