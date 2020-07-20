use std::{
    rc::Rc,
    sync::mpsc,
    collections::HashMap,
};

use futures::{
    channel::mpsc::UnboundedSender,
};

use super::{
    super::encoder::{
        self,
        Modulable,
    },
    super::code::{
        Op,
        Ops,
        Fun,
        Const,
        Coord,
        Number,
        Syntax,
        Picture,
        Variable,
        Modulation,
        EncodedNumber,
        PositiveNumber,
        NegativeNumber,
        Equality,
        Script,
        Statement,
    },
};

#[cfg(test)]
mod tests;

pub struct Interpreter {
    outer_channel: Option<UnboundedSender<OuterRequest>>,
}

pub enum OuterRequest {
    ProxySend {
        modulated_req: String,
        modulated_rep: mpsc::Sender<String>,
    },
    RenderPictures {
        pictures: Vec<Picture>,
    },
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Error {
    NoAppFunProvided,
    NoAppArgProvided { fun: Ops, },
    EvalEmptyTree,
    AppOnNumber { number: EncodedNumber, arg: Ops, },
    AppExpectsNumButFunProvided { fun: Ops, },
    TwoNumbersOpInDifferentModulation { number_a: EncodedNumber, number_b: EncodedNumber, },
    DivisionByZero,
    IsNilAppOnANumber { number: EncodedNumber, },
    ModOnModulatedNumber { number: EncodedNumber, },
    DemOnDemodulatedNumber { number: EncodedNumber, },
    ListNotClosed,
    ListCommaWithoutElement,
    ListSyntaxUnexpectedNode { node: Ops, },
    ListSyntaxSeveralCommas,
    ListSyntaxClosingAfterComma,
    InvalidCoordForDrawArg,
    ExpectedListArgForSendButGotNumber { number: EncodedNumber, },
    ConsListDem(encoder::Error),
    SendOpIsNotSupportedWithoutOuterChannel,
    RenderOpIsNotSupportedWithoutOuterChannel,
    OuterChannelIsClosed,
    DemodulatedNumberInList { number: EncodedNumber, },
    RenderItemIsNotAPicture { ops: Ops, },
    InvalidConsListItem { ops: Ops, },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ast {
    Empty,
    Tree(AstNode),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum AstNode {
    Literal { value: Op, },
    App { fun: Rc<AstNode>, arg: Rc<AstNode>, },
}

#[derive(Debug)]
pub struct Env {
    forward: HashMap<AstNode, AstNode>,
    backward: HashMap<AstNode, AstNode>,
}

impl Env {
    pub fn new() -> Env {
        Env {
            forward: HashMap::new(),
            backward: HashMap::new(),
        }
    }

    pub fn add_equality(&mut self, left: Ast, right: Ast) {
        if let (Ast::Tree(left), Ast::Tree(right)) = (left, right) {
            if let AstNode::Literal { value: Op::Variable(..), } = left {
                self.forward.insert(left.clone(), right.clone());
            }
            if let AstNode::Literal { value: Op::Variable(..), } = right {
                self.backward.insert(right, left);
            }
        }
    }

    pub fn lookup_ast(&self, key: &AstNode) -> Option<&AstNode> {
        match self.forward.get(key) {
            Some(o) => {
                Some(o)
            },
            None => match self.backward.get(key) {
                Some(o) => {
                    Some(o)
                },
                None =>
                    None,
            }
        }
    }

    pub fn clear(&mut self) {
        self.forward.clear();
        self.backward.clear();
    }
}

#[derive(Debug)]
pub struct Cache {
    memo: HashMap<Rc<AstNode>, Rc<AstNode>>,
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            memo: HashMap::new(),
        }
    }

    pub fn get(&self, key: &Rc<AstNode>) -> Option<Rc<AstNode>> {
        if let Some(ast_node) = self.memo.get(key) {
            Some(ast_node.clone())
        } else {
            None
        }
    }

    pub fn memo(&mut self, key: Rc<AstNode>, value: Rc<AstNode>) {
        self.memo.insert(key, value);
    }
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            outer_channel: None,
        }
    }

    pub fn with_outer_channel(outer_channel: UnboundedSender<OuterRequest>) -> Interpreter {
        Interpreter {
            outer_channel: Some(outer_channel),
        }
    }

    pub fn make_prev_variable_ast(&self) -> Ast {
        Ast::Tree(AstNode::Literal { value: Op::Variable(Variable { name: Number::Negative(NegativeNumber { value: -2, }), }), })
    }

    pub fn build_tree(&self, Ops(mut ops): Ops) -> Result<Ast, Error> {
        enum State {
            AwaitAppFun,
            AwaitAppArg { fun: Rc<AstNode>, },
            ListBegin,
            ListPush { element: Rc<AstNode>, },
            ListContinue,
            ListContinueComma,
        }

        let mut states = vec![];
        ops.reverse();
        loop {
            let mut maybe_node: Option<AstNode> = match ops.pop() {
                None =>
                    None,
                Some(Op::Const(Const::Fun(Fun::Galaxy))) =>
                    Some(AstNode::Literal {
                        value: Op::Variable(Variable {
                            name: Number::Negative(NegativeNumber {
                                value: -1,
                            }),
                        }),
                    }),
                Some(Op::Syntax(Syntax::LeftParen)) => {
                    states.push(State::ListBegin);
                    continue;
                },
                Some(value @ Op::Const(..)) |
                Some(value @ Op::Variable(..)) |
                Some(value @ Op::Syntax(..)) =>
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
                        states.push(State::AwaitAppArg { fun: Rc::new(node), });
                        break;
                    },
                    (Some(State::AwaitAppArg { fun, }), None) =>
                        return Err(Error::NoAppArgProvided { fun: fun.render(), }),
                    (Some(State::AwaitAppArg { fun, }), Some(node)) => {
                        maybe_node = Some(AstNode::App {
                            fun: fun,
                            arg: Rc::new(node),
                        });
                    },
                    (Some(State::ListBegin), None) =>
                        return Err(Error::ListNotClosed),
                    (Some(State::ListBegin), Some(AstNode::Literal { value: Op::Syntax(Syntax::Comma), })) =>
                        return Err(Error::ListCommaWithoutElement),
                    (Some(State::ListBegin), Some(AstNode::Literal { value: Op::Syntax(Syntax::RightParen), })) =>
                        maybe_node = Some(AstNode::Literal {
                            value: Op::Const(Const::Fun(Fun::Nil)),
                        }),
                    (Some(State::ListBegin), Some(node)) => {
                        states.push(State::ListPush { element: Rc::new(node), });
                        states.push(State::ListContinue);
                        break;
                    },
                    (Some(State::ListContinue), None) =>
                        return Err(Error::ListNotClosed),
                    (Some(State::ListContinue), Some(AstNode::Literal { value: Op::Syntax(Syntax::Comma), })) => {
                        states.push(State::ListContinueComma);
                        break;
                    },
                    (Some(State::ListContinue), Some(AstNode::Literal { value: Op::Syntax(Syntax::RightParen), })) =>
                        maybe_node = Some(AstNode::Literal {
                            value: Op::Const(Const::Fun(Fun::Nil)),
                        }),
                    (Some(State::ListContinue), Some(node)) =>
                        return Err(Error::ListSyntaxUnexpectedNode { node: Rc::new(node).render(), }),
                    (Some(State::ListContinueComma), None) =>
                        return Err(Error::ListNotClosed),
                    (Some(State::ListContinueComma), Some(AstNode::Literal { value: Op::Syntax(Syntax::Comma), })) =>
                        return Err(Error::ListSyntaxSeveralCommas),
                    (Some(State::ListContinueComma), Some(AstNode::Literal { value: Op::Syntax(Syntax::RightParen), })) =>
                        return Err(Error::ListSyntaxClosingAfterComma),
                    (Some(State::ListContinueComma), Some(node)) => {
                        states.push(State::ListPush { element: Rc::new(node), });
                        states.push(State::ListContinue);
                        break;
                    },
                    (Some(State::ListPush { .. }), None) =>
                        unreachable!(),
                    (Some(State::ListPush { element, }), Some(tail)) =>
                        maybe_node = Some(AstNode::App {
                            fun: Rc::new(AstNode::App {
                                fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cons)), }),
                                arg: element,
                            }),
                            arg: Rc::new(tail),
                        }),
                }
            }
        }
    }

    pub fn eval_script(&self, script: Script) -> Result<Env, Error> {
        let mut env = Env::new();

        for Statement::Equality(eq) in script.statements {
            let _next_eq = self.eval_equality(eq, &mut env)?;
        }

        Ok(env)
    }

    fn eval_equality(&self, Equality { left, right }: Equality, env: &mut Env) -> Result<(), Error> {
        env.add_equality(
            self.build_tree(left)?,
            self.build_tree(right)?,
        );

        Ok(())
    }

    pub fn lookup_env(&self, env: &Env, key: Ops) -> Result<Option<Ops>, Error> {
        if let Ast::Tree(ast_node) = self.build_tree(key)? {
            if let Some(ast_node) = env.lookup_ast(&ast_node) {
                let ast_node = Rc::new(ast_node.clone());
                return Ok(Some(ast_node.render()))
            }
        }
        Ok(None)
    }

    pub fn eval(&self, ast: Ast, env: &Env) -> Result<Ops, Error> {
        let mut cache = Cache::new();
        self.eval_cache(ast, env, &mut cache)
    }

    pub fn eval_cache(&self, ast: Ast, env: &Env, cache: &mut Cache) -> Result<Ops, Error> {
        match ast {
            Ast::Empty =>
                Err(Error::EvalEmptyTree),
            Ast::Tree(node) =>
                self.eval_tree(node, env, cache),
        }
    }

    fn eval_tree(&self, ast_node: AstNode, env: &Env, cache: &mut Cache) -> Result<Ops, Error> {
        let mut ast_node = Rc::new(ast_node);

        enum State {
            EvalAppFun { arg: Rc<AstNode>, },
            EvalAppArgNum { fun: EvalFunNum, },
            EvalAppArgIsNil,
        }

        struct StackFrame {
            root: Rc<AstNode>,
            state: State,
        }

        let mut states = vec![];
        loop {
            if let Some(memo_ast) = cache.get(&ast_node) {
                ast_node = memo_ast;
                continue;
            }

            let mut eval_op = match &*ast_node {
                AstNode::Literal { value, } =>
                    EvalOp::new(value.clone()),

                AstNode::App { fun, arg, } => {
                    states.push(StackFrame { root: ast_node.clone(), state: State::EvalAppFun { arg: arg.clone(), }, });
                    ast_node = fun.clone();
                    continue;
                },
            };

            loop {
                let frame = match states.pop() {
                    None =>
                        match eval_op {
                            EvalOp::Abs(top_ast_node) => {
                                match env.lookup_ast(&top_ast_node) {
                                    Some(subst_ast_node) => {
                                        ast_node = Rc::new(subst_ast_node.clone());
                                        break;
                                    },
                                    None =>
                                        return Ok(EvalOp::Abs(top_ast_node).render()),
                                }
                            },

                            eval_op =>
                                return Ok(eval_op.render()),
                        },
                    Some(frame) =>
                        frame,
                };

                let root = frame.root;
                match (frame.state, eval_op) {
                    (State::EvalAppFun { arg, .. }, EvalOp::Num { number, }) =>
                        return Err(Error::AppOnNumber { number, arg: arg.render(), }),

                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgNum(fun))) => {
                        states.push(StackFrame { root, state: State::EvalAppArgNum { fun, }, });
                        ast_node = arg;
                        break;
                    },

                    // true0 on a something
                    (State::EvalAppFun { arg, .. }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::True0))) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::True1 {
                            captured: arg,
                        })),

                    // true1 on a something: ap ap t x0 x1 = x0
                    (State::EvalAppFun { .. }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::True1 { captured, }))) => {
                        ast_node = captured;
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // false0 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::False0))) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::False1 {
                            captured: arg,
                        })),

                    // false1 on a something: ap ap t x0 x1 = x1
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::False1 { .. }))) => {
                        ast_node = arg;
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // I0 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::I0))) => {
                        ast_node = arg;
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // C0 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::C0))) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::C1 {
                            x: arg,
                        })),

                    // C1 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::C1 { x, }))) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::C2 {
                            x, y: arg,
                        })),

                    // C2 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::C2 { x, y, }))) => {
                        ast_node = Rc::new(AstNode::App {
                            fun: Rc::new(AstNode::App {
                                fun: x,
                                arg: arg,
                            }),
                            arg: y,
                        });
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // B0 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::B0))) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::B1 {
                            x: arg,
                        })),

                    // B1 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::B1 { x, }))) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::B2 {
                            x, y: arg,
                        })),

                    // B2 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::B2 { x, y, }))) => {
                        ast_node = Rc::new(AstNode::App {
                            fun: x,
                            arg: Rc::new(AstNode::App {
                                fun: y,
                                arg: arg,
                            }),
                        });
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // S0 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::S0))) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::S1 {
                            x: arg,
                        })),

                    // S1 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::S1 { x, }))) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::S2 {
                            x, y: arg,
                        })),

                    // S2 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::S2 { x, y, }))) => {
                        ast_node = Rc::new(AstNode::App {
                            fun: Rc::new(AstNode::App {
                                fun: x,
                                arg: arg.clone(),
                            }),
                            arg: Rc::new(AstNode::App {
                                fun: y,
                                arg: arg,
                            }),
                        });
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // Cons0 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Cons0))) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Cons1 {
                            x: arg,
                        })),

                    // Cons1 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Cons1 { x, }))) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Cons2 {
                            x, y: arg,
                        })),

                    // Cons2 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Cons2 { x, y, }))) => {
                        ast_node = Rc::new(AstNode::App {
                            fun: Rc::new(AstNode::App {
                                fun: arg,
                                arg: x,
                            }),
                            arg: y,
                        });
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // Car0 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Car0))) => {
                        ast_node = Rc::new(AstNode::App {
                            fun: arg,
                            arg: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::True)), }),
                        });
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // Cdr0 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Cdr0))) => {
                        ast_node = Rc::new(AstNode::App {
                            fun: arg,
                            arg: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::False)), }),
                        });
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // Nil0 on a something
                    (State::EvalAppFun { .. }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Nil0))) => {
                        ast_node = Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::True)), });
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // IsNil0 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::IsNil0))) => {
                        states.push(StackFrame { root, state: State::EvalAppArgIsNil, });
                        ast_node = arg;
                        break;
                    },

                    // IsNil on a number
                    (State::EvalAppArgIsNil, EvalOp::Num { number, }) =>
                        return Err(Error::IsNilAppOnANumber { number, }),

                    // IsNil on a Nil0
                    (State::EvalAppArgIsNil, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Nil0))) => {
                        ast_node = Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::True)), });
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // IsNil on another fun
                    (State::EvalAppArgIsNil, EvalOp::Fun(..)) => {
                        ast_node = Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::False)), });
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // IsNil on an abstract
                    (State::EvalAppArgIsNil, EvalOp::Abs(arg_ast_node)) =>
                        match env.lookup_ast(&arg_ast_node) {
                            Some(subst_ast_node) => {
                                states.push(StackFrame { root, state: State::EvalAppArgIsNil, });
                                ast_node = Rc::new(subst_ast_node.clone());
                                break;
                            },
                            None =>
                                eval_op = EvalOp::Abs(Rc::new(AstNode::App {
                                    fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::IsNil)), }),
                                    arg: arg_ast_node,
                                })),
                        },

                    // IfZero1 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::IfZero1 { cond, }))) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::IfZero2 {
                            cond, true_clause: arg,
                        })),

                    // IfZero2 on a something
                    (State::EvalAppFun { arg: false_clause, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::IfZero2 { cond, true_clause, }))) => {
                        ast_node = match cond {
                            EncodedNumber { number: Number::Positive(PositiveNumber { value: 0, }), .. } =>
                                true_clause,
                            EncodedNumber { .. } =>
                                false_clause,
                        };
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // Draw0 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Draw0))) => {
                        ast_node = Rc::new(AstNode::Literal {
                            value: Op::Const(Const::Picture(self.eval_draw(arg, env, cache)?)),
                        });
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // MultipleDraw0 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::MultipleDraw0))) => {
                        ast_node = self.eval_multiple_draw(arg, env, cache)?;
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // Send0 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Send0))) => {
                        ast_node = self.eval_send(arg, env, cache)?;
                        break;
                    },

                    // Render0 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Render0))) => {
                        ast_node = self.eval_render(arg, env, cache)?;
                        break;
                    },

                    // Mod0 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Mod0))) => {
                        ast_node = self.eval_mod(arg, env, cache)?;
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // Dem0 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Dem0))) => {
                        ast_node = self.eval_dem(arg, env, cache)?;
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // Modem0 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Modem0))) => {
                        ast_node = self.eval_modem(arg, env, cache)?;
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // Interact0 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Interact0))) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Interact1 {
                            protocol: arg,
                        })),

                    // Interact1 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Interact1 { protocol, }))) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Interact2 {
                            protocol, state: arg,
                        })),

                    // Interact2 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Interact2 { protocol, state, }))) => {
                        let vector = arg;
                        ast_node = self.eval_interact(protocol, state, vector, env)?;
                        break;
                    },

                    // F38_0 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::F38_0))) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::F38_1 {
                            protocol: arg,
                        })),

                    // F38_1 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::F38_1 { protocol, }))) => {
                        ast_node = self.eval_f38(protocol, arg, env)?;
                        break;
                    }

                    // unresolved fun on something
                    (State::EvalAppFun { arg: arg_ast_node, }, EvalOp::Abs(fun_ast_node)) =>
                        match env.lookup_ast(&fun_ast_node) {
                            Some(subst_ast_node) => {
                                ast_node = Rc::new(AstNode::App {
                                    fun: Rc::new(subst_ast_node.clone()),
                                    arg: arg_ast_node,
                                });
                                break;
                            }
                            None =>
                                eval_op = EvalOp::Abs(Rc::new(AstNode::App {
                                    fun: fun_ast_node,
                                    arg: arg_ast_node,
                                })),
                        },

                    // if0 on a number
                    (State::EvalAppArgNum { fun: EvalFunNum::IfZero0, }, EvalOp::Num { number, }) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::IfZero1 {
                            cond: number,
                        })),

                    // inc on positive number
                    (
                        State::EvalAppArgNum { fun: EvalFunNum::Inc0, },
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
                        State::EvalAppArgNum { fun: EvalFunNum::Inc0, },
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

                    // dec on positive number
                    (
                        State::EvalAppArgNum { fun: EvalFunNum::Dec0, },
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
                        State::EvalAppArgNum { fun: EvalFunNum::Dec0, },
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

                    // sum0 on a number
                    (
                        State::EvalAppArgNum { fun: EvalFunNum::Sum0, },
                        EvalOp::Num { number, },
                    ) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Sum1 {
                            captured: number,
                        })),

                    // sum1 on two numbers with different modulation
                    (
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Sum1 {
                                captured: number_a @ EncodedNumber {
                                    modulation: Modulation::Modulated,
                                    ..
                                },
                            },
                        },
                        EvalOp::Num { number: number_b @ EncodedNumber { modulation: Modulation::Demodulated, .. }, },
                    ) |
                    (
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Sum1 {
                                captured: number_a @ EncodedNumber {
                                    modulation: Modulation::Demodulated,
                                    ..
                                },
                            },
                        },
                        EvalOp::Num { number: number_b @ EncodedNumber { modulation: Modulation::Modulated, .. }, },
                    ) =>
                        return Err(Error::TwoNumbersOpInDifferentModulation { number_a, number_b, }),

                    // sum1 on two positive
                    (
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Sum1 {
                                captured: EncodedNumber {
                                    number: Number::Positive(PositiveNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        },
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
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Sum1 {
                                captured: EncodedNumber {
                                    number: Number::Positive(PositiveNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        },
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
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Sum1 {
                                captured: EncodedNumber {
                                    number: Number::Negative(NegativeNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        },
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
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Sum1 {
                                captured: EncodedNumber {
                                    number: Number::Negative(NegativeNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        },
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

                    // mul0 on a number
                    (
                        State::EvalAppArgNum { fun: EvalFunNum::Mul0, },
                        EvalOp::Num { number, },
                    ) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Mul1 {
                            captured: number,
                        })),

                    // mul1 on two numbers with different modulation
                    (
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Mul1 {
                                captured: number_a @ EncodedNumber {
                                    modulation: Modulation::Modulated,
                                    ..
                                },
                            },
                        },
                        EvalOp::Num { number: number_b @ EncodedNumber { modulation: Modulation::Demodulated, .. }, },
                    ) |
                    (
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Mul1 {
                                captured: number_a @ EncodedNumber {
                                    modulation: Modulation::Demodulated,
                                    ..
                                },
                            },
                        },
                        EvalOp::Num { number: number_b @ EncodedNumber { modulation: Modulation::Modulated, .. }, },
                    ) =>
                        return Err(Error::TwoNumbersOpInDifferentModulation { number_a, number_b, }),

                    // mul1 on two positive
                    (
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Mul1 {
                                captured: EncodedNumber {
                                    number: Number::Positive(PositiveNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        },
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
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Mul1 {
                                captured: EncodedNumber {
                                    number: Number::Positive(PositiveNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        },
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
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Mul1 {
                                captured: EncodedNumber {
                                    number: Number::Negative(NegativeNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        },
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
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Mul1 {
                                captured: EncodedNumber {
                                    number: Number::Negative(NegativeNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        },
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

                    // div0 on a number
                    (
                        State::EvalAppArgNum { fun: EvalFunNum::Div0, },
                        EvalOp::Num { number, },
                    ) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Div1 {
                            captured: number,
                        })),

                    // div1 on a zero
                    (
                        State::EvalAppArgNum { fun: EvalFunNum::Div1 { .. }, },
                        EvalOp::Num { number: EncodedNumber { number: Number::Positive(PositiveNumber { value: 0, }), .. }, },
                    ) =>
                        return Err(Error::DivisionByZero),

                    // div1 on two numbers with different modulation
                    (
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Div1 {
                                captured: number_a @ EncodedNumber {
                                    modulation: Modulation::Modulated,
                                    ..
                                },
                            },
                        },
                        EvalOp::Num { number: number_b @ EncodedNumber { modulation: Modulation::Demodulated, .. }, },
                    ) |
                    (
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Div1 {
                                captured: number_a @ EncodedNumber {
                                    modulation: Modulation::Demodulated,
                                    ..
                                },
                            },
                        },
                        EvalOp::Num { number: number_b @ EncodedNumber { modulation: Modulation::Modulated, .. }, },
                    ) =>
                        return Err(Error::TwoNumbersOpInDifferentModulation { number_a, number_b, }),

                    // div1 on two positive
                    (
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Div1 {
                                captured: EncodedNumber {
                                    number: Number::Positive(PositiveNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        },
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
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Div1 {
                                captured: EncodedNumber {
                                    number: Number::Positive(PositiveNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        },
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
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Div1 {
                                captured: EncodedNumber {
                                    number: Number::Negative(NegativeNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        },
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
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Div1 {
                                captured: EncodedNumber {
                                    number: Number::Negative(NegativeNumber { value: value_a, }),
                                    modulation,
                                },
                            },
                        },
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

                    // eq0 on a number
                    (
                        State::EvalAppArgNum { fun: EvalFunNum::Eq0, },
                        EvalOp::Num { number, },
                    ) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Eq1 {
                            captured: number,
                        })),

                    // eq1 on two equal numbers
                    (
                        State::EvalAppArgNum { fun: EvalFunNum::Eq1 { captured: number_a, }, },
                        EvalOp::Num { number: number_b, },
                    ) if number_a == number_b =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::True0)),

                    // eq1 on two different numbers
                    (
                        State::EvalAppArgNum { fun: EvalFunNum::Eq1 { .. }, },
                        EvalOp::Num { .. },
                    ) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::False0)),

                    // lt0 on a number
                    (
                        State::EvalAppArgNum { fun: EvalFunNum::Lt0, },
                        EvalOp::Num { number, },
                    ) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Lt1 {
                            captured: number,
                        })),

                    // lt1 on two numbers with different modulation
                    (
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Lt1 {
                                captured: number_a @ EncodedNumber {
                                    modulation: Modulation::Modulated,
                                    ..
                                },
                            },
                        },
                        EvalOp::Num { number: number_b @ EncodedNumber { modulation: Modulation::Demodulated, .. }, },
                    ) |
                    (
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Lt1 {
                                captured: number_a @ EncodedNumber {
                                    modulation: Modulation::Demodulated,
                                    ..
                                },
                            },
                        },
                        EvalOp::Num { number: number_b @ EncodedNumber { modulation: Modulation::Modulated, .. }, },
                    ) =>
                        return Err(Error::TwoNumbersOpInDifferentModulation { number_a, number_b, }),

                    // lt1 on two positive
                    (
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Lt1 {
                                captured: EncodedNumber { number: Number::Positive(PositiveNumber { value: value_a, }), .. },
                            },
                        },
                        EvalOp::Num {
                            number: EncodedNumber { number: Number::Positive(PositiveNumber { value: value_b, }), .. },
                        },
                    ) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(if value_a < value_b {
                            EvalFunAbs::True0
                        } else {
                            EvalFunAbs::False0
                        })),

                    // lt1 on two negative
                    (
                        State::EvalAppArgNum {
                            fun: EvalFunNum::Lt1 {
                                captured: EncodedNumber { number: Number::Negative(NegativeNumber { value: value_a, }), .. },
                            },
                        },
                        EvalOp::Num {
                            number: EncodedNumber { number: Number::Negative(NegativeNumber { value: value_b, }), .. },
                        },
                    ) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(if value_a < value_b {
                            EvalFunAbs::True0
                        } else {
                            EvalFunAbs::False0
                        })),

                    // lt1 on positive and negative
                    (
                        State::EvalAppArgNum { fun: EvalFunNum::Lt1 { captured: EncodedNumber { number: Number::Positive(..), .. }, }, },
                        EvalOp::Num { number: EncodedNumber { number: Number::Negative(..), .. }, },
                    ) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::False0)),

                    // lt1 on negative and positive
                    (
                        State::EvalAppArgNum { fun: EvalFunNum::Lt1 { captured: EncodedNumber { number: Number::Negative(..), .. }, }, },
                        EvalOp::Num { number: EncodedNumber { number: Number::Positive(..), .. }, },
                    ) =>
                        eval_op = EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::True0)),

                    // neg on zero
                    (
                        State::EvalAppArgNum { fun: EvalFunNum::Neg0, },
                        number @ EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Positive(PositiveNumber { value: 0, }),
                                ..
                            },
                        },
                    ) =>
                        eval_op = number,

                    // neg on positive number
                    (
                        State::EvalAppArgNum { fun: EvalFunNum::Neg0, },
                        EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Positive(PositiveNumber { value, }),
                                modulation,
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Negative(NegativeNumber { value: -(value as isize), }),
                                modulation,
                            },
                        },

                    // neg on negative number
                    (
                        State::EvalAppArgNum { fun: EvalFunNum::Neg0, },
                        EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Negative(NegativeNumber { value, }),
                                modulation,
                            },
                        },
                    ) =>
                        eval_op = EvalOp::Num {
                            number: EncodedNumber {
                                number: Number::Positive(PositiveNumber { value: ((-value) as usize), }),
                                modulation,
                            },
                        },

                    // number type argument fun on a fun
                    (State::EvalAppArgNum { .. }, EvalOp::Fun(fun)) =>
                        return Err(Error::AppExpectsNumButFunProvided { fun: EvalOp::Fun(fun).render(), }),

                    // fun on abs
                    (State::EvalAppArgNum { fun }, EvalOp::Abs(arg_ast_node)) =>
                        match env.lookup_ast(&arg_ast_node) {
                            Some(subst_ast_node) => {
                                states.push(StackFrame { root, state: State::EvalAppArgNum { fun, }, });
                                ast_node = Rc::new(subst_ast_node.clone());
                                break;
                            },
                            None => {
                                let mut fun_ops_iter = EvalOp::Fun(EvalFun::ArgNum(fun))
                                    .render()
                                    .0
                                    .into_iter();
                                let ast_node = match fun_ops_iter.next() {
                                    None =>
                                        panic!("render failure: expected op, but got none"),
                                    Some(Op::App) =>
                                        match fun_ops_iter.next() {
                                            None =>
                                                panic!("render failure: expected op fun, but got none"),
                                            Some(op_a) =>
                                                match fun_ops_iter.next() {
                                                    None =>
                                                        panic!("render failure: expected op {:?} arg, but got none", op_a),
                                                    Some(op_b) =>
                                                        match fun_ops_iter.next() {
                                                            None =>
                                                                AstNode::App {
                                                                    fun: Rc::new(AstNode::App {
                                                                        fun: Rc::new(AstNode::Literal { value: op_a, }),
                                                                        arg: Rc::new(AstNode::Literal { value: op_b, }),
                                                                    }),
                                                                    arg: arg_ast_node,
                                                                },
                                                            Some(..) =>
                                                                unreachable!(),
                                                        },
                                                },
                                        },
                                    Some(op_a) =>
                                        AstNode::App {
                                            fun: Rc::new(AstNode::Literal { value: op_a, }),
                                            arg: arg_ast_node,
                                        },
                                };
                                eval_op = EvalOp::Abs(Rc::new(ast_node));
                            },
                        },
                }

                let maybe_cache = match &eval_op {
                    EvalOp::Num { ref number, } =>
                        Some(Rc::new(AstNode::Literal { value: Op::Const(Const::EncodedNumber(number.clone())), })),
                    EvalOp::Abs(ref ast_node) =>
                        Some(ast_node.clone()),
                    EvalOp::Fun(..) =>
                        None,
                };
                if let Some(value) = maybe_cache {
                    cache.memo(root, value);
                }
            }
        }
    }

    pub fn eval_force_list(&self, mut list_ops: Ops, env: &Env, cache: &mut Cache) -> Result<Ops, Error> {
        let mut forced_ops = Ops(Vec::new());
        loop {
            let ops = self.eval_ops_on(&[Op::App, Op::Const(Const::Fun(Fun::IsNil))], &list_ops, env, cache)?;
            if let [Op::Const(Const::Fun(Fun::True))] = &*ops.0 {
                break;
            }

            let ops = self.eval_ops_on(&[Op::App, Op::Const(Const::Fun(Fun::Car))], &list_ops, env, cache)?;
            forced_ops.0.push(Op::App);
            forced_ops.0.push(Op::App);
            forced_ops.0.push(Op::Const(Const::Fun(Fun::Cons)));
            forced_ops.0.extend(ops.0);

            list_ops = self.eval_ops_on(&[Op::App, Op::Const(Const::Fun(Fun::Cdr))], &list_ops, env, cache)?;
        }
        forced_ops.0.push(Op::Const(Const::Fun(Fun::Nil)));
        Ok(forced_ops)
    }

    fn eval_draw(&self, points: Rc<AstNode>, env: &Env, cache: &mut Cache) -> Result<Picture, Error> {
        let mut points_vec = Vec::new();
        let mut points_ops = points.render();
        loop {
            let ops = self.eval_ops_on(&[Op::App, Op::Const(Const::Fun(Fun::IsNil))], &points_ops, env, cache)?;
            if let [Op::Const(Const::Fun(Fun::True))] = &*ops.0 {
                break;
            }

            let coord_ops = self.eval_ops_on(&[Op::App, Op::Const(Const::Fun(Fun::Car))], &points_ops, env, cache)?;

            let mut ops = self.eval_ops_on(&[Op::App, Op::Const(Const::Fun(Fun::Car))], &coord_ops, env, cache)?;
            let coord_a = match (ops.0.len(), ops.0.pop()) {
                (1, Some(Op::Const(Const::EncodedNumber(number)))) =>
                    number,
                _ =>
                    return Err(Error::InvalidCoordForDrawArg),
            };
            let mut ops = self.eval_ops_on(&[Op::App, Op::Const(Const::Fun(Fun::Cdr))], &coord_ops, env, cache)?;
            let coord_b = match (ops.0.len(), ops.0.pop()) {
                (1, Some(Op::Const(Const::EncodedNumber(number)))) =>
                    number,
                _ =>
                    return Err(Error::InvalidCoordForDrawArg),
            };
            points_vec.push(Coord {
                x: coord_a,
                y: coord_b,
            });

            points_ops = self.eval_ops_on(&[Op::App, Op::Const(Const::Fun(Fun::Cdr))], &points_ops, env, cache)?;
        }
        Ok(Picture { points: points_vec, })
    }

    fn eval_multiple_draw(&self, points_list_of_lists: Rc<AstNode>, env: &Env, cache: &mut Cache) -> Result<Rc<AstNode>, Error> {
        let mut list_ops = points_list_of_lists.render();

        let mut output_ops = Ops(vec![]);
        loop {
            let ops = self.eval_ops_on(&[Op::App, Op::Const(Const::Fun(Fun::IsNil))], &list_ops, env, cache)?;
            if let [Op::Const(Const::Fun(Fun::True))] = &*ops.0 {
                break;
            }

            let child_ops = self.eval_ops_on(&[Op::App, Op::Const(Const::Fun(Fun::Car))], &list_ops, env, cache)?;
            output_ops.0.push(Op::App);
            output_ops.0.push(Op::App);
            output_ops.0.push(Op::Const(Const::Fun(Fun::Cons)));
            output_ops.0.push(Op::App);
            output_ops.0.push(Op::Const(Const::Fun(Fun::Draw)));
            output_ops.0.extend(child_ops.0);

            list_ops = self.eval_ops_on(&[Op::App, Op::Const(Const::Fun(Fun::Cdr))], &list_ops, env, cache)?;

        }
        output_ops.0.push(Op::Const(Const::Fun(Fun::Nil)));

        match self.build_tree(output_ops)? {
            Ast::Empty =>
                unreachable!(), // we should got at least nil
            Ast::Tree(ast_node) =>
                Ok(Rc::new(ast_node)),
        }
    }

    fn eval_send(&self, send_args: Rc<AstNode>, env: &Env, cache: &mut Cache) -> Result<Rc<AstNode>, Error> {
        let send_args = self.eval_mod(send_args, env, cache)?;
        let args_ops = send_args.render();
        let send_list_val = self.eval_ops_to_list_val(args_ops, env, cache)?;
        let send_cons_list = match send_list_val {
            encoder::ListVal::Number(number) =>
                return Err(Error::ExpectedListArgForSendButGotNumber { number, }),
            encoder::ListVal::Cons(value) =>
                *value,
        };
        let send_mod = send_cons_list.modulate_to_string();

        // perform send
        let recv_mod = if let Some(outer_channel) = &self.outer_channel {
            let (tx, rx) = mpsc::channel();

            let outer_send_result = outer_channel.unbounded_send(OuterRequest::ProxySend {
                modulated_req: send_mod,
                modulated_rep: tx,
            });
            if let Err(..) = outer_send_result {
                return Err(Error::OuterChannelIsClosed);
            }

            match rx.recv() {
                Ok(response) =>
                    response,
                Err(..) =>
                    return Err(Error::OuterChannelIsClosed),
            }
        } else {
            return Err(Error::SendOpIsNotSupportedWithoutOuterChannel);
        };

        let recv_cons_list = encoder::ConsList::demodulate_from_string(&recv_mod)
            .map_err(Error::ConsListDem)?;
        let recv_ops = list_val_to_ops(encoder::ListVal::Cons(Box::new(recv_cons_list)));

        match self.build_tree(recv_ops)? {
            Ast::Empty =>
                unreachable!(), // list_val_to_ops should return at least nil
            Ast::Tree(ast_node) =>
                Ok(Rc::new(ast_node)),
        }
    }

    fn eval_render(&self, render_args: Rc<AstNode>, env: &Env, cache: &mut Cache) -> Result<Rc<AstNode>, Error> {
        let mut render_ops = render_args.render();

        let mut pictures = Vec::new();
        loop {
            let ops = self.eval_ops_on(&[Op::App, Op::Const(Const::Fun(Fun::IsNil))], &render_ops, env, cache)?;
            if let [Op::Const(Const::Fun(Fun::True))] = &*ops.0 {
                break;
            }

            let mut ops = self.eval_ops_on(&[Op::App, Op::Const(Const::Fun(Fun::Car))], &render_ops, env, cache)?;
            match (ops.0.len(), ops.0.pop()) {
                (_, None) =>
                    unreachable!(),
                (1, Some(Op::Const(Const::Picture(picture)))) => {
                    pictures.push(picture);
                },
                (_, Some(last_item)) => {
                    ops.0.push(last_item);
                    return Err(Error::RenderItemIsNotAPicture { ops, });
                },
            }

            render_ops = self.eval_ops_on(&[Op::App, Op::Const(Const::Fun(Fun::Cdr))], &render_ops, env, cache)?;
        }

        // perform send
        if let Some(outer_channel) = &self.outer_channel {
            let outer_send_result = outer_channel.unbounded_send(OuterRequest::RenderPictures { pictures, });
            if let Err(..) = outer_send_result {
                return Err(Error::OuterChannelIsClosed);
            }
        } else {
            return Err(Error::RenderOpIsNotSupportedWithoutOuterChannel);
        };

        Ok(Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::True)), }))
    }

    fn eval_mod(&self, args: Rc<AstNode>, env: &Env, cache: &mut Cache) -> Result<Rc<AstNode>, Error> {
        let args_ops = args.render();
        let ops = self.eval_num_list_map(args_ops, &|num| match num {
            EncodedNumber { number, modulation: Modulation::Demodulated, } =>
                Ok(EncodedNumber { number, modulation: Modulation::Modulated, }),
            number @ EncodedNumber { modulation: Modulation::Modulated, .. } =>
                Err(Error::ModOnModulatedNumber { number, }),
        }, env, cache)?;

        match self.build_tree(ops)? {
            Ast::Empty =>
                unreachable!(), // eval_num_list_map should return at least nil
            Ast::Tree(ast_node) =>
                Ok(Rc::new(ast_node)),
        }
    }

    fn eval_dem(&self, args: Rc<AstNode>, env: &Env, cache: &mut Cache) -> Result<Rc<AstNode>, Error> {
        let args_ops = args.render();
        let ops = self.eval_num_list_map(args_ops, &|num| match num {
            EncodedNumber { number, modulation: Modulation::Modulated, } =>
                Ok(EncodedNumber { number, modulation: Modulation::Demodulated, }),
            number @ EncodedNumber { modulation: Modulation::Demodulated, .. } =>
                Err(Error::DemOnDemodulatedNumber { number, }),
        }, env, cache)?;

        match self.build_tree(ops)? {
            Ast::Empty =>
                unreachable!(), // eval_num_list_map should return at least nil
            Ast::Tree(ast_node) =>
                Ok(Rc::new(ast_node)),
        }
    }

    fn eval_modem(&self, ast_node: Rc<AstNode>, env: &Env, cache: &mut Cache) -> Result<Rc<AstNode>, Error> {
        let ast_node = self.eval_mod(ast_node, env, cache)?;
        let ast_node = self.eval_dem(ast_node, env, cache)?;
        Ok(ast_node)
    }

    fn eval_num_list_map<F>(&self, mut list_ops: Ops, trans: &F, env: &Env, cache: &mut Cache) -> Result<Ops, Error>
    where F: Fn(EncodedNumber) -> Result<EncodedNumber, Error>
    {
        match (list_ops.0.len(), list_ops.0.pop()) {
            (_, None) =>
                unreachable!(),
            (1, Some(Op::Const(Const::EncodedNumber(number)))) => {
                let transformed = trans(number)?;
                return Ok(Ops(vec![Op::Const(Const::EncodedNumber(transformed))]));
            },
            (_, Some(last_item)) =>
                list_ops.0.push(last_item),
        }

        let mut trans_ops = Ops(vec![]);
        loop {
            let ops = self.eval_ops_on(&[Op::App, Op::Const(Const::Fun(Fun::IsNil))], &list_ops, env, cache)?;
            if let [Op::Const(Const::Fun(Fun::True))] = &*ops.0 {
                break;
            }

            let ops = self.eval_ops_on(&[Op::App, Op::Const(Const::Fun(Fun::Car))], &list_ops, env, cache)?;
            let child_ops = self.eval_num_list_map(ops, trans, env, cache)?;
            trans_ops.0.push(Op::App);
            trans_ops.0.push(Op::App);
            trans_ops.0.push(Op::Const(Const::Fun(Fun::Cons)));
            trans_ops.0.extend(child_ops.0);

            list_ops = self.eval_ops_on(&[Op::App, Op::Const(Const::Fun(Fun::Cdr))], &list_ops, env, cache)?;
            match (list_ops.0.len(), list_ops.0.pop()) {
                (_, None) =>
                    unreachable!(),
                (1, Some(Op::Const(Const::EncodedNumber(number)))) => {
                    let transformed = trans(number)?;
                    trans_ops.0.push(Op::Const(Const::EncodedNumber(transformed)));
                    return Ok(trans_ops);
                },
                (_, Some(Op::App)) =>
                    list_ops.0.push(Op::App),
                (_, Some(Op::Const(Const::Fun(Fun::Nil)))) =>
                    list_ops.0.push(Op::Const(Const::Fun(Fun::Nil))),
                (_, Some(last_item)) => {
                    list_ops.0.push(last_item);
                    return Err(Error::InvalidConsListItem { ops: list_ops, });
                },
            }
        }
        trans_ops.0.push(Op::Const(Const::Fun(Fun::Nil)));

        Ok(trans_ops)
    }

    fn eval_ops_to_list_val(&self, mut list_ops: Ops, env: &Env, cache: &mut Cache) -> Result<encoder::ListVal, Error> {
        match (list_ops.0.len(), list_ops.0.pop()) {
            (_, None) =>
                unreachable!(),
            (1, Some(Op::Const(Const::EncodedNumber(number @ EncodedNumber { modulation: Modulation::Modulated, .. })))) =>
                return Ok(encoder::ListVal::Number(number)),
            (1, Some(Op::Const(Const::EncodedNumber(number @ EncodedNumber { modulation: Modulation::Demodulated, .. })))) =>
                return Err(Error::DemodulatedNumberInList { number, }),
            (_, Some(last_item)) =>
                list_ops.0.push(last_item),
        }

        let mut cons_stack = vec![];
        loop {
            let ops = self.eval_ops_on(&[Op::App, Op::Const(Const::Fun(Fun::IsNil))], &list_ops, env, cache)?;
            if let [Op::Const(Const::Fun(Fun::True))] = &*ops.0 {
                break;
            }

            let ops = self.eval_ops_on(&[Op::App, Op::Const(Const::Fun(Fun::Car))], &list_ops, env, cache)?;
            let child_list_val = self.eval_ops_to_list_val(ops, env, cache)?;
            cons_stack.push(child_list_val);

            list_ops = self.eval_ops_on(&[Op::App, Op::Const(Const::Fun(Fun::Cdr))], &list_ops, env, cache)?;
        }

        let mut cons_list = encoder::ConsList::Nil;
        while let Some(item) = cons_stack.pop() {
            cons_list = encoder::ConsList::Cons(
                item,
                encoder::ListVal::Cons(Box::new(cons_list)),
            );
        }
        Ok(encoder::ListVal::Cons(Box::new(cons_list)))
    }

    fn eval_ops_on(&self, ops: &[Op], on_script: &Ops, env: &Env, cache: &mut Cache) -> Result<Ops, Error> {
        let mut script = Ops(Vec::with_capacity(ops.len() + on_script.0.len()));
        script.0.clear();
        script.0.extend(ops.iter().cloned());
        script.0.extend(on_script.0.iter().cloned());
        let tree = self.build_tree(script)?;
        self.eval_cache(tree, env, cache)
    }

    fn eval_interact(&self, protocol: Rc<AstNode>, state: Rc<AstNode>, vector: Rc<AstNode>, _env: &Env) -> Result<Rc<AstNode>, Error> {
        Ok(Rc::new(AstNode::App {
            fun: Rc::new(AstNode::App {
                fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::F38)), }),
                arg: protocol.clone(),
            }),
            arg: Rc::new(AstNode::App {
                fun: Rc::new(AstNode::App {
                    fun: protocol,
                    arg: state,
                }),
                arg: vector,
            }),
        }))
    }

    fn eval_f38(&self, protocol: Rc<AstNode>, tuple3: Rc<AstNode>, _env: &Env) -> Result<Rc<AstNode>, Error> {
        Ok(Rc::new(AstNode::App {
            fun: Rc::new(AstNode::App {
                fun: Rc::new(AstNode::App {
                    fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::If0)), }),
                    arg: Rc::new(AstNode::App {
                        fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Car)), }),
                        arg: tuple3.clone(),
                    }),
                }),
                arg: Rc::new(AstNode::App {
                    fun: Rc::new(AstNode::App {
                        fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cons)), }),
                        arg: Rc::new(AstNode::App {
                            fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Modem)), }),
                            arg: Rc::new(AstNode::App {
                                fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Car)), }),
                                arg: Rc::new(AstNode::App {
                                    fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cdr)), }),
                                    arg: tuple3.clone(),
                                }),
                            }),
                        }),
                    }),
                    arg: Rc::new(AstNode::App {
                        fun: Rc::new(AstNode::App {
                            fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cons)), }),
                            arg: Rc::new(AstNode::App {
                                fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::MultipleDraw)), }),
                                arg: Rc::new(AstNode::App {
                                    fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Car)), }),
                                    arg: Rc::new(AstNode::App {
                                        fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cdr)), }),
                                        arg: Rc::new(AstNode::App {
                                            fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cdr)), }),
                                            arg: tuple3.clone(),
                                        }),
                                    }),
                                }),
                            }),
                        }),
                        arg: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Nil)), }),
                    }),
                }),
            }),
            arg: Rc::new(AstNode::App {
                fun: Rc::new(AstNode::App {
                    fun: Rc::new(AstNode::App {
                        fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Interact)), }),
                        arg: protocol,
                    }),
                    arg: Rc::new(AstNode::App {
                        fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Modem)), }),
                        arg: Rc::new(AstNode::App {
                            fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Car)), }),
                            arg: Rc::new(AstNode::App {
                                fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cdr)), }),
                                arg: tuple3.clone(),
                            }),
                        }),
                    }),
                }),
                arg: Rc::new(AstNode::App {
                    fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Send)), }),
                    arg: Rc::new(AstNode::App {
                        fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Car)), }),
                        arg: Rc::new(AstNode::App {
                            fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cdr)), }),
                            arg: Rc::new(AstNode::App {
                                fun: Rc::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cdr)), }),
                                arg: tuple3,
                            }),
                        }),
                    }),
                }),
            }),
        }))
    }
}

fn list_val_to_ops(mut value: encoder::ListVal) -> Ops {
    let mut ops = Ops(vec![]);
    loop {
        match value {
            encoder::ListVal::Number(number) => {
                ops.0.push(Op::Const(Const::EncodedNumber(number)));
                break;
            },
            encoder::ListVal::Cons(cons_list) =>
                match *cons_list {
                    encoder::ConsList::Nil => {
                        ops.0.push(Op::Const(Const::Fun(Fun::Nil)));
                        break;
                    },
                    encoder::ConsList::Cons(car, cdr) => {
                        ops.0.push(Op::App);
                        ops.0.push(Op::App);
                        ops.0.push(Op::Const(Const::Fun(Fun::Cons)));
                        let car_ops = list_val_to_ops(car);
                        ops.0.extend(car_ops.0);
                        value = cdr;
                    },
                },
        }
    }
    ops
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum EvalOp {
    Num { number: EncodedNumber, },
    Fun(EvalFun),
    Abs(Rc<AstNode>),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EvalFun {
    ArgNum(EvalFunNum),
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
    Eq0,
    Eq1 { captured: EncodedNumber, },
    Lt0,
    Lt1 { captured: EncodedNumber, },
    Neg0,
    IfZero0,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EvalFunFun {
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EvalFunAbs {
    True0,
    True1 { captured: Rc<AstNode>, },
    False0,
    False1 { captured: Rc<AstNode>, },
    I0,
    C0,
    C1 { x: Rc<AstNode>, },
    C2 { x: Rc<AstNode>, y: Rc<AstNode>, },
    B0,
    B1 { x: Rc<AstNode>, },
    B2 { x: Rc<AstNode>, y: Rc<AstNode>, },
    S0,
    S1 { x: Rc<AstNode>, },
    S2 { x: Rc<AstNode>, y: Rc<AstNode>, },
    Cons0,
    Cons1 { x: Rc<AstNode>, },
    Cons2 { x: Rc<AstNode>, y: Rc<AstNode>, },
    Car0,
    Cdr0,
    Nil0,
    IsNil0,
    IfZero1 { cond: EncodedNumber, },
    IfZero2 { cond: EncodedNumber, true_clause: Rc<AstNode>, },
    Draw0,
    MultipleDraw0,
    Send0,
    Mod0,
    Dem0,
    Modem0,
    Interact0,
    Interact1 { protocol: Rc<AstNode>, },
    Interact2 { protocol: Rc<AstNode>, state: Rc<AstNode>, },
    F38_0,
    F38_1 { protocol: Rc<AstNode>, },
    Render0,
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
                EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Eq0)),
            Op::Const(Const::Fun(Fun::Lt)) =>
                EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Lt0)),
            Op::Const(Const::Fun(Fun::Mod)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Mod0)),
            Op::Const(Const::Fun(Fun::Dem)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Dem0)),
            Op::Const(Const::Fun(Fun::Send)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Send0)),
            Op::Const(Const::Fun(Fun::Neg)) =>
                EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Neg0)),
            Op::Const(Const::Fun(Fun::S)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::S0)),
            Op::Const(Const::Fun(Fun::C)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::C0)),
            Op::Const(Const::Fun(Fun::B)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::B0)),
            Op::Const(Const::Fun(Fun::True)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::True0)),
            Op::Const(Const::Fun(Fun::False)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::False0)),
            Op::Const(Const::Fun(Fun::Pwr2)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::I)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::I0)),
            Op::Const(Const::Fun(Fun::Cons)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Cons0)),
            Op::Const(Const::Fun(Fun::Car)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Car0)),
            Op::Const(Const::Fun(Fun::Cdr)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Cdr0)),
            Op::Const(Const::Fun(Fun::Nil)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Nil0)),
            Op::Const(Const::Fun(Fun::IsNil)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::IsNil0)),
            Op::Const(Const::Fun(Fun::Vec)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Cons0)),
            Op::Const(Const::Fun(Fun::Draw)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Draw0)),
            Op::Const(Const::Fun(Fun::Chkb)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::MultipleDraw)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::MultipleDraw0)),
            Op::Const(Const::Fun(Fun::If0)) =>
                EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::IfZero0)),
            Op::Const(Const::Fun(Fun::Interact)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Interact0)),
            Op::Const(Const::Fun(Fun::Modem)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Modem0)),
            Op::Const(Const::Fun(Fun::Galaxy)) =>
                unreachable!(), // should be renamed to variable with name "-1"
            Op::Const(Const::Picture(picture)) =>
                EvalOp::Abs(Rc::new(AstNode::Literal { value: Op::Const(Const::Picture(picture)), })),
            Op::Variable(var) =>
                EvalOp::Abs(Rc::new(AstNode::Literal { value: Op::Variable(var), })),
            Op::Const(Const::Fun(Fun::Checkerboard)) =>
                unimplemented!(),
            Op::Const(Const::Fun(Fun::F38)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::F38_0)),
            Op::Const(Const::Fun(Fun::Render)) =>
                EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Render0)),
            Op::App =>
                unreachable!(), // should be processed by ast builder
            Op::Syntax(..) =>
                unreachable!(), // should be processed by ast builder
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
                    Op::App,
                    Op::Const(Const::Fun(Fun::Sum)),
                    Op::Const(Const::EncodedNumber(captured)),
                ]),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Mul0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Mul))]),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Mul1 { captured, })) =>
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::Mul)),
                    Op::Const(Const::EncodedNumber(captured)),
                ]),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Div0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Div))]),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Div1 { captured, })) =>
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::Div)),
                    Op::Const(Const::EncodedNumber(captured)),
                ]),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Eq0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Eq))]),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Eq1 { captured, })) =>
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::Eq)),
                    Op::Const(Const::EncodedNumber(captured)),
                ]),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Lt0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Lt))]),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Lt1 { captured, })) =>
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::Lt)),
                    Op::Const(Const::EncodedNumber(captured)),
                ]),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Neg0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Neg))]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::True0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::True))]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::True1 { captured, })) => {
                let mut ops = vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::True)),
                ];
                ops.extend(captured.render().0);
                Ops(ops)
            },
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::False0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::False))]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::False1 { captured, })) => {
                let mut ops = vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::False)),
                ];
                ops.extend(captured.render().0);
                Ops(ops)
            },
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::I0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::I))]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::C0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::C))]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::C1 { x, })) => {
                let mut ops = vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::C)),
                ];
                ops.extend(x.render().0);
                Ops(ops)
            },
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::C2 { x, y, })) => {
                let mut ops = vec![
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::C)),
                ];
                ops.extend(x.render().0);
                ops.extend(y.render().0);
                Ops(ops)
            },
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::B0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::B))]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::B1 { x, })) => {
                let mut ops = vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::B)),
                ];
                ops.extend(x.render().0);
                Ops(ops)
            },
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::B2 { x, y, })) => {
                let mut ops = vec![
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::B)),
                ];
                ops.extend(x.render().0);
                ops.extend(y.render().0);
                Ops(ops)
            },
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::S0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::S))]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::S1 { x, })) => {
                let mut ops = vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::S)),
                ];
                ops.extend(x.render().0);
                Ops(ops)
            },
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::S2 { x, y, })) => {
                let mut ops = vec![
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::S)),
                ];
                ops.extend(x.render().0);
                ops.extend(y.render().0);
                Ops(ops)
            },
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Cons0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Cons))]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Cons1 { x, })) => {
                let mut ops = vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::Cons)),
                ];
                ops.extend(x.render().0);
                Ops(ops)
            },
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Cons2 { x, y, })) => {
                let mut ops = vec![
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::Cons)),
                ];
                ops.extend(x.render().0);
                ops.extend(y.render().0);
                Ops(ops)
            },
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Car0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Car))]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Cdr0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Cdr))]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Nil0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Nil))]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::IsNil0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::IsNil))]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Draw0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Draw))]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::MultipleDraw0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::MultipleDraw))]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Send0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Send))]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Mod0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Mod))]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Dem0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Dem))]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Modem0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Modem))]),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::IfZero0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::If0))]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::IfZero1 { cond, })) =>
                Ops(vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::If0)),
                    Op::Const(Const::EncodedNumber(cond)),
                ]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::IfZero2 { cond, true_clause, })) => {
                let mut ops = vec![
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::If0)),
                    Op::Const(Const::EncodedNumber(cond)),
                ];
                ops.extend(true_clause.render().0);
                Ops(ops)
            },
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Interact0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Interact))]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Interact1 { protocol, })) => {
                let mut ops = vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::Interact)),
                ];
                ops.extend(protocol.render().0);
                Ops(ops)
            },
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Interact2 { protocol, state, })) => {
                let mut ops = vec![
                    Op::App,
                    Op::App,
                    Op::Const(Const::Fun(Fun::Interact)),
                ];
                ops.extend(protocol.render().0);
                ops.extend(state.render().0);
                Ops(ops)
            },
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::F38_0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::F38))]),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::F38_1 { protocol, })) => {
                let mut ops = vec![
                    Op::App,
                    Op::Const(Const::Fun(Fun::F38)),
                ];
                ops.extend(protocol.render().0);
                Ops(ops)
            },
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Render0)) =>
                Ops(vec![Op::Const(Const::Fun(Fun::Render))]),

            EvalOp::Abs(ast_node) =>
                ast_node.render(),
        }
    }
}

impl AstNode {
    pub fn render(self: Rc<AstNode>) -> Ops {
        enum State {
            RenderAppFun { arg: Rc<AstNode>, },
            RenderAppArg,
        }

        let mut ops = vec![];
        let mut ast_node = self;
        let mut stack = vec![];
        loop {
            match &*ast_node {
                AstNode::Literal { value, } =>
                    ops.push(value.clone()),
                AstNode::App { fun, arg, } => {
                    ops.push(Op::App);
                    stack.push(State::RenderAppFun { arg: arg.clone(), });
                    ast_node = fun.clone();
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
