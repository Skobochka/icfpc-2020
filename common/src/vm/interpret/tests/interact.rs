use std::rc::Rc;

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
    super::AstNode,
};

#[test]
fn render_interact() {
    let interpreter = Interpreter::new();

    assert_eq!(
        interpreter.eval_interact(
            Rc::new(AstNode::Literal { value: Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 777, }), }), }),
            Rc::new(AstNode::Literal { value: Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 111, }), }), }),
            Rc::new(AstNode::Literal { value: Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 333, }), }), }),
            &Env::new(),
        ).unwrap().render(),
        Ops(vec![
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::F38)),
            Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 777, }), }),
            Op::App,
            Op::App,
            Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 777, }), }),
            Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 111, }), }),
            Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 333, }), }),
        ]),
    );
}

#[test]
fn render_f38() {
    let interpreter = Interpreter::new();

    assert_eq!(
        interpreter.eval_f38(
            Rc::new(AstNode::Literal { value: Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 777, }), }), }),
            Rc::new(AstNode::Literal { value: Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 111, }), }), }),
            &Env::new(),
        ).unwrap().render(),
        Ops(vec![
            Op::App,
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::If0)),
            Op::App,
            Op::Const(Const::Fun(Fun::Car)),
            Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 111, }), }),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::App,
            Op::Const(Const::Fun(Fun::Modem)),
            Op::App,
            Op::Const(Const::Fun(Fun::Car)),
            Op::App,
            Op::Const(Const::Fun(Fun::Cdr)),
            Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 111, }), }),
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Cons)),
            Op::App,
            Op::Const(Const::Fun(Fun::MultipleDraw)),
            Op::App,
            Op::Const(Const::Fun(Fun::Car)),
            Op::App,
            Op::Const(Const::Fun(Fun::Cdr)),
            Op::App,
            Op::Const(Const::Fun(Fun::Cdr)),
            Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 111, }), }),
            Op::Const(Const::Fun(Fun::Nil)),
            Op::App,
            Op::App,
            Op::App,
            Op::Const(Const::Fun(Fun::Interact)),
            Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 777, }), }),
            Op::App,
            Op::Const(Const::Fun(Fun::Modem)),
            Op::App,
            Op::Const(Const::Fun(Fun::Car)),
            Op::App,
            Op::Const(Const::Fun(Fun::Cdr)),
            Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 111, }), }),
            Op::App,
            Op::Const(Const::Fun(Fun::Send)),
            Op::App,
            Op::Const(Const::Fun(Fun::Car)),
            Op::App,
            Op::Const(Const::Fun(Fun::Cdr)),
            Op::App,
            Op::Const(Const::Fun(Fun::Cdr)),
            Op::Variable(Variable { name: Number::Positive(PositiveNumber { value: 111, }), }),
        ]),
    );
}
