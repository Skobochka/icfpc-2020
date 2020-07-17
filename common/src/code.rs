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
    Const(Const), // constants
    Variable(Variable), // variables
    App, // function application
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Const {
    Literal(Literal),
    Fun(Fun), // predefined functions from spec
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
    Inc, // successor
    Dec, // predecessor
    Sum, // sum
    Mul, // product
    Div, // integer division
    Eq, // equality
    Lt, // strict less-than
    Mod, // modulate / modulate list
    Dem, // demodulate
    Send, // send
    Neg, // negate
    S, // S combinator
    C, // C combinator
    B, // B combinator
    True, // true (K combinator)
    False, // false (combinator)
    Pwr2, // power of two
    I, // I combinator
    Cons, // cons / pair
    Car, // car / first
    Cdr, // cdr / tail
    Nil, // nil / empty list
    IsNil, // is nil (is empty list)
    LeftParen, // left parenthesis (list construction syntax)
    Comma, // comma (list construction syntax)
    RightParen, // right parenthesis (list construction syntax)
    Vec, // vector (alias for cons)
    Draw, // draw (communication with display)
    Chkb, // checkerboard
    MultipleDraw, // takes a list of lists of 2D-points and returns a list of plot canvases
    If0, // compare 1st argument to 0 and pick 2nd, else 3rd
    Interact, // interact
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
