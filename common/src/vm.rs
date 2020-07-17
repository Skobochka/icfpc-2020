
use super::{
    code::{
        Op,
        Ops,
    },
};

pub struct Vm {
    inner: VmInner,
}

impl Vm {
    pub fn new() -> Vm {
        Vm {
            inner: VmInner {

            },
        }
    }

    pub fn start(self) -> VmNext {
        VmNext::NeedOp(NeedOp { inner: self.inner, })
    }
}

pub enum VmNext {
    Ready(Ready),
    NeedOp(NeedOp),
}

pub struct Ready {
    inner: VmInner,
}

pub struct ReadyNext {
    inner: VmInner,
}

impl ReadyNext {
    pub fn proceed(self) -> VmNext {
        VmNext::NeedOp(NeedOp { inner: self.inner, })
    }
}

pub struct NeedOp {
    inner: VmInner,
}

impl NeedOp {
    pub fn input_op(self, op: Op) -> VmNext {
    }
}

struct VmInner {
    stack: Vec<OpState>,
}

enum OpState {

}
