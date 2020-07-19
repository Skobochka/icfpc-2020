
use crate::map::Obstacles;

type Axis = usize;

#[derive(Debug,Clone,Copy)]
enum Direction {
    Left,
    Right,
}

#[derive(Debug,Clone,Copy)]
struct TreePath {
    len: usize,
    nodes: u64,
}
impl TreePath {
    fn left(&self) -> TreePath {
        let mut tp = *self;
        tp.len += 1;
        tp
    }
    fn right(&self) -> TreePath {
        let mut tp = *self;
        tp.nodes |= 0x1 << tp.len;
        tp.len += 1;
        tp
    }
}
impl Iterator for TreePath {
    type Item = Direction;
    fn next(&mut self) -> Option<Self::Item> {
        match self.len {
            0 => None,
            _ => {
                let c = self.nodes % 2;
                self.nodes /= 2;
                self.len -= 1;
                Some(match c {
                    0 => Direction::Left,
                    _ => Direction::Right,
                })
            },
        }
    }
}

    
#[derive(Debug,Clone,Copy)]
struct MBR([f64; 4]); // Xmin Ymin Xmax Ymax

#[derive(Debug)]
enum Medians {
    Uniformed(MBR)
}
impl Medians {
    const DIMENSIONS: usize = 2;
    fn get_median(&self, tp: TreePath) -> (Axis,f64) {
        match self {
            Medians::Uniformed(mbr) => {
                if tp.len > 64 { panic!("Too deep tree path"); }
                let axis = tp.len as usize % Self::DIMENSIONS;
                let mut mbr = mbr.0;
                let mut ax = 0;
                for dir in tp {
                    match dir {
                        Direction::Left => mbr[Self::DIMENSIONS + ax] = (mbr[ax] + mbr[Self::DIMENSIONS + ax])/2.0,
                        Direction::Right => mbr[ax] = (mbr[ax] + mbr[Self::DIMENSIONS + ax])/2.0,
                    }
                    ax += 1;
                    ax %= Self::DIMENSIONS;
                }
                (axis,(mbr[axis] + mbr[Self::DIMENSIONS + axis])/2.0)
            }
        }
    }
    fn max_l2_2(&self) -> f64 {
        match self {
            Medians::Uniformed(mbr) => {
                let mut s = 0.0;
                for i in 0 .. Self::DIMENSIONS {
                    s += (mbr.0[i] - mbr.0[Self::DIMENSIONS + i]).powi(2);
                }
                s
            }
        }
    }
}



enum KDNode {
    Leaf(Vec<(u64,[f64; Self::DIMENSIONS])>),
    Node {
        axis: Axis,
        value: f64,
        children: Vec<KDNode>,
    }
}
impl KDNode {
    const DIMENSIONS: usize = 2;
    
    fn get_mut_leaf(&mut self, p: &[f64; Self::DIMENSIONS], tp: &mut TreePath) -> &mut Self {
        match self {
            KDNode::Leaf(..) => self,
            KDNode::Node{ axis,value, children } => {
                let dir = match p[*axis] <= *value {
                    true => 0,
                    false => {
                        tp.nodes |= 0x1 << tp.len;
                        1
                    },
                };          
                tp.len += 1;
                children[dir].get_mut_leaf(p,tp)
            },
        }
    }

    fn nearest(&self, obstacles: &Obstacles, p: &[f64; KDNode::DIMENSIONS], current_dst2: &mut f64, cnt: &mut usize) -> Option<(u64,[f64; KDNode::DIMENSIONS])> {
        match self {
            KDNode::Leaf(v) => {
                let mut min = None;
                for (id,pn) in v.iter() {
                    if obstacles.intersect(*p,*pn) { continue; }
                    let d2 = l2_norm2(p,pn);
                    if d2 < *current_dst2 {
                        *current_dst2 = d2;
                        min = Some((*id,*pn));
                    }
                }
                *cnt += v.len();
                min
            },
            KDNode::Node{ axis, value, children } => {
                let (d_left, d_right, left): (f64,f64,bool) = match p[*axis] <= *value {
                    true => (0.0,*value - p[*axis],true),
                    false => (p[*axis] - *value,0.0,false),
                };
                let mut min = None;
                if left {                   
                    if d_left.powi(2) < *current_dst2 {
                        if let Some(n) = children[0].nearest(obstacles, p,current_dst2,cnt) {
                            min = Some(n);
                        }
                    }
                    if d_right.powi(2) < *current_dst2 {
                        if let Some(n) = children[1].nearest(obstacles, p,current_dst2,cnt) {
                            min = Some(n);
                        }
                    }
                } else {
                    if d_right.powi(2) < *current_dst2 {
                        if let Some(n) = children[1].nearest(obstacles, p,current_dst2,cnt) {
                            min = Some(n);
                        }
                    }
                    if d_left.powi(2) < *current_dst2 {
                        if let Some(n) = children[0].nearest(obstacles, p,current_dst2,cnt) {
                            min = Some(n);
                        }
                    }                  
                }
                min
            },
        }
    }
    
    fn print(&self, level: usize) {
        let mut s = String::new();
        for i in 0 .. level {
            s += "  ";
        }
        match self {
            KDNode::Leaf(v) => println!("{}LEAF: {:?}",s,v),
            KDNode::Node{ axis,value, children } => {
                println!("{}NODE: {} {}",s,axis,value);
                for c in children.iter() {
                    c.print(level+1);
                }
            }
        }
    }
}


const EPS: f64 = 1.0e-15;
pub fn l1_norm(p1: &[f64; KDNode::DIMENSIONS], p2: &[f64; KDNode::DIMENSIONS]) -> f64 {
    let mut s = 0.0;
    for i in 0 .. KDNode::DIMENSIONS {
        s += (p1[i] - p2[i]).abs()
    }
    s
}
pub fn l2_norm2(p1: &[f64; KDNode::DIMENSIONS], p2: &[f64; KDNode::DIMENSIONS]) -> f64 {
    let mut s = 0.0;
    for i in 0 .. KDNode::DIMENSIONS {
        s += (p1[i] - p2[i]).powi(2)
    }
    s
}


pub struct KDTree {
    root: KDNode,
    medians: Medians,

    len: usize,
}
impl KDTree {
    const NODE_PER_LEAF: usize = 16;

    pub fn new_uniform2(rect: [f64; 4]) -> KDTree {
        KDTree {
            root: KDNode::Leaf(Vec::with_capacity(Self::NODE_PER_LEAF)),
            medians: Medians::Uniformed(MBR(rect)),
            len: 0,
        }
    }
    
    /*pub fn new(medians: Medians) -> KDTree {
        KDTree {
            root: KDNode::Leaf(Vec::with_capacity(Self::NODE_PER_LEAF)),
            medians: medians,
        }
    }*/

    fn new_node(medians: &Medians, mut ps: Vec<(u64,[f64; KDNode::DIMENSIONS])>, tp: TreePath) -> KDNode {
        if ps.len() < Self::NODE_PER_LEAF { return KDNode::Leaf(ps); }
        let (axis,value) = medians.get_median(tp);
        let mut pnts2 = Vec::with_capacity(Self::NODE_PER_LEAF);
        let mut i = 0;
        while i < ps.len() {
            if ps[i].1[axis] > value {
                pnts2.push(ps.swap_remove(i));
            } else {
                i += 1;
            }
        }
        KDNode::Node{ axis, value, children: vec![
            Self::new_node(medians,ps,tp.left()),
            Self::new_node(medians,pnts2,tp.right()),
        ] }
    }

    pub fn insert(&mut self, id: u64, p: [f64; KDNode::DIMENSIONS]) -> bool {
        let mut tp = TreePath { len: 0, nodes: 0 };
        let leaf = self.root.get_mut_leaf(&p, &mut tp);
        let split = match leaf {
            KDNode::Leaf(v) => {
                for pnt in v.iter_mut() {
                    if l1_norm(&pnt.1,&p) < EPS { return false; }
                }
                v.push((id,p));
                v.len() >= Self::NODE_PER_LEAF
            },
            _ => unreachable!(),
        };
        if split {
            let mut tmp = KDNode::Leaf(vec![]);
            std::mem::swap(&mut tmp, leaf);
            let mut pnts = match tmp {
                KDNode::Leaf(v) => v,
                _ => unreachable!(),
            };
            *leaf = Self::new_node(&self.medians,pnts,tp);
        }
        self.len += 1;
        true
    }

    pub fn nearest(&self, obs: &Obstacles, p: &[f64; KDNode::DIMENSIONS]) -> Option<(u64,[f64; KDNode::DIMENSIONS])> {
        let mut d = self.medians.max_l2_2() + 1.0;
        let mut cnt = 0;
        let r = self.root.nearest(obs,p,&mut d, &mut cnt);
        //println!("trace kdtree.nearest: {} of {}",cnt,self.len);
        r
    }

    fn print(&self) {
        self.root.print(0);
    }
}

