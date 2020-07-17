#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Script {
    pub statements: Vec<Statement>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Statement {
    Single(Ops),
    EqBind(EqBind),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Ops(pub Vec<Op>);

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Op {
    Const(Const),
    Variable(Variable),
    App,
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
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct EqBind {
    pub left: Ops,
    pub right: Ops,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_00() {
        let _script = Script {
            statements: vec![
                // 1
                Statement::Single(Ops(vec![Op::Const(Const::Literal(Literal::Positive(PositiveLiteral {
                    value: 1,
                })))])),

                // -1
                Statement::Single(Ops(vec![Op::Const(Const::Literal(Literal::Negative(NegativeLiteral {
                    value: -1,
                })))])),

                // 1 = 1
                Statement::EqBind(EqBind {
                    left: Ops(vec![Op::Const(Const::Literal(Literal::Positive(PositiveLiteral {
                        value: 1,
                    })))]),
                    right: Ops(vec![Op::Const(Const::Literal(Literal::Positive(PositiveLiteral {
                        value: 1,
                    })))]),
                }),

                // 1 = x0
                Statement::EqBind(EqBind {
                    left: Ops(vec![Op::Const(Const::Literal(Literal::Positive(PositiveLiteral {
                        value: 1,
                    })))]),
                    right: Ops(vec![Op::Variable(Variable {
                        name: Literal::Positive(PositiveLiteral {
                            value: 0,
                        }),
                    })]),
                }),
            ],
        };
    }
}
