use std::{
    rc::Rc,
    sync::mpsc,
    collections::{
        HashMap,
        hash_map::DefaultHasher,
    },
    hash::{
        Hash,
        Hasher,
    },
};

use futures::{
    channel::mpsc::UnboundedSender,
};

use lru::LruCache;

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
    AppExpectsNumButFunProvided { fun: Ops, arg: Ops, },
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
    ExpectedListArgForModButGotNumber { number: EncodedNumber, },
    ConsListDem(encoder::Error),
    SendOpIsNotSupportedWithoutOuterChannel,
    RenderOpIsNotSupportedWithoutOuterChannel,
    OuterChannelIsClosed,
    DemodulatedNumberInList { number: EncodedNumber, },
    RenderItemIsNotAPicture { ops: Ops, },
    InvalidConsListItem { ops: Ops, },
    ApplyingModulatedBitsOn { bits: String, arg: Ops, },
    ApplyingFunOnModulatedBits { fun: Ops, },
    ApplyingCarToLiteral { value: Op, },
    ApplyingCarToInvalidFun { fun: Ops, },
    ApplyingCdrToLiteral { value: Op, },
    ApplyingCdrToInvalidFun { fun: Ops, },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ast {
    Empty,
    Tree(Rc<AstNodeH>),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AstNodeH {
    hash: u64,
    pub kind: AstNode,
}

impl AstNodeH {
    pub fn new(kind: AstNode) -> AstNodeH {
        let mut s = DefaultHasher::new();
        match &kind {
            AstNode::Literal { value, } => {
                0.hash(&mut s);
                value.hash(&mut s);
            },
            AstNode::App { fun, arg, } => {
                1.hash(&mut s);
                fun.hash.hash(&mut s);
                arg.hash.hash(&mut s);
            },
        }
        AstNodeH { hash: s.finish(), kind, }
    }
}

impl Hash for AstNodeH {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AstNode {
    Literal { value: Op, },
    App { fun: Rc<AstNodeH>, arg: Rc<AstNodeH>, },
}

#[derive(Debug)]
pub struct Env {
    forward: HashMap<Rc<AstNodeH>, Rc<AstNodeH>>,
    backward: HashMap<Rc<AstNodeH>, Rc<AstNodeH>>,
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
            if let AstNode::Literal { value: Op::Variable(..), } = left.kind {
                self.forward.insert(left.clone(), right.clone());
            }
            if let AstNode::Literal { value: Op::Variable(..), } = right.kind {
                self.backward.insert(right, left);
            }
        }
    }

    pub fn lookup_ast(&self, key: &Rc<AstNodeH>) -> Option<&Rc<AstNodeH>> {
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
    memo: LruCache<Rc<AstNodeH>, Rc<AstNodeH>>,
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            memo: LruCache::new(128 * 1024),
        }
    }

    pub fn get(&mut self, key: &Rc<AstNodeH>) -> Option<Rc<AstNodeH>> {
        if let Some(ast_node) = self.memo.get(key) {
            Some(ast_node.clone())
        } else {
            None
        }
    }

    pub fn memo(&mut self, key: Rc<AstNodeH>, value: Rc<AstNodeH>) {
        self.memo.put(key, value);
    }

    pub fn clear(&mut self) {
        self.memo.clear();
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
        Ast::Tree(Rc::new(AstNodeH::new(
            AstNode::Literal { value: Op::Variable(Variable { name: Number::Negative(NegativeNumber { value: -2, }), }), },
        )))
    }

    pub fn build_tree(&self, Ops(mut ops): Ops) -> Result<Ast, Error> {
        // println!("Interpreter::build_tree() on {} ops", ops.len());

        enum State {
            AwaitAppFun,
            AwaitAppArg { fun: Rc<AstNodeH>, },
            ListBegin,
            ListPush { element: Rc<AstNodeH>, },
            ListContinue,
            ListContinueComma,
        }

        let mut states = vec![];
        ops.reverse();
        loop {
            let mut maybe_node: Option<AstNodeH> = match ops.pop() {
                None =>
                    None,
                Some(Op::Const(Const::Fun(Fun::Galaxy))) =>
                    Some(AstNodeH::new(AstNode::Literal {
                        value: Op::Variable(Variable {
                            name: Number::Negative(NegativeNumber {
                                value: -1,
                            }),
                        }),
                    })),
                Some(Op::Syntax(Syntax::LeftParen)) => {
                    states.push(State::ListBegin);
                    continue;
                },
                Some(value @ Op::Const(..)) |
                Some(value @ Op::Variable(..)) |
                Some(value @ Op::Syntax(..)) =>
                    Some(AstNodeH::new(AstNode::Literal { value: value, })),
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
                        return Ok(Ast::Tree(Rc::new(node))),
                    (Some(State::AwaitAppFun), None) =>
                        return Err(Error::NoAppFunProvided),
                    (Some(State::AwaitAppFun), Some(node)) => {
                        states.push(State::AwaitAppArg { fun: Rc::new(node), });
                        break;
                    },
                    (Some(State::AwaitAppArg { fun, }), None) =>
                        return Err(Error::NoAppArgProvided { fun: fun.render(), }),
                    (Some(State::AwaitAppArg { fun, }), Some(node)) => {
                        maybe_node = Some(AstNodeH::new(AstNode::App {
                            fun: fun,
                            arg: Rc::new(node),
                        }));
                    },
                    (Some(State::ListBegin), None) =>
                        return Err(Error::ListNotClosed),
                    (Some(State::ListBegin), Some(AstNodeH { kind: AstNode::Literal { value: Op::Syntax(Syntax::Comma), }, .. })) =>
                        return Err(Error::ListCommaWithoutElement),
                    (Some(State::ListBegin), Some(AstNodeH { kind: AstNode::Literal { value: Op::Syntax(Syntax::RightParen), }, ..})) =>
                        maybe_node = Some(AstNodeH::new(AstNode::Literal {
                            value: Op::Const(Const::Fun(Fun::Nil)),
                        })),
                    (Some(State::ListBegin), Some(node)) => {
                        states.push(State::ListPush { element: Rc::new(node), });
                        states.push(State::ListContinue);
                        break;
                    },
                    (Some(State::ListContinue), None) =>
                        return Err(Error::ListNotClosed),
                    (Some(State::ListContinue), Some(AstNodeH { kind: AstNode::Literal { value: Op::Syntax(Syntax::Comma), }, ..})) => {
                        states.push(State::ListContinueComma);
                        break;
                    },
                    (Some(State::ListContinue), Some(AstNodeH { kind: AstNode::Literal { value: Op::Syntax(Syntax::RightParen), }, .. })) =>
                        maybe_node = Some(AstNodeH::new(AstNode::Literal {
                            value: Op::Const(Const::Fun(Fun::Nil)),
                        })),
                    (Some(State::ListContinue), Some(node)) =>
                        return Err(Error::ListSyntaxUnexpectedNode { node: Rc::new(node).render(), }),
                    (Some(State::ListContinueComma), None) =>
                        return Err(Error::ListNotClosed),
                    (Some(State::ListContinueComma), Some(AstNodeH { kind: AstNode::Literal { value: Op::Syntax(Syntax::Comma), }, .. })) =>
                        return Err(Error::ListSyntaxSeveralCommas),
                    (Some(State::ListContinueComma), Some(AstNodeH { kind: AstNode::Literal { value: Op::Syntax(Syntax::RightParen), }, .. })) =>
                        return Err(Error::ListSyntaxClosingAfterComma),
                    (Some(State::ListContinueComma), Some(node)) => {
                        states.push(State::ListPush { element: Rc::new(node), });
                        states.push(State::ListContinue);
                        break;
                    },
                    (Some(State::ListPush { .. }), None) =>
                        unreachable!(),
                    (Some(State::ListPush { element, }), Some(tail)) =>
                        maybe_node = Some(AstNodeH::new(AstNode::App {
                            fun: Rc::new(AstNodeH::new(AstNode::App {
                                fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cons)), })),
                                arg: element,
                            })),
                            arg: Rc::new(tail),
                        })),
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

    fn eval_tree(&self, ast_node: Rc<AstNodeH>, env: &Env, cache: &mut Cache) -> Result<Ops, Error> {
        let ast_node = self.eval_tree_ast(ast_node, env, cache)?;
        Ok(ast_node.render())
    }

    fn eval_tree_ast(&self, mut ast_node: Rc<AstNodeH>, env: &Env, cache: &mut Cache) -> Result<Rc<AstNodeH>, Error> {

        enum State {
            EvalAppFun { arg: Rc<AstNodeH>, },
            EvalAppArgNum { fun: EvalFunNum, },
            EvalAppArgIsNil,
            EvalAppArgCar,
            EvalAppArgCdr,
        }

        struct StackFrame {
            root: Rc<AstNodeH>,
            state: State,
        }

        let mut states = vec![];
        loop {
            if let Some(memo_ast) = cache.get(&ast_node) {
                ast_node = memo_ast;
                continue;
            }

            let mut eval_op = match &*ast_node {
                AstNodeH { kind: AstNode::Literal { value, }, .. } =>
                    EvalOp::new(value.clone()),

                AstNodeH { kind: AstNode::App { fun, arg, }, .. } => {
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
                                        ast_node = subst_ast_node.clone();
                                        break;
                                    },
                                    None =>
                                        return Ok(top_ast_node),
                                }
                            },

                            eval_op =>
                                return Ok(eval_op.render_ast()),
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
                        ast_node = Rc::new(AstNodeH::new(AstNode::App {
                            fun: Rc::new(AstNodeH::new(AstNode::App {
                                fun: x,
                                arg: arg,
                            })),
                            arg: y,
                        }));
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
                        ast_node = Rc::new(AstNodeH::new(AstNode::App {
                            fun: x,
                            arg: Rc::new(AstNodeH::new(AstNode::App {
                                fun: y,
                                arg: arg,
                            })),
                        }));
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
                        ast_node = Rc::new(AstNodeH::new(AstNode::App {
                            fun: Rc::new(AstNodeH::new(AstNode::App {
                                fun: x,
                                arg: arg.clone(),
                            })),
                            arg: Rc::new(AstNodeH::new(AstNode::App {
                                fun: y,
                                arg: arg,
                            })),
                        }));
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
                        ast_node = Rc::new(AstNodeH::new(AstNode::App {
                            fun: Rc::new(AstNodeH::new(AstNode::App {
                                fun: arg,
                                arg: x,
                            })),
                            arg: y,
                        }));
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // Car0 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Car0))) => {
                        states.push(StackFrame { root, state: State::EvalAppArgCar, });
                        ast_node = arg;
                        break;
                    },

                    // Cdr0 on a something
                    (State::EvalAppFun { arg, }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Cdr0))) => {
                        states.push(StackFrame { root, state: State::EvalAppArgCdr, });
                        ast_node = arg;
                        break;
                    },

                    // Nil0 on a something
                    (State::EvalAppFun { .. }, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Nil0))) => {
                        ast_node = Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::True)), }));
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
                        ast_node = Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::True)), }));
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // IsNil on another fun
                    (State::EvalAppArgIsNil, EvalOp::Fun(..)) => {
                        ast_node = Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::False)), }));
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // IsNil on an abstract
                    (State::EvalAppArgIsNil, EvalOp::Abs(arg_ast_node)) =>
                        match env.lookup_ast(&arg_ast_node) {
                            Some(subst_ast_node) => {
                                states.push(StackFrame { root, state: State::EvalAppArgIsNil, });
                                ast_node = subst_ast_node.clone();
                                break;
                            },
                            None =>
                                eval_op = EvalOp::Abs(Rc::new(AstNodeH::new(AstNode::App {
                                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::IsNil)), })),
                                    arg: arg_ast_node,
                                }))),
                        },

                    // IsNil on an modulated bits
                    (State::EvalAppArgIsNil, EvalOp::Mod { bits, }) => {
                        ast_node = Rc::new(AstNodeH::new(AstNode::Literal {
                            value: Op::Const(Const::Fun(
                                match encoder::ConsList::demodulate_from_string(&bits) {
                                    Ok(encoder::ConsList::Nil) =>
                                        Fun::True,
                                    _ =>
                                        Fun::False,
                                }
                            )),
                        }));
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // Car0 on a number
                    (State::EvalAppArgCar, EvalOp::Num { number, }) =>
                        return Err(Error::ApplyingCarToLiteral { value: Op::Const(Const::EncodedNumber(number)), }),

                    // Car0 on a Cons2
                    (State::EvalAppArgCar, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Cons2 { x, .. }))) => {
                        ast_node = x;
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // Car0 on another fun
                    (State::EvalAppArgCar, EvalOp::Fun(fun)) =>
                        return Err(Error::ApplyingCarToInvalidFun { fun: EvalOp::Fun(fun).render_ast().render(), }),

                    // Car0 on an abstract
                    (State::EvalAppArgCar, EvalOp::Abs(arg_ast_node)) =>
                        match env.lookup_ast(&arg_ast_node) {
                            Some(subst_ast_node) => {
                                states.push(StackFrame { root, state: State::EvalAppArgCar, });
                                ast_node = subst_ast_node.clone();
                                break;
                            },
                            None =>
                                eval_op = EvalOp::Abs(Rc::new(AstNodeH::new(AstNode::App {
                                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Car)), })),
                                    arg: arg_ast_node,
                                }))),
                        },

                    // Car0 on an modulated bits
                    (State::EvalAppArgCar, EvalOp::Mod { bits, }) => {
                        ast_node = Rc::new(AstNodeH::new(match encoder::ConsList::demodulate_from_string(&bits) {
                            Ok(encoder::ConsList::Nil) =>
                                return Err(Error::ApplyingCarToLiteral { value: Op::Const(Const::Fun(Fun::Nil)), }),
                            Ok(encoder::ConsList::Cons(encoder::ListVal::Number(number), _)) =>
                                AstNode::Literal { value: Op::Const(Const::EncodedNumber(number)), },
                            Ok(encoder::ConsList::Cons(encoder::ListVal::Cons(car), _)) =>
                                AstNode::Literal { value: Op::Const(Const::ModulatedBits(car.modulate_to_string())), },
                            Err(error) =>
                                return Err(Error::ConsListDem(error)),
                        }));
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // Cdr0 on a number
                    (State::EvalAppArgCdr, EvalOp::Num { number, }) =>
                        return Err(Error::ApplyingCdrToLiteral { value: Op::Const(Const::EncodedNumber(number)), }),

                    // Cdr0 on a Cons2
                    (State::EvalAppArgCdr, EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Cons2 { y, .. }))) => {
                        ast_node = y;
                        cache.memo(root, ast_node.clone());
                        break;
                    },

                    // Cdr0 on another fun
                    (State::EvalAppArgCdr, EvalOp::Fun(fun)) =>
                        return Err(Error::ApplyingCdrToInvalidFun { fun: EvalOp::Fun(fun).render_ast().render(), }),

                    // Cdr0 on an abstract
                    (State::EvalAppArgCdr, EvalOp::Abs(arg_ast_node)) =>
                        match env.lookup_ast(&arg_ast_node) {
                            Some(subst_ast_node) => {
                                states.push(StackFrame { root, state: State::EvalAppArgCdr, });
                                ast_node = subst_ast_node.clone();
                                break;
                            },
                            None =>
                                eval_op = EvalOp::Abs(Rc::new(AstNodeH::new(AstNode::App {
                                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cdr)), })),
                                    arg: arg_ast_node,
                                }))),
                        },

                    // Cdr0 on an modulated bits
                    (State::EvalAppArgCdr, EvalOp::Mod { bits, }) => {
                        ast_node = Rc::new(AstNodeH::new(match encoder::ConsList::demodulate_from_string(&bits) {
                            Ok(encoder::ConsList::Nil) =>
                                return Err(Error::ApplyingCdrToLiteral { value: Op::Const(Const::Fun(Fun::Nil)), }),
                            Ok(encoder::ConsList::Cons(_, encoder::ListVal::Number(number))) =>
                                AstNode::Literal { value: Op::Const(Const::EncodedNumber(number)), },
                            Ok(encoder::ConsList::Cons(_, encoder::ListVal::Cons(cdr))) =>
                                AstNode::Literal { value: Op::Const(Const::ModulatedBits(cdr.modulate_to_string())), },
                            Err(error) =>
                                return Err(Error::ConsListDem(error)),
                        }));
                        cache.memo(root, ast_node.clone());
                        break;
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
                        ast_node = Rc::new(AstNodeH::new(AstNode::Literal {
                            value: Op::Const(Const::Picture(self.eval_draw(arg, env, cache)?)),
                        }));
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
                                ast_node = Rc::new(AstNodeH::new(AstNode::App {
                                    fun: subst_ast_node.clone(),
                                    arg: arg_ast_node,
                                }));
                                break;
                            }
                            None =>
                                eval_op = EvalOp::Abs(Rc::new(AstNodeH::new(AstNode::App {
                                    fun: fun_ast_node,
                                    arg: arg_ast_node,
                                }))),
                        },

                    // modulated bits on something
                    (State::EvalAppFun { arg, }, EvalOp::Mod { bits }) =>
                        match encoder::ConsList::demodulate_from_string(&bits) {
                            Ok(encoder::ConsList::Nil) =>
                                eval_op = EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::True0)),
                            Ok(encoder::ConsList::Cons(car, cdr)) => {
                                fn to_op(val: encoder::ListVal) -> Op {
                                    match val {
                                        encoder::ListVal::Number(number) =>
                                            Op::Const(Const::EncodedNumber(number)),
                                        encoder::ListVal::Cons(cell) =>
                                            Op::Const(Const::ModulatedBits(cell.modulate_to_string())),
                                    }
                                }
                                ast_node = Rc::new(AstNodeH::new(AstNode::App {
                                    fun: Rc::new(AstNodeH::new(AstNode::App {
                                        fun: arg,
                                        arg: Rc::new(AstNodeH::new(AstNode::Literal { value: to_op(car), })),
                                    })),
                                    arg: Rc::new(AstNodeH::new(AstNode::Literal { value: to_op(cdr), })),
                                }));
                                break;
                            },
                            Err(error) =>
                                return Err(Error::ConsListDem(error)),
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
                    (State::EvalAppArgNum { fun }, EvalOp::Fun(arg_fun)) => {
                        return Err(Error::AppExpectsNumButFunProvided {
                            fun: EvalOp::Fun(EvalFun::ArgNum(fun)).render_ast().render(),
                            arg: EvalOp::Fun(arg_fun).render_ast().render(),
                        });
                    },

                    // fun on abs
                    (State::EvalAppArgNum { fun }, EvalOp::Abs(arg_ast_node)) =>
                        match env.lookup_ast(&arg_ast_node) {
                            Some(subst_ast_node) => {
                                states.push(StackFrame { root, state: State::EvalAppArgNum { fun, }, });
                                ast_node = subst_ast_node.clone();
                                break;
                            },
                            None => {
                                let ast_node = Rc::new(AstNodeH::new(AstNode::App {
                                    fun: EvalOp::Fun(EvalFun::ArgNum(fun)).render_ast(),
                                    arg: arg_ast_node,
                                }));
                                eval_op = EvalOp::Abs(ast_node);
                            },
                        },

                    // fun on mod
                    (State::EvalAppArgNum { fun }, EvalOp::Mod { .. }) =>
                        return Err(Error::ApplyingFunOnModulatedBits { fun: EvalOp::Fun(EvalFun::ArgNum(fun)).render_ast().render(), }),

                }

                let maybe_cache = match &eval_op {
                    EvalOp::Num { ref number, } =>
                        Some(Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::EncodedNumber(number.clone())), }))),
                    EvalOp::Abs(ref ast_node) =>
                        Some(ast_node.clone()),
                    EvalOp::Fun(..) | EvalOp::Mod { .. } =>
                        None,
                };
                if let Some(value) = maybe_cache {
                    cache.memo(root, value);
                }
            }
        }
    }

    pub fn eval_force_list(&self, list_ops: Ops, env: &Env, cache: &mut Cache) -> Result<Ops, Error> {
        match self.build_tree(list_ops)? {
            Ast::Empty =>
                Ok(Ops(vec![])),
            Ast::Tree(ast_node) =>
                Ok(self.eval_force_list_ast(ast_node, env, cache)?.render()),
        }
    }

    fn eval_force_list_ast(&self, list_ast: Rc<AstNodeH>, env: &Env, cache: &mut Cache) -> Result<Rc<AstNodeH>, Error> {
        let force_list_val = self.eval_ast_to_list_val(list_ast, env, cache)?;
        let force_cons_list = match force_list_val {
            encoder::ListVal::Number(number) =>
                return Err(Error::ExpectedListArgForModButGotNumber { number, }),
            encoder::ListVal::Cons(value) =>
                *value,
        };
        let bits = force_cons_list.modulate_to_string();
        Ok(Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::ModulatedBits(bits)), })))
    }

    fn eval_draw(&self, mut points_ast: Rc<AstNodeH>, env: &Env, cache: &mut Cache) -> Result<Picture, Error> {
        let mut points_vec = Vec::new();
        loop {
            let ast_node = self.eval_ast_on(self.eval_isnil(), points_ast.clone(), env, cache)?;
            if let AstNode::Literal { value: Op::Const(Const::Fun(Fun::True)), } = &ast_node.kind {
                break;
            }

            let coord_ast = self.eval_ast_on(self.eval_car(), points_ast.clone(), env, cache)?;

            let ast_node = self.eval_ast_on(self.eval_car(), coord_ast.clone(), env, cache)?;
            let x = if let AstNode::Literal { value: Op::Const(Const::EncodedNumber(number)), } = &ast_node.kind {
                number.clone()
            } else {
                return Err(Error::InvalidCoordForDrawArg);
            };

            let ast_node = self.eval_ast_on(self.eval_cdr(), coord_ast.clone(), env, cache)?;
            let y = if let AstNode::Literal { value: Op::Const(Const::EncodedNumber(number)), } = &ast_node.kind {
                number.clone()
            } else {
                return Err(Error::InvalidCoordForDrawArg);
            };

            points_vec.push(Coord { x, y, });

            points_ast = self.eval_ast_on(self.eval_cdr(), points_ast, env, cache)?;
        }
        Ok(Picture { points: points_vec, })
    }

    fn eval_multiple_draw(&self, mut points_list_of_lists: Rc<AstNodeH>, env: &Env, cache: &mut Cache) -> Result<Rc<AstNodeH>, Error> {
        let mut asts = vec![];
        loop {
            let ast_node = self.eval_ast_on(self.eval_isnil(), points_list_of_lists.clone(), env, cache)?;
            if let AstNode::Literal { value: Op::Const(Const::Fun(Fun::True)), } = &ast_node.kind {
                break;
            }

            let ast_node = self.eval_ast_on(self.eval_car(), points_list_of_lists.clone(), env, cache)?;
            asts.push(Rc::new(AstNodeH::new(AstNode::App {
                fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Draw)), })),
                arg: ast_node,
            })));

            points_list_of_lists = self.eval_ast_on(self.eval_cdr(), points_list_of_lists, env, cache)?;
        }

        let mut ast_node = Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Nil)), }));
        while let Some(cell_ast) = asts.pop() {
            ast_node = Rc::new(AstNodeH::new(AstNode::App {
                fun: Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cons)), })),
                    arg: cell_ast,
                })),
                arg: ast_node,
            }));
        }
        Ok(ast_node)
    }

    fn eval_send(&self, send_ast: Rc<AstNodeH>, env: &Env, cache: &mut Cache) -> Result<Rc<AstNodeH>, Error> {
        match &self.eval_force_list_ast(send_ast, env, cache)?.kind {
            AstNode::Literal { value: Op::Const(Const::ModulatedBits(bits)), } => {
                let send_mod = bits.clone();
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

                Ok(Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::ModulatedBits(recv_mod)), })))
            },
            _ =>
                unreachable!(),
        }
    }

    fn eval_render(&self, mut render_ast: Rc<AstNodeH>, env: &Env, cache: &mut Cache) -> Result<Rc<AstNodeH>, Error> {
        loop {
            let ast_node = self.eval_ast_on(self.eval_isnil(), render_ast.clone(), env, cache)?;
            if let AstNode::Literal { value: Op::Const(Const::Fun(Fun::True)), } = &ast_node.kind {
                break;
            }

            let ast_node = self.eval_ast_on(self.eval_car(), render_ast.clone(), env, cache)?;
            match &ast_node.kind {
                AstNode::Literal { value: Op::Const(Const::Picture(picture)), } => {
                    // perform send
                    if let Some(outer_channel) = &self.outer_channel {
                        let outer_send_result = outer_channel.unbounded_send(OuterRequest::RenderPictures {
                            pictures: vec![picture.clone()],
                        });
                        if let Err(..) = outer_send_result {
                            return Err(Error::OuterChannelIsClosed);
                        }
                    } else {
                        return Err(Error::RenderOpIsNotSupportedWithoutOuterChannel);
                    };
                },
                _ =>
                    return Err(Error::RenderItemIsNotAPicture { ops: ast_node.render(), }),
            }

            render_ast = self.eval_ast_on(self.eval_cdr(), render_ast, env, cache)?;
            break;
        }

        Ok(render_ast)
    }

    fn eval_mod(&self, ast_node: Rc<AstNodeH>, env: &Env, cache: &mut Cache) -> Result<Rc<AstNodeH>, Error> {
        self.eval_num_list_map(ast_node, &|num| match num {
            EncodedNumber { number, modulation: Modulation::Demodulated, } =>
                Ok(EncodedNumber { number: number.clone(), modulation: Modulation::Modulated, }),
            number @ EncodedNumber { modulation: Modulation::Modulated, .. } =>
                Err(Error::ModOnModulatedNumber { number: number.clone(), }),
        }, env, cache)
    }

    fn eval_dem(&self, ast_node: Rc<AstNodeH>, env: &Env, cache: &mut Cache) -> Result<Rc<AstNodeH>, Error> {
        self.eval_num_list_map(ast_node, &|num| match num {
            EncodedNumber { number, modulation: Modulation::Modulated, } =>
                Ok(EncodedNumber { number: number.clone(), modulation: Modulation::Demodulated, }),
            number @ EncodedNumber { modulation: Modulation::Demodulated, .. } =>
                Err(Error::DemOnDemodulatedNumber { number: number.clone(), }),
        }, env, cache)
    }

    fn eval_modem(&self, ast_node: Rc<AstNodeH>, env: &Env, cache: &mut Cache) -> Result<Rc<AstNodeH>, Error> {
        let ast_node = self.eval_mod(ast_node, env, cache)?;
        let ast_node = self.eval_dem(ast_node, env, cache)?;
        Ok(ast_node)
    }

    fn eval_num_list_map<F>(&self, list_ast: Rc<AstNodeH>, trans: &F, env: &Env, cache: &mut Cache) -> Result<Rc<AstNodeH>, Error>
    where F: Fn(&EncodedNumber) -> Result<EncodedNumber, Error>
    {
        match &list_ast.kind {
            AstNode::Literal { value: Op::Const(Const::Fun(Fun::Nil)), } =>
                Ok(list_ast),
            AstNode::Literal { value: Op::Const(Const::EncodedNumber(number)), } => {
                let transformed = trans(number)?;
                Ok(Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::EncodedNumber(transformed)), })))
            },
            _ => {
                let ast_node = self.eval_ast_on(self.eval_car(), list_ast.clone(), env, cache)?;
                let car_ast = self.eval_num_list_map(ast_node, trans, env, cache)?;
                let ast_node = self.eval_ast_on(self.eval_cdr(), list_ast.clone(), env, cache)?;
                let cdr_ast = self.eval_num_list_map(ast_node, trans, env, cache)?;
                Ok(Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::App {
                        fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cons)), })),
                        arg: car_ast,
                    })),
                    arg: cdr_ast,
                })))
            },
        }
    }

    fn eval_ast_to_list_val(&self, mut list_ast: Rc<AstNodeH>, env: &Env, cache: &mut Cache) -> Result<encoder::ListVal, Error> {
        match &list_ast.kind {
            AstNode::Literal { value: Op::Const(Const::EncodedNumber(number)), } =>
                return Ok(encoder::ListVal::Number(number.clone())),
            _ =>
                (),
        }

        let mut cons_stack = vec![];
        loop {
            let ast_node = self.eval_ast_on(self.eval_isnil(), list_ast.clone(), env, cache)?;
            if let AstNode::Literal { value: Op::Const(Const::Fun(Fun::True)), } = &ast_node.kind {
                break;
            }

            let ast_node = self.eval_ast_on(self.eval_car(), list_ast.clone(), env, cache)?;
            let child_list_val = self.eval_ast_to_list_val(ast_node, env, cache)?;
            cons_stack.push(child_list_val);

            list_ast = self.eval_ast_on(self.eval_cdr(), list_ast, env, cache)?;
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

    fn eval_ast_on(&self, fun: Rc<AstNodeH>, on_script: Rc<AstNodeH>, env: &Env, cache: &mut Cache) -> Result<Rc<AstNodeH>, Error> {
        let app_ast = Rc::new(AstNodeH::new(AstNode::App { fun, arg: on_script, }));
        self.eval_tree_ast(app_ast, env, cache)
    }

    fn eval_isnil(&self) -> Rc<AstNodeH> {
        Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::IsNil)), }))
    }

    fn eval_car(&self) -> Rc<AstNodeH> {
        Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Car)), }))
    }

    fn eval_cdr(&self) -> Rc<AstNodeH> {
        Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cdr)), }))
    }

    fn eval_interact(&self, protocol: Rc<AstNodeH>, state: Rc<AstNodeH>, vector: Rc<AstNodeH>, _env: &Env) -> Result<Rc<AstNodeH>, Error> {
        Ok(Rc::new(AstNodeH::new(AstNode::App {
            fun: Rc::new(AstNodeH::new(AstNode::App {
                fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::F38)), })),
                arg: protocol.clone(),
            })),
            arg: Rc::new(AstNodeH::new(AstNode::App {
                fun: Rc::new(AstNodeH::new(AstNode::App {
                    fun: protocol,
                    arg: state,
                })),
                arg: vector,
            })),
        })))
    }

    fn eval_f38(&self, protocol: Rc<AstNodeH>, tuple3: Rc<AstNodeH>, _env: &Env) -> Result<Rc<AstNodeH>, Error> {
        Ok(Rc::new(AstNodeH::new(AstNode::App {
            fun: Rc::new(AstNodeH::new(AstNode::App {
                fun: Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::If0)), })),
                    arg: Rc::new(AstNodeH::new(AstNode::App {
                        fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Car)), })),
                        arg: tuple3.clone(),
                    })),
                })),
                arg: Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::App {
                        fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cons)), })),
                        arg: Rc::new(AstNodeH::new(AstNode::App {
                            fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Modem)), })),
                            arg: Rc::new(AstNodeH::new(AstNode::App {
                                fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Car)), })),
                                arg: Rc::new(AstNodeH::new(AstNode::App {
                                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cdr)), })),
                                    arg: tuple3.clone(),
                                })),
                            })),
                        })),
                    })),
                    arg: Rc::new(AstNodeH::new(AstNode::App {
                        fun: Rc::new(AstNodeH::new(AstNode::App {
                            fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cons)), })),
                            arg: Rc::new(AstNodeH::new(AstNode::App {
                                fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::MultipleDraw)), })),
                                arg: Rc::new(AstNodeH::new(AstNode::App {
                                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Car)), })),
                                    arg: Rc::new(AstNodeH::new(AstNode::App {
                                        fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cdr)), })),
                                        arg: Rc::new(AstNodeH::new(AstNode::App {
                                            fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cdr)), })),
                                            arg: tuple3.clone(),
                                        })),
                                    })),
                                })),
                            })),
                        })),
                        arg: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Nil)), })),
                    })),
                })),
            })),
            arg: Rc::new(AstNodeH::new(AstNode::App {
                fun: Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::App {
                        fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Interact)), })),
                        arg: protocol,
                    })),
                    arg: Rc::new(AstNodeH::new(AstNode::App {
                        fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Modem)), })),
                        arg: Rc::new(AstNodeH::new(AstNode::App {
                            fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Car)), })),
                            arg: Rc::new(AstNodeH::new(AstNode::App {
                                fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cdr)), })),
                                arg: tuple3.clone(),
                            })),
                        })),
                    })),
                })),
                arg: Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Send)), })),
                    arg: Rc::new(AstNodeH::new(AstNode::App {
                        fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Car)), })),
                        arg: Rc::new(AstNodeH::new(AstNode::App {
                            fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cdr)), })),
                            arg: Rc::new(AstNodeH::new(AstNode::App {
                                fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cdr)), })),
                                arg: tuple3,
                            })),
                        })),
                    })),
                })),
            })),
        })))
    }
}

#[cfg(test)]
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
    Mod { bits: String, },
    Fun(EvalFun),
    Abs(Rc<AstNodeH>),
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
    True1 { captured: Rc<AstNodeH>, },
    False0,
    False1 { captured: Rc<AstNodeH>, },
    I0,
    C0,
    C1 { x: Rc<AstNodeH>, },
    C2 { x: Rc<AstNodeH>, y: Rc<AstNodeH>, },
    B0,
    B1 { x: Rc<AstNodeH>, },
    B2 { x: Rc<AstNodeH>, y: Rc<AstNodeH>, },
    S0,
    S1 { x: Rc<AstNodeH>, },
    S2 { x: Rc<AstNodeH>, y: Rc<AstNodeH>, },
    Cons0,
    Cons1 { x: Rc<AstNodeH>, },
    Cons2 { x: Rc<AstNodeH>, y: Rc<AstNodeH>, },
    Car0,
    Cdr0,
    Nil0,
    IsNil0,
    IfZero1 { cond: EncodedNumber, },
    IfZero2 { cond: EncodedNumber, true_clause: Rc<AstNodeH>, },
    Draw0,
    MultipleDraw0,
    Send0,
    Mod0,
    Dem0,
    Modem0,
    Interact0,
    Interact1 { protocol: Rc<AstNodeH>, },
    Interact2 { protocol: Rc<AstNodeH>, state: Rc<AstNodeH>, },
    F38_0,
    F38_1 { protocol: Rc<AstNodeH>, },
    Render0,
}

impl EvalOp {
    fn new(op: Op) -> EvalOp {
        match op {
            Op::Const(Const::EncodedNumber(number)) =>
                EvalOp::Num { number, },
            Op::Const(Const::ModulatedBits(bits)) =>
                EvalOp::Mod { bits, },
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
                EvalOp::Abs(Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Picture(picture)), }))),
            Op::Variable(var) =>
                EvalOp::Abs(Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Variable(var), }))),
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

    fn render_ast(self) -> Rc<AstNodeH> {
        match self {
            EvalOp::Num { number, } =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::EncodedNumber(number)), })),
            EvalOp::Mod { bits, } =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::ModulatedBits(bits)), })),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Inc0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Inc)), })),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Dec0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Dec)), })),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Sum0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Sum)), })),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Sum1 { captured, })) =>
                Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Sum)), })),
                    arg: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::EncodedNumber(captured)), })),
                })),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Mul0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Mul)), })),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Mul1 { captured, })) =>
                Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Mul)), })),
                    arg: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::EncodedNumber(captured)), })),
                })),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Div0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Div)), })),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Div1 { captured, })) =>
                Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Div)), })),
                    arg: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::EncodedNumber(captured)), })),
                })),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Eq0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Eq)), })),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Eq1 { captured, })) =>
                Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Eq)), })),
                    arg: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::EncodedNumber(captured)), })),
                })),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Lt0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Lt)), })),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Lt1 { captured, })) =>
                Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Lt)), })),
                    arg: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::EncodedNumber(captured)), })),
                })),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::Neg0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Neg)), })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::True0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::True)), })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::True1 { captured, })) =>
                Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::True)), })),
                    arg: captured,
                })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::False0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::False)), })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::False1 { captured, })) =>
                Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::False)), })),
                    arg: captured,
                })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::I0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::I)), })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::C0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::C)), })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::C1 { x, })) =>
                Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::C)), })),
                    arg: x,
                })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::C2 { x, y, })) =>
                Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::App {
                        fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::C)), })),
                        arg: x,
                    })),
                    arg: y,
                })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::B0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::B)), })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::B1 { x, })) =>
                Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::B)), })),
                    arg: x,
                })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::B2 { x, y, })) =>
                Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::App {
                        fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::B)), })),
                        arg: x,
                    })),
                    arg: y,
                })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::S0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::S)), })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::S1 { x, })) =>
                Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::S)), })),
                    arg: x,
                })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::S2 { x, y, })) =>
                Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::App {
                        fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::S)), })),
                        arg: x,
                    })),
                    arg: y,
                })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Cons0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cons)), })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Cons1 { x, })) =>
                Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cons)), })),
                    arg: x,
                })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Cons2 { x, y, })) =>
                Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::App {
                        fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cons)), })),
                        arg: x,
                    })),
                    arg: y,
                })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Car0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Car)), })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Cdr0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Cdr)), })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Nil0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Nil)), })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::IsNil0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::IsNil)), })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Draw0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Draw)), })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::MultipleDraw0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::MultipleDraw)), })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Send0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Send)), })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Mod0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Mod)), })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Dem0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Dem)), })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Modem0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Modem)), })),
            EvalOp::Fun(EvalFun::ArgNum(EvalFunNum::IfZero0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::If0)), })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::IfZero1 { cond, })) =>
                Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::If0)), })),
                    arg: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::EncodedNumber(cond)), })),
                })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::IfZero2 { cond, true_clause, })) =>
                Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::App {
                        fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::If0)), })),
                        arg: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::EncodedNumber(cond)), })),
                    })),
                    arg: true_clause,
                })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Interact0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Interact)), })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Interact1 { protocol, })) =>
                Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Interact)), })),
                    arg: protocol,
                })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Interact2 { protocol, state, })) =>
                Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::App {
                        fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Interact)), })),
                        arg: protocol,
                    })),
                    arg: state,
                })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::F38_0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::F38)), })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::F38_1 { protocol, })) =>
                Rc::new(AstNodeH::new(AstNode::App {
                    fun: Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::F38)), })),
                    arg: protocol,
                })),
            EvalOp::Fun(EvalFun::ArgAbs(EvalFunAbs::Render0)) =>
                Rc::new(AstNodeH::new(AstNode::Literal { value: Op::Const(Const::Fun(Fun::Render)), })),

            EvalOp::Abs(ast_node) =>
                ast_node,
        }
    }
}

impl AstNodeH {
    pub fn render(self: Rc<AstNodeH>) -> Ops {
        enum State {
            RenderAppFun { arg: Rc<AstNodeH>, },
            RenderAppArg,
        }

        let mut ops = vec![];
        let mut ast_node = self;
        let mut stack = vec![];
        loop {
            match &ast_node.kind {
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

impl Ast {
    pub fn render(self: Ast) -> Ops {
        match self {
            Ast::Empty =>
                Ops(vec![]),
            Ast::Tree(ast_node) =>
                ast_node.render(),
        }
    }

    #[cfg(test)]
    fn take_tree(self) -> Rc<AstNodeH> {
        match self {
            Ast::Empty =>
                panic!("should not be here"),
            Ast::Tree(ast_node) =>
                ast_node,
        }
    }
}
