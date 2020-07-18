use super::{
    super::code::{
        Op,
        Ops,
        Fun,
        Const,
        Number,
        Modulation,
        EncodedNumber,
        PositiveNumber,
        NegativeNumber,
    },
    Env,
};

#[cfg(test)]
mod tests;

pub struct Interpreter {

}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Error {
    NoAppFunProvided,
    NoAppArgProvided { fun: AstNode, },
    EvalEmptyTree,
    AppOnNumber { number: EncodedNumber, arg: AstNode, },
    AppExpectsNumButFunProvided { fun: EvalFun, },
    TwoNumbersOpInDifferentModulation { number_a: EncodedNumber, number_b: EncodedNumber, },
    DivisionByZero,
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

    pub fn build_tree(&self, Ops(ops): Ops) -> Result<Ast, Error> {
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

    pub fn eval(&self, ast: Ast, env: &mut Env) -> Result<Ops, Error> {
        match ast {
            Ast::Empty =>
                Err(Error::EvalEmptyTree),
            Ast::Tree(node) =>
                self.eval_tree(node, env),
        }
    }

    fn eval_tree(&self, mut ast_node: AstNode, _env: &mut Env) -> Result<Ops, Error> {
        enum State {
            EvalAppFun { arg: AstNode, },
            EvalAppArg { fun: EvalFunNum, },
        }

        let mut states = vec![];
        loop {
            let mut eval_op = match ast_node {
                AstNode::Literal { value, } =>
                    EvalOp::new(value),

                AstNode::App { fun, arg, } => {
                    states.push(State::EvalAppFun { arg: *arg, });
                    ast_node = *fun;
                    continue;
                },
            };

            loop {
                match (states.pop(), eval_op) {
                    (None, eval_op) =>
                        return Ok(eval_op.render()),

                    (Some(State::EvalAppFun { arg, }), EvalOp::Num { number, }) =>
                        return Err(Error::AppOnNumber { number, arg, }),
                    (Some(State::EvalAppFun { arg, }), EvalOp::Fun(EvalFun::ArgNum(fun))) => {
                        states.push(State::EvalAppArg { fun, });
                        ast_node = arg;
                        break;
                    },
                    (Some(State::EvalAppFun { arg, }), EvalOp::Fun(EvalFun::ArgFun(fun))) =>
                        unimplemented!(),
                    (Some(State::EvalAppFun { arg, }), EvalOp::Fun(EvalFun::ArgAbs(fun))) =>
                        unimplemented!(),
                    (Some(State::EvalAppFun { arg: _, }), EvalOp::Abs(..)) =>
                        unimplemented!(),

                    // inc on positive number
                    (
                        Some(State::EvalAppArg { fun: EvalFunNum::Inc0, }),
                        EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Positive(PositiveNumber { value, }),
                                modulation,
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Positive(PositiveNumber { value: value + 1, }),
                                modulation,
                            },
                        },

                    // inc on negative number
                    (
                        Some(State::EvalAppArg { fun: EvalFunNum::Inc0, }),
                        EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Negative(NegativeNumber { value, }),
                                modulation,
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Num {
                            number: EncodedNumber {
                                number: if value + 1 < 0 {
                                    Number::Negative(NegativeNumber { value: value + 1, })
                                } else {
                                    Number::Positive(PositiveNumber { value: (value + 1) as usize, })
                                },
                                modulation,
                            },
                        },

                    // inc on fun
                    (Some(State::EvalAppArg { fun: EvalFunNum::Inc0, }), EvalOp::Fun(fun)) =>
                        return Err(Error::AppExpectsNumButFunProvided { fun, }),

                    // dec on positive number
                    (
                        Some(State::EvalAppArg { fun: EvalFunNum::Dec0, }),
                        EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Positive(PositiveNumber { value, }),
                                modulation,
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Num {
                            number: EncodedNumber {
                                number: if value == 0 {
                                    Number::Negative(NegativeNumber { value: -1, })
                                } else {
                                    Number::Positive(PositiveNumber { value: value - 1, })
                                },
                                modulation,
                            },
                        },

                    // dec on negative number
                    (
                        Some(State::EvalAppArg { fun: EvalFunNum::Dec0, }),
                        EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Negative(NegativeNumber { value, }),
                                modulation,
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Negative(NegativeNumber { value: value - 1, }),
                                modulation,
                            },
                        },

                    // dec on fun
                    (Some(State::EvalAppArg { fun: EvalFunNum::Dec0, }), EvalOp::Fun(fun)) =>
                        return Err(Error::AppExpectsNumButFunProvided { fun, }),

                    // sum0 on a number
                    (
                        Some(State::EvalAppArg { fun: EvalFunNum::Sum0, }),
                        EvalOp::Num { number, },
                    ) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Sum1 {
                            captured: number,
                        })),

                    // sum0 on fun
                    (Some(State::EvalAppArg { fun: EvalFunNum::Sum0, }), EvalOp::Fun(fun)) =>
                        return Err(Error::AppExpectsNumButFunProvided { fun, }),

                    // sum1 on two numbers with different modulation
                    (
                        Some(State::EvalAppArg {
                            fun: EvalFunNum::Sum1 {
                                captured: number_a @ EncodedNumber {
                                    modulation: Modulation::Modulated,
                                    ..
                                },
                            },
                        }),
                        EvalOp::Num { number: number_b @ EncodedNumber { modulation: Modulation::Demodulated, .. }, },
                    ) |
                    (
                        Some(State::EvalAppArg {
                            fun: EvalFunNum::Sum1 {
                                captured: number_a @ EncodedNumber {
                                    modulation: Modulation::Demodulated,
                                    ..
                                },
                            },
                        }),
                        EvalOp::Num { number: number_b @ EncodedNumber { modulation: Modulation::Modulated, .. }, },
                    ) =>
                        return Err(Error::TwoNumbersOpInDifferentModulation { number_a, number_b, }),

                    // sum1 on two positive
                    (
                        Some(State::EvalAppArg {
                            fun: EvalFunNum::Sum1 {
                                captured: EncodedNumber {
                                    number: Number::Positive(PositiveNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        }),
                        EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Positive(PositiveNumber { value: value_b, }),
                                ..
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Positive(PositiveNumber { value: value_a + value_b, }),
                                modulation,
                            },
                        },

                    // sum1 on positive and negative
                    (
                        Some(State::EvalAppArg {
                            fun: EvalFunNum::Sum1 {
                                captured: EncodedNumber {
                                    number: Number::Positive(PositiveNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        }),
                        EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Negative(NegativeNumber { value: value_b, }),
                                ..
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Num {
                            number: EncodedNumber {
                                number: if (value_a as isize) + value_b < 0 {
                                    Number::Negative(NegativeNumber { value: value_a as isize + value_b, })
                                } else {
                                    Number::Positive(PositiveNumber { value: (value_a as isize + value_b) as usize, })
                                },
                                modulation,
                            },
                        },

                    // sum1 on negative and positive
                    (
                        Some(State::EvalAppArg {
                            fun: EvalFunNum::Sum1 {
                                captured: EncodedNumber {
                                    number: Number::Negative(NegativeNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        }),
                        EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Positive(PositiveNumber { value: value_b, }),
                                ..
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Num {
                            number: EncodedNumber {
                                number: if value_a + (value_b as isize) < 0 {
                                    Number::Negative(NegativeNumber { value: value_a + value_b as isize, })
                                } else {
                                    Number::Positive(PositiveNumber { value: (value_a + value_b as isize) as usize, })
                                },
                                modulation,
                            },
                        },

                    // sum1 on two negative
                    (
                        Some(State::EvalAppArg {
                            fun: EvalFunNum::Sum1 {
                                captured: EncodedNumber {
                                    number: Number::Negative(NegativeNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        }),
                        EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Negative(NegativeNumber { value: value_b, }),
                                ..
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Negative(NegativeNumber { value: value_a + value_b, }),
                                modulation,
                            },
                        },

                    // sum1 on fun
                    (Some(State::EvalAppArg { fun: EvalFunNum::Sum1 { .. }, }), EvalOp::Fun(fun)) =>
                        return Err(Error::AppExpectsNumButFunProvided { fun, }),

                    // mul0 on a number
                    (
                        Some(State::EvalAppArg { fun: EvalFunNum::Mul0, }),
                        EvalOp::Num { number, },
                    ) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Mul1 {
                            captured: number,
                        })),

                    // mul0 on fun
                    (Some(State::EvalAppArg { fun: EvalFunNum::Mul0, }), EvalOp::Fun(fun)) =>
                        return Err(Error::AppExpectsNumButFunProvided { fun, }),

                    // mul1 on two numbers with different modulation
                    (
                        Some(State::EvalAppArg {
                            fun: EvalFunNum::Mul1 {
                                captured: number_a @ EncodedNumber {
                                    modulation: Modulation::Modulated,
                                    ..
                                },
                            },
                        }),
                        EvalOp::Num { number: number_b @ EncodedNumber { modulation: Modulation::Demodulated, .. }, },
                    ) |
                    (
                        Some(State::EvalAppArg {
                            fun: EvalFunNum::Mul1 {
                                captured: number_a @ EncodedNumber {
                                    modulation: Modulation::Demodulated,
                                    ..
                                },
                            },
                        }),
                        EvalOp::Num { number: number_b @ EncodedNumber { modulation: Modulation::Modulated, .. }, },
                    ) =>
                        return Err(Error::TwoNumbersOpInDifferentModulation { number_a, number_b, }),

                    // mul1 on two positive
                    (
                        Some(State::EvalAppArg {
                            fun: EvalFunNum::Mul1 {
                                captured: EncodedNumber {
                                    number: Number::Positive(PositiveNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        }),
                        EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Positive(PositiveNumber { value: value_b, }),
                                ..
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Positive(PositiveNumber { value: value_a * value_b, }),
                                modulation,
                            },
                        },

                    // mul1 on positive and negative
                    (
                        Some(State::EvalAppArg {
                            fun: EvalFunNum::Mul1 {
                                captured: EncodedNumber {
                                    number: Number::Positive(PositiveNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        }),
                        EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Negative(NegativeNumber { value: value_b, }),
                                ..
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Negative(NegativeNumber { value: value_a as isize * value_b, }),
                                modulation,
                            },
                        },

                    // mul1 on negative and positive
                    (
                        Some(State::EvalAppArg {
                            fun: EvalFunNum::Mul1 {
                                captured: EncodedNumber {
                                    number: Number::Negative(NegativeNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        }),
                        EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Positive(PositiveNumber { value: value_b, }),
                                ..
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Negative(NegativeNumber { value: value_a * value_b as isize, }),
                                modulation,
                            },
                        },

                    // mul1 on two negative
                    (
                        Some(State::EvalAppArg {
                            fun: EvalFunNum::Mul1 {
                                captured: EncodedNumber {
                                    number: Number::Negative(NegativeNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        }),
                        EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Negative(NegativeNumber { value: value_b, }),
                                ..
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Positive(PositiveNumber { value: (value_a * value_b) as usize, }),
                                modulation,
                            },
                        },

                    // mul1 on fun
                    (Some(State::EvalAppArg { fun: EvalFunNum::Mul1 { .. }, }), EvalOp::Fun(fun)) =>
                        return Err(Error::AppExpectsNumButFunProvided { fun, }),

                    // div0 on a number
                    (
                        Some(State::EvalAppArg { fun: EvalFunNum::Div0, }),
                        EvalOp::Num { number, },
                    ) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Div1 {
                            captured: number,
                        })),

                    // div0 on fun
                    (Some(State::EvalAppArg { fun: EvalFunNum::Div0, }), EvalOp::Fun(fun)) =>
                        return Err(Error::AppExpectsNumButFunProvided { fun, }),

                    // div1 on a zero
                    (
                        Some(State::EvalAppArg { fun: EvalFunNum::Div1 { .. }, }),
                        EvalOp::Num { number: EncodedNumber { number: Number::Positive(PositiveNumber { value: 0, }), .. }, },
                    ) =>
                        return Err(Error::DivisionByZero),

                    // div1 on two numbers with different modulation
                    (
                        Some(State::EvalAppArg {
                            fun: EvalFunNum::Div1 {
                                captured: number_a @ EncodedNumber {
                                    modulation: Modulation::Modulated,
                                    ..
                                },
                            },
                        }),
                        EvalOp::Num { number: number_b @ EncodedNumber { modulation: Modulation::Demodulated, .. }, },
                    ) |
                    (
                        Some(State::EvalAppArg {
                            fun: EvalFunNum::Div1 {
                                captured: number_a @ EncodedNumber {
                                    modulation: Modulation::Demodulated,
                                    ..
                                },
                            },
                        }),
                        EvalOp::Num { number: number_b @ EncodedNumber { modulation: Modulation::Modulated, .. }, },
                    ) =>
                        return Err(Error::TwoNumbersOpInDifferentModulation { number_a, number_b, }),

                    // div1 on two positive
                    (
                        Some(State::EvalAppArg {
                            fun: EvalFunNum::Div1 {
                                captured: EncodedNumber {
                                    number: Number::Positive(PositiveNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        }),
                        EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Positive(PositiveNumber { value: value_b, }),
                                ..
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Positive(PositiveNumber { value: value_a / value_b, }),
                                modulation,
                            },
                        },

                    // div1 on positive and negative
                    (
                        Some(State::EvalAppArg {
                            fun: EvalFunNum::Div1 {
                                captured: EncodedNumber {
                                    number: Number::Positive(PositiveNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        }),
                        EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Negative(NegativeNumber { value: value_b, }),
                                ..
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Negative(NegativeNumber { value: value_a as isize / value_b, }),
                                modulation,
                            },
                        },

                    // div1 on negative and positive
                    (
                        Some(State::EvalAppArg {
                            fun: EvalFunNum::Div1 {
                                captured: EncodedNumber {
                                    number: Number::Negative(NegativeNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        }),
                        EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Positive(PositiveNumber { value: value_b, }),
                                ..
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Negative(NegativeNumber { value: value_a / value_b as isize, }),
                                modulation,
                            },
                        },

                    // div1 on two negative
                    (
                        Some(State::EvalAppArg {
                            fun: EvalFunNum::Div1 {
                                captured: EncodedNumber {
                                    number: Number::Negative(NegativeNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        }),
                        EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Negative(NegativeNumber { value: value_b, }),
                                ..
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Positive(PositiveNumber { value: (value_a / value_b) as usize, }),
                                modulation,
                            },
                        },

                    // div1 on fun
                    (Some(State::EvalAppArg { fun: EvalFunNum::Div1 { .. }, }), EvalOp::Fun(fun)) =>
                        return Err(Error::AppExpectsNumButFunProvided { fun, }),

                    // fun on abs
                    (Some(State::EvalAppArg { fun }), EvalOp::Abs(arg_ast_node)) => {
                        let mut fun_ops_iter = EvalOp::Fun(EvalFun::ArgNum(fun))
                            .render()
                            .0
                            .into_iter();
                        let ast_node = match fun_ops_iter.next() {
                            None =>
                                unreachable!(),
                            Some(op_a) =>
                                match fun_ops_iter.next() {
                                    None =>
                                        AstNode::App {
                                            fun: Box::new(AstNode::Literal { value: op_a, }),
                                            arg: Box::new(arg_ast_node),
                                        },
                                    Some(op_b) =>
                                        match fun_ops_iter.next() {
                                            None =>
                                                AstNode::App {
                                                    fun: Box::new(AstNode::App {
                                                        fun: Box::new(AstNode::Literal { value: op_a, }),
                                                        arg: Box::new(AstNode::Literal { value: op_b, }),
                                                    }),
                                                    arg: Box::new(arg_ast_node),
                                                },
                                            Some(..) =>
                                                unreachable!(),
                                        },
                                },
                        };
                        eval_op = EvalOp::Abs(ast_node);
                    },
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum EvalOp {
    Num { number: EncodedNumber, },
    Fun(EvalFun),
    Abs(AstNode),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EvalFun {
    ArgNum(EvalFunNum),
    ArgFun(EvalFunFun),
    ArgAbs(EvalFunAbs),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EvalFunNum {
    Inc0,
    Dec0,
    Sum0,
    Sum1 { captured: EncodedNumber, },
    Mul0,
    Mul1 { captured: EncodedNumber, },
    Div0,
    Div1 { captured: EncodedNumber, },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EvalFunFun {
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EvalFunAbs {
    True0,
    True1 { captured: AstNode, },
}

impl EvalOp {
    fn new(op: Op) -> EvalOp {
        match op {
            Op::Const(Const::EncodedNumber(number)) =>
                EvalOp::Num { number, },
            Op::Const(Const::Fun(Fun::Inc)) =>
                EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Inc0)),
            Op::Const(Const::Fun(Fun::Dec)) =>
                EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Dec0)),
            Op::Const(Const::Fun(Fun::Sum)) =>
                EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Sum0)),
            Op::Const(Const::Fun(Fun::Mul)) =>
                EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Mul0)),
            Op::Const(Const::Fun(Fun::Div)) =>
                EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Div0)),
            Op::Const(Const::Fun(Fun::Eq)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Lt)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Mod)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Dem)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Send)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Neg)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::S)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::C)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::B)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::True)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::True0)),
            Op::Const(Const::Fun(Fun::False)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Pwr2)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::I)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Cons)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Car)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Cdr)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Nil)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::IsNil)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::LeftParen)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Comma)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::RightParen)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Vec)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Draw)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Chkb)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::MultipleDraw)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::If0)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Interact)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Modem)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::Galaxy)) =>
                unimplemented!(),
            Op::Variable(var) =>
                EvalOp::Abs(AstNode::Literal { value: Op::Variable(var), }),
            Op::App =>
                unreachable!(),
        }
    }

    fn render(self) -> Ops {
        match self {
            EvalOp::Num { number, } =>
                Ops(vec![Op::Const(Const::EncodedNumber(number))]),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Inc0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Inc))]),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Dec0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Dec))]),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Sum0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Sum))]),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Sum1 { captured, })) =>
                Ops(vec![
                    Op::Const(Const::Fun(Fun::Sum)),
                    Op::Const(Const::EncodedNumber(captured)),
                ]),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Mul0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Mul))]),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Mul1 { captured, })) =>
                Ops(vec![
                    Op::Const(Const::Fun(Fun::Mul)),
                    Op::Const(Const::EncodedNumber(captured)),
                ]),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Div0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Div))]),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Div1 { captured, })) =>
                Ops(vec![
                    Op::Const(Const::Fun(Fun::Div)),
                    Op::Const(Const::EncodedNumber(captured)),
                ]),
            EvalOp::Fun(EvalFun::ArgFun(..)) =>
                unimplemented!(),
            EvalOp::Fun(EvalFun::ArgAbs(..)) =>
                unimplemented!(),
            EvalOp::Abs(ast_node) =>
                ast_node.render(),
        }
    }
}

impl AstNode {
    pub fn render(self) -> Ops {
        enum State {
            RenderAppFun { arg: AstNode, },
            RenderAppArg,
        }

        let mut ops = vec![];
        let mut ast_node = self;
        let mut stack = vec![];
        loop {
            match ast_node {
                AstNode::Literal { value, } =>
                    ops.push(value),
                AstNode::App { fun, arg, } => {
                    ops.push(Op::App);
                    stack.push(State::RenderAppFun { arg: *arg, });
                    ast_node = *fun;
                    continue;
                },
            }

            loop {
                match stack.pop() {
                    None =>
                        return Ops(ops),
                    Some(State::RenderAppFun { arg, }) => {
                        stack.push(State::RenderAppArg);
                        ast_node = arg;
                        break;
                    },
                    Some(State::RenderAppArg) =>
                        (),
                }
            }
        }
    }
}
