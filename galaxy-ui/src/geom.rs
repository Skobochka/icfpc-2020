
const EPS: f64 = 1e-9;

const SQRT2: f64 = 1.41421356;

#[derive(Debug)]
pub enum NotLine{
    Point
}

pub trait Form {
    fn mbr(&self) -> [f64; 4];
    fn contains_point(&self, p: [f64; 2]) -> bool;
    fn intersect_segment(&self, line: &LineExt) -> Intersection;
    fn is_intersecting_segment(&self, line: &LineExt) -> bool;
    fn surrounding(&self, dr: f64, cnt: usize) -> Vec<[f64; 2]>;
}

fn v3_vec_mul(v: [f64; 3], q: [f64; 3]) -> [f64; 3] {
    [ v[1]*q[2] - q[1]*v[2],
      v[2]*q[0] - q[2]*v[0],
      v[0]*q[1] - q[0]*v[1]]
}

fn l2_norm2(p: [f64;2], q: [f64; 2]) -> f64 {
    (p[0]-q[0]).powi(2) + (p[1]-q[1]).powi(2)
}

fn same_point(p: [f64;2], q: [f64; 2]) -> bool {
    ((p[0]-q[0]).abs() < EPS)&&((p[1]-q[1]).abs() < EPS)
}

#[derive(Debug,Clone,Copy)]
pub struct LineExt {
    ps: [f64; 4],
    eq: [f64; 3],
    seq: [f64; 3],
    d: f64,
}
impl std::convert::TryFrom<[f64; 4]> for LineExt {
    type Error = NotLine;
    fn try_from(ps: [f64;4]) -> Result<LineExt,NotLine> {
        LineExt::new(ps)
    }
}
impl LineExt {
    pub fn from_to(p1: [f64; 2], p2: [f64; 2]) -> Result<LineExt,NotLine> {
        LineExt::new([p1[0],p1[1],p2[0],p2[1]])
    }
    pub fn new(ps: [f64; 4]) -> Result<LineExt,NotLine> {
        let dx = ps[2] - ps[0];
        let dy = ps[3] - ps[1];
        let d = (dx.powi(2) + dy.powi(2)).powf(0.5);
        if d < EPS { return Err(NotLine::Point); }
        Ok(LineExt{
            ps: ps,
            eq: [ -dy/d, dx/d, dy*ps[0]/d - dx*ps[1]/d ],
            seq: [ dx/d, dy/d, -dx*ps[0]/d -dy*ps[1]/d ],
            d: d,
        })
            
    }
    pub fn f_xy(&self, p: [f64; 2]) -> f64 {
        self.eq[0] * p[0] + self.eq[1] * p[1] + self.eq[2]
    }
    pub fn s_xy(&self, p: [f64; 2]) -> f64 {
        self.seq[0] * p[0] + self.seq[1] * p[1] + self.seq[2]
    }
    pub fn projection_in_segment(&self, p: [f64; 2]) -> bool {
        (self.s_xy(p)/self.d - 0.5).abs() < (0.5 + EPS) 
    }
    pub fn from(&self) -> [f64; 2] {
        [self.ps[0],self.ps[1]]
    }
    pub fn to(&self) -> [f64; 2] {
        [self.ps[2],self.ps[3]]
    }
    pub fn normal(&self) -> [f64; 2] {
        [self.eq[0],self.eq[1]]
    }
    pub fn vec(&self) -> [f64; 2] {
        [self.seq[0],self.seq[1]]
    }
    pub fn len(&self) -> f64 {
        self.d
    }
}

impl Form for LineExt {
    fn mbr(&self) -> [f64; 4] {
        [f64::min(self.ps[0],self.ps[2]),
         f64::min(self.ps[1],self.ps[3]),
         (self.ps[0] - self.ps[2]).abs(),
         (self.ps[1] - self.ps[3]).abs()]
    }
    fn contains_point(&self, p: [f64; 2]) -> bool {
        match self.f_xy(p).abs() < EPS {
            true => self.projection_in_segment(p),
            false => false,
        }
    }
    fn intersect_segment(&self, line: &LineExt) -> Intersection {
        Intersection::seg_seg(self,line)
    }
    fn is_intersecting_segment(&self, line: &LineExt) -> bool {
        match self.intersect_segment(line) {
            Intersection::None => false,
            _ => true,
        }
    }
    fn surrounding(&self, mut dr: f64, _cnt: usize) -> Vec<[f64; 2]> {
        if dr < EPS {
            dr = self.d * 0.05;            
        } 
        dr = dr / self.d / SQRT2;
        if dr < EPS { dr = EPS * 10.0; }
        
        let dv = [self.seq[0]*dr,self.seq[1]*dr];
        let dn = [self.eq[0]*dr,self.eq[1]*dr];
        vec![
            [self.ps[0]-dv[0]-dn[0],self.ps[1]-dv[1]-dn[1]],
            [self.ps[0]-dv[0]+dn[0],self.ps[1]-dv[1]+dn[1]],
            [self.ps[2]+dv[0]-dn[0],self.ps[3]+dv[1]-dn[1]],
            [self.ps[2]+dv[0]+dn[0],self.ps[3]+dv[1]+dn[1]],
        ]
    }
}

#[derive(Debug)]
pub enum NotRect {
    Line,
    Point,
}

#[derive(Debug,Clone,Copy)]
pub struct RectExt {
    rect: [f64; 4],
    lines: [LineExt; 4],
}
impl RectExt {
    pub fn new(r: [f64; 4]) -> Result<RectExt,NotRect> {
        let x = r[0];
        let y = r[1];
        let w = r[2];
        let h = r[3];
        match (w.abs() < EPS,h.abs() < EPS) {
            (true,true) => return Err(NotRect::Point),
            (true,_) | (_,true) => return Err(NotRect::Line),
            _ => {},
        }
        Ok(RectExt{
            rect: r,
            lines: [
                LineExt::from_to([x,y],[x,y+h]).unwrap(),
                LineExt::from_to([x,y+h],[x+w,y+h]).unwrap(),
                LineExt::from_to([x+w,y+h],[x+w,y]).unwrap(),
                LineExt::from_to([x+w,y],[x,y]).unwrap(),
            ],
        })
    }
    pub fn rect(&self) -> [f64; 4] {
        self.rect
    }
}
impl Form for RectExt {
    fn mbr(&self) -> [f64; 4] {
        self.rect
    }
    fn surrounding(&self, mut dr: f64, _cnt: usize) -> Vec<[f64; 2]> {
        let d = f64::max(self.rect[2],self.rect[3]);
        if dr < EPS { 
            dr = d * 0.05;         
        }
        dr = dr / d / SQRT2;
        if dr < EPS { dr = EPS * 10.0; }
        vec![
            [self.rect[0]-dr,self.rect[1]-dr],
            [self.rect[0]+self.rect[2]+dr,self.rect[1]-dr],
            [self.rect[0]-dr,self.rect[1]+self.rect[3]+dr],
            [self.rect[0]+self.rect[2]+dr,self.rect[1]+self.rect[3]+dr],
        ]
    }
    fn contains_point(&self, p: [f64; 2]) -> bool {
        for ln in &self.lines {
            if ln.f_xy(p) > 0.0 { return false; }
        }
        return true;
    }
    fn is_intersecting_segment(&self, line: &LineExt) -> bool {
        match (self.contains_point(line.from()),self.contains_point(line.to())) {
            (false,false) => {
                for ln in &self.lines {
                    match ln.intersect_segment(line) {
                        Intersection::None => continue,
                        Intersection::Point(_) |
                        Intersection::Segment(_) => return true,
                    }
                }
                false
            },
            _ => true,
        }
    }
    fn intersect_segment(&self, line: &LineExt) -> Intersection {
        match (self.contains_point(line.from()),self.contains_point(line.to())) {
            (true,true) => Intersection::Segment(line.ps),
            (true,_) => {
                let q = line.from();
                for ln in &self.lines {
                    match ln.intersect_segment(line) {
                        Intersection::None => continue,
                        Intersection::Point(p) => return match same_point(q,p) {
                            false => Intersection::Segment([q[0],q[1],p[0],p[1]]),
                            true => Intersection::Point(q),
                        },
                        Intersection::Segment(s) => return Intersection::Segment(s),
                    }
                }
                unreachable!();
            },
            (_,true) => {
                let q = line.to();
                for ln in &self.lines {
                    match ln.intersect_segment(line) {
                        Intersection::None => continue,
                        Intersection::Point(p) => return match same_point(q,p) {
                            false => Intersection::Segment([q[0],q[1],p[0],p[1]]),
                            true => Intersection::Point(q),
                        },
                        Intersection::Segment(s) => return Intersection::Segment(s),
                    }
                }
                unreachable!();
            },
            (false,false) => {
                let mut p1 = None;
                let mut p2 = None;
                for ln in &self.lines {
                    match ln.intersect_segment(line) {
                        Intersection::None => continue,
                        Intersection::Point(p) => {
                            match p1 {
                                None => { p1 = Some(p); },
                                Some(p1) => {
                                    if !same_point(p1,p) {
                                        p2 = Some(p);
                                    }
                                },
                            }
                        },
                        Intersection::Segment(_) => unreachable!(),
                    }
                }
                match (p1,p2) {
                    (None,None) => Intersection::None,
                    (Some(p),None) | (None,Some(p)) => Intersection::Point(p),
                    (Some(q),Some(p)) => Intersection::Segment([q[0],q[1],p[0],p[1]]),
                }
            },
        }
    }
}

#[derive(Debug)]
pub enum NotCircle {
    Point,
}

#[derive(Debug,Clone,Copy)]
pub struct CircleExt {
    pivot: [f64; 2],
    radius: f64,
}
impl CircleExt {
    pub fn new(pivot: [f64; 2], r: f64) -> Result<CircleExt,NotCircle> {
        if r.abs() < EPS { return Err(NotCircle::Point); }
        Ok(CircleExt{
            pivot: pivot,
            radius: r,
        })
    }
}
impl Form for CircleExt {
    fn mbr(&self) -> [f64; 4] {
        [self.pivot[0]-self.radius,self.pivot[1]-self.radius,self.radius*2.0,self.radius*2.0]
    }
    fn contains_point(&self, p: [f64; 2]) -> bool {
        l2_norm2(self.pivot,p) < (self.radius.powi(2) + EPS)
    }
    fn surrounding(&self, mut dr: f64, _cnt: usize) -> Vec<[f64; 2]> {
        if dr < EPS { 
            dr = self.radius * (SQRT2 + 0.02);         
        }
        if dr < EPS { dr = EPS * 10.0; }
        let dr2 = dr / SQRT2;
        vec![
            [self.pivot[0]+dr,self.pivot[1]],
            [self.pivot[0]-dr,self.pivot[1]],
            [self.pivot[0],self.pivot[1]+dr],
            [self.pivot[0],self.pivot[1]+dr],
            [self.pivot[0]+dr2,self.pivot[1]+dr2],
            [self.pivot[0]-dr2,self.pivot[1]+dr2],
            [self.pivot[0]+dr2,self.pivot[1]-dr2],
            [self.pivot[0]-dr2,self.pivot[1]-dr2],
        ]
    }
    fn is_intersecting_segment(&self, line: &LineExt) -> bool {
        match (self.contains_point(line.from()),self.contains_point(line.to())) {
            (false,false) => match Intersection::seg_cir(line,self.pivot,self.radius) {
                Intersection::None => false,
                Intersection::Point(_) |
                Intersection::Segment(_) => true,
            },
            _ => true,
        }
    }
    fn intersect_segment(&self, line: &LineExt) -> Intersection {
        match (self.contains_point(line.from()),self.contains_point(line.to())) {
            (true,true) => Intersection::Segment(line.ps),
            (true,_) => {
                let q = line.from();
                match Intersection::seg_cir(line,self.pivot,self.radius) {
                    Intersection::Point(p) => Intersection::Segment([q[0],q[1],p[0],p[1]]),
                    Intersection::None | Intersection::Segment(_) => unreachable!(),
                }
            },
            (_,true) => {
                let q = line.to();
                match Intersection::seg_cir(line,self.pivot,self.radius) {
                    Intersection::Point(p) => Intersection::Segment([q[0],q[1],p[0],p[1]]),
                    Intersection::None | Intersection::Segment(_) => unreachable!(),
                }
            },
            (false,false) => Intersection::seg_cir(line,self.pivot,self.radius),
        }
    }
}

#[derive(Debug,Clone,Copy)]
pub enum Intersection {
    None,
    Point([f64; 2]),
    Segment([f64; 4]),
}
impl Intersection {
    pub fn seg_seg(l1: &LineExt, l2: &LineExt) -> Intersection {
        let vm = v3_vec_mul(l1.eq,l2.eq);
        match vm[2].abs() > EPS {
            // point
            true => {
                let p = [ vm[0] / vm[2] , vm[1] / vm[2] ];
                match l1.projection_in_segment(p) && l2.projection_in_segment(p) {
                    true => Intersection::Point(p),
                    false => Intersection::None,
                }
            },
            false => match (l1.eq[2] - l2.eq[2]).abs() > EPS {
                // parallel
                true => Intersection::None,
                // same line
                false => {
                    let b1 = (0.0, l1.from());
                    let e1 = (l1.d, l1.to());
                    let (b2,e2) = {
                        let cfrom = l1.s_xy(l2.from());
                        let cto = l1.s_xy(l2.to());
                        match cfrom < cto {
                            true => ((cfrom, l2.from()),(cto, l2.to())),
                            false => ((cto, l2.to()),(cfrom, l2.from())),
                        }
                    };
                    let b = match b1.0 > b2.0 { true => b1, false => b2 }; // max of begins
                    let e = match e1.0 < e2.0 { true => e1, false => e2 }; // min of ends
                    match (b.0 - e.0).abs() < EPS {
                        true => Intersection::Point([b.1[0],b.1[1]]),
                        false => match b.0 < e.0 {
                            true => Intersection::Segment([b.1[0],b.1[1],e.1[0],e.1[1]]),
                            false => Intersection::None,
                        },
                    }
                },
            },
        }
    }
    pub fn seg_cir(l: &LineExt, p: [f64;2], r: f64) -> Intersection {
        if r < 0.0 { return Intersection::None; }
        let d = l.f_xy(p).abs();
        if d > r { return Intersection::None; }
        
        let s = l.s_xy(p);
        let c = [ l.ps[0] + s*l.seq[0] , l.ps[1] + s*l.seq[1] ];
        
        if (d-r).abs() < EPS {
            return match l.projection_in_segment(c) {
                true => Intersection::Point(c),
                false => Intersection::None,
            };
        }

        let dc = (r*r - d*d).powf(0.5);
        let c1 = [c[0]-dc*l.seq[0],c[1]-dc*l.seq[1]];
        let c2 = [c[0]+dc*l.seq[0],c[1]+dc*l.seq[1]];
        match (l.projection_in_segment(c1),l.projection_in_segment(c2)) {
            (true,true) => Intersection::Segment([c1[0],c1[1],c2[0],c2[1]]),
            (true,false) => Intersection::Point(c1),
            (false,true) => Intersection::Point(c2),
            (false,false) => Intersection::None,
        }
    }
}
