use std::sync::Arc;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Script {
    pub statements: Vec<Statement>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Statement {
    Single(Op),
    EqBind(EqBind),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Op {
    Const(Const),
    Variable(Variable),
    App(App),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Const {
    Literal(Literal),
    Fun(Fun),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Literal {
    Positive(PositiveLiteral),
    Negative(NegativeLiteral),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PositiveLiteral {
    pub value: usize,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct NegativeLiteral {
    pub value: isize,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Fun {
    Inc,
    Dec,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Variable {
    pub name: Literal,
    pub bind: Binding,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Binding {
    Unbound,
    Bound(Arc<Op>),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct App {
    fun: Arc<Op>,
    arg: Arc<Op>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct EqBind {
    pub left: Arc<Op>,
    pub right: Arc<Op>,
}

impl EqBind {
    pub fn new(left_op: Op, right_op: Op) -> EqBind {
        EqBind {
            left: Arc::new(left_op),
            right: Arc::new(right_op),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_00() {
        let _script = Script {
            statements: vec![
                Statement::Single(Op::Const(Const::Literal(Literal::Positive(PositiveLiteral {
                    value: 1,
                })))),
                Statement::EqBind(EqBind::new(
                    Op::Const(Const::Literal(Literal::Positive(PositiveLiteral {
                        value: 1,
                    }))),
                    Op::Const(Const::Literal(Literal::Positive(PositiveLiteral {
                        value: 1,
                    }))),
                )),
                Statement::EqBind(EqBind::new(
                    Op::Const(Const::Literal(Literal::Positive(PositiveLiteral {
                        value: 1,
                    }))),
                    Op::Variable(Variable {
                        name: Literal::Positive(PositiveLiteral {
                            value: 0,
                        }),
                        bind: Binding::Unbound,
                    }),
                )),
            ],
        };
    }
}
