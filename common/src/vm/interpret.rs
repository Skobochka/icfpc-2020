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
    NoAppFunProvided,
    NoAppArgProvided { fun: AstNode, },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ast {
    Empty,
    Tree(AstNode),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum AstNode {
    Literal { value: Op, },
    App { fun: Box<AstNode>, arg: Box<AstNode>, },
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {

        }
    }

    pub fn build_tree(&self, Ops(ops): Ops, _env: &mut Env) -> Result<Ast, Error> {
        enum State {
            AwaitAppFun,
            AwaitAppArg { fun: AstNode, },
        }

        let mut states = vec![];
        let mut ops_iter = ops.into_iter();
        loop {
            let mut maybe_node: Option<AstNode> = match ops_iter.next() {
                None =>
                    None,
                Some(value @ Op::Const(..)) |
                Some(value @ Op::Variable(..)) =>
                    Some(AstNode::Literal { value: value, }),
                Some(Op::App) => {
                    states.push(State::AwaitAppFun);
                    continue;
                },
            };

            loop {
                match (states.pop(), maybe_node) {
                    (None, None) =>
                        return Ok(Ast::Empty),
                    (None, Some(node)) =>
                        return Ok(Ast::Tree(node)),
                    (Some(State::AwaitAppFun), None) =>
                        return Err(Error::NoAppFunProvided),
                    (Some(State::AwaitAppFun), Some(node)) => {
                        states.push(State::AwaitAppArg { fun: node, });
                        break;
                    },
                    (Some(State::AwaitAppArg { fun, }), None) =>
                        return Err(Error::NoAppArgProvided { fun, }),
                    (Some(State::AwaitAppArg { fun, }), Some(node)) => {
                        maybe_node = Some(AstNode::App {
                            fun: Box::new(fun),
                            arg: Box::new(node),
                        });
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
        Ast,
        AstNode,
        Error,
        Interpreter,
        super::super::{
            code::{
                Op,
                Ops,
                Fun,
                Const,
                Number,
                Variable,
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
            interpreter.build_tree(
                Ops(vec![]),
                &mut Env::new(),
            ),
            Ok(Ast::Empty),
        );

        assert_eq!(
            interpreter.build_tree(
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
            Ok(Ast::Tree(AstNode::Literal {
                value: Op::Const(Const::EncodedNumber(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 1,
                    }),
                    modulation: Modulation::Demodulated,
                })),
            })),
        );

        assert_eq!(
            interpreter.build_tree(
                Ops(vec![Op::App]),
                &mut Env::new(),
            ),
            Err(Error::NoAppFunProvided),
        );

        assert_eq!(
            interpreter.build_tree(
                Ops(vec![Op::App, Op::App]),
                &mut Env::new(),
            ),
            Err(Error::NoAppFunProvided),
        );

        assert_eq!(
            interpreter.build_tree(
                Ops(vec![Op::App, Op::Const(Const::Fun(Fun::Inc))]),
                &mut Env::new(),
            ),
            Err(Error::NoAppArgProvided { fun: AstNode::Literal { value: Op::Const(Const::Fun(Fun::Inc)), }, }),
        );

        assert_eq!(
            interpreter.build_tree(
                Ops(vec![
                    Op::App,
                    Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                    }),
                    Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                ]),
                &mut Env::new(),
            ),
            Ok(Ast::Tree(AstNode::App {
                fun: Box::new(AstNode::Literal {
                    value: Op::Variable(Variable {
                        name: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                    }),
                }),
                arg: Box::new(AstNode::Literal {
                    value: Op::Const(Const::EncodedNumber(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 1,
                        }),
                        modulation: Modulation::Demodulated,
                    })),
                }),
            })),
        );
    }
}
