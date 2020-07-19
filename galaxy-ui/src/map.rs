use graphics::*;
use graphics::math::Matrix2d;

use rand::{
    Rng,
    seq::SliceRandom,
    distributions::Distribution,
};

use rstar::{RTree,RTreeObject,AABB,PointDistance};

use crate::{
    Data,
    GlContext,
    geom::{self,LineExt,Intersection,Form},
    controls::{
        Cursor,Button as CursorButton,CursorState,CursorAction,
        DrawControl,DrawContext,
    },
    kdtree::{KDTree,l2_norm2},
};

use std::collections::VecDeque;

const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

const P: f32 = 3.141592654;


#[derive(Debug,Clone,Copy)]
pub struct MapSize {
    pub size_x: (f64,f64),
    pub size_y: (f64,f64),
}

pub struct MiniMap {
    map: [f64; 4],
    view: [f64; 4],
}
impl DrawControl for MiniMap {
    fn draw<'t>(&mut self, c: &DrawContext, glc: &mut GlContext<'t>) {
        let back = math::multiply(DrawContext::reverse(c.transform),c.original_transform);
        let radius = math::transform_vec(back,[0.5,0.0])[0];

        let t = c.transform.trans(-1.0,-1.0).scale(2.0,2.0);
        let r = Rectangle::new([0.0,0.0,0.0,1.0]);
        r.draw(self.map,&c.draw_state,t,&mut glc.gl);
        let r = Rectangle::new([0.0,0.0,0.0,0.0]).border(rectangle::Border{ color: [1.0,1.0,1.0,1.0], radius: radius });
        r.draw(self.view,&c.draw_state,t,&mut glc.gl);
        let r = Rectangle::new([0.0,0.0,0.0,0.0]).border(rectangle::Border{ color: [1.0,0.0,0.0,1.0], radius: 2.0*radius });
        r.draw(self.map,&c.draw_state,t,&mut glc.gl);
    }
    fn cursor(&mut self, mut cursor: Cursor) -> CursorAction {
        if let CursorState::Click{ from: _, button: CursorButton::Left, .. } = cursor.state {
            let t = math::scale(0.5,0.5).trans(1.0,1.0);
            cursor.transform(t);
            let pos = [(cursor.cursor[0] - self.map[0])/self.map[2],(cursor.cursor[1] - self.map[1])/self.map[3]];
            if (0.0 <= pos[0])&&(pos[0] <= 1.0)&&(0.0 <= pos[1])&&(pos[1] <= 1.0) {
                return CursorAction::Click{ relative: pos, button: CursorButton::Left };
            }
        }
        CursorAction::Processed
    }
}

pub struct Map {
    pub size: MapSize,
    pub current_view: MapSize,

    lines: Vec<([f64; 2],[f64;2])>,
    lines2: Vec<([f64; 2],[f64;2])>,
    obstacles: Obstacles,
    dots: DotsXYT,

    //path_tree: Vec<usize>,
    //dq: VecDeque<(f64,f64)>,

    rng: rand::prelude::ThreadRng,
    //kdt: KDTree,
}

#[derive(Clone,Debug,Copy)]
enum Obstacle {
    Circle(geom::CircleExt),
    Line(geom::LineExt),
    Rect(geom::RectExt),
    RectInTime{ rect: geom::RectExt, tm: f64 },
    //Tri([f64;6])
}



pub struct DotsXYT {
    sx: f64,
    sy: f64,
    mx: f64,
    my: f64,
    dtm: f64,
    dots: Vec<Obstacle>,
}
impl DotsXYT {
    pub fn new(data: &Data, sx: f64, sy: f64) -> DotsXYT {
        let mx = sx.abs()*2.0;
        let my = sy.abs()*2.0;
        let mut v = Vec::new();
        for coo in &data.data {
            if (coo[0]<sx)||(coo[1]<sy)||(coo[0]>mx)||(coo[1]>my) {
                println!("Point out-of-range: {:?}",coo);
            }
            v.push(Obstacle::RectInTime{
                //rect: geom::RectExt::new([sx + coo[0],sy - coo[1]-1.0,1.0,1.0]).unwrap(),
                rect: geom::RectExt::new([coo[0],-coo[1],1.0,1.0]).unwrap(),
                tm: 1.0,
            });
        }
        DotsXYT {
            sx: sx,
            sy: sy,
            mx: mx,
            my: my,
            dtm: 0.11,
            dots: v,
        }
    }
    pub fn next(&mut self, data: &Data) {
        self.dots = self.dots.iter().filter_map(|obs| {
            let mut obs = obs.clone();
            let mut drop = false;
            if let Obstacle::RectInTime{ tm, .. } = &mut obs {
                *tm -= self.dtm;
                if *tm <= 0.0 {
                    drop = true;
                }
            }
            match drop {
                true => None,
                false => Some(obs),
            }
        }).collect();
        for coo in &data.data {
            if (coo[0]<0.0)||(coo[1]<0.0)||(coo[0]>self.mx)||(coo[1]>self.my) {
                println!("Point out-of-range: {:?}",coo);
            }
            self.dots.push(Obstacle::RectInTime{
                rect: geom::RectExt::new([self.sx + coo[0],self.sy - coo[1]-1.0,1.0,1.0]).unwrap(),
                tm: 1.0,
            });
        }
    }
}

pub struct Obstacles {
    tree: RTree<Obstacle>,
}
impl Obstacles {
    /*pub fn rect_mask(data: Data, sx: f64, sy: f64) -> Obstacles {
        let mut v = Vec::new();
        for (y,row) in data.data.iter().enumerate() {
            for (x,r) in row.iter().enumerate() {
                if *r > 0 {
                    v.push(Obstacle::Rect(geom::RectExt::new([sx + x as f64,sy - (y+1) as f64,1.0,1.0]).unwrap()));
                }
            }
        }
        Obstacles {
            tree: RTree::bulk_load(v),
        }
    }*/
    pub fn empty() -> Obstacles {
        Obstacles {
            tree: RTree::bulk_load(Vec::new()),
        }
    }
    /*pub fn random(rng: &mut rand::prelude::ThreadRng, free: Vec<[f64;2]>) -> Obstacles {
        let mut v = Vec::new();
        let free = free.into_iter().map(|p| geom::RectExt::new([p[0]-1.0,p[1]-1.0,2.0,2.0]).unwrap()).collect::<Vec<_>>();
        let rand_n = 1000;
        for _ in 0 .. 3000 {
            let (x,y) = loop {
                let x = rng.gen::<f64>() * 50.0 - 25.0;
                let y = rng.gen::<f64>() * 40.0 - 20.0;
                let mut ok = true;
                for f in &free {
                    if f.contains_point([x,y]) { ok = false; break; }
                }
                if ok { break (x,y); }
            };
            let r = rng.gen::<f64>() * 0.2 + 0.1;
            v.push(Obstacle::Circle(geom::CircleExt::new([x,y],r).unwrap()));
        }
        Obstacles {
            random_part_tree: match v.len() > rand_n {
                true => Some(RTree::bulk_load(v[0..rand_n].to_vec())),
                false => None,
            },
            tree: RTree::bulk_load(v),
        }
    }
    pub fn vertical() -> Obstacles {     
        Obstacles {
            random_part_tree: None,
            tree: RTree::bulk_load(vec![
                Obstacle::Line(LineExt::new([-19.0,-19.0,-19.0,21.0]).unwrap()),
                Obstacle::Line(LineExt::new([-19.1,-19.0,-19.1,21.0]).unwrap()),
                Obstacle::Line(LineExt::new([-19.0,-19.0,-19.1,-19.0]).unwrap()),

                Obstacle::Line(LineExt::new([-17.0,19.0,-17.0,-21.0]).unwrap()),
                Obstacle::Line(LineExt::new([-17.1,19.0,-17.1,-21.0]).unwrap()),
                Obstacle::Line(LineExt::new([-17.0,19.0,-17.1,19.0]).unwrap()),

                Obstacle::Line(LineExt::new([-15.0,-19.0,-15.0,21.0]).unwrap()),
                Obstacle::Line(LineExt::new([-15.1,-19.0,-15.1,21.0]).unwrap()),
                Obstacle::Line(LineExt::new([-15.0,-19.0,-15.1,-19.0]).unwrap()),             

                Obstacle::Line(LineExt::new([-13.0,19.0,-13.0,-21.0]).unwrap()),
                Obstacle::Line(LineExt::new([-13.1,19.0,-13.1,-21.0]).unwrap()),
                Obstacle::Line(LineExt::new([-13.0,19.0,-13.1,19.0]).unwrap()),

                Obstacle::Line(LineExt::new([-7.0,-19.0,-7.0,21.0]).unwrap()),
                Obstacle::Line(LineExt::new([-7.1,-19.0,-7.1,21.0]).unwrap()),
                Obstacle::Line(LineExt::new([-7.0,-19.0,-7.1,-19.0]).unwrap()),

                Obstacle::Line(LineExt::new([0.0,19.0,0.0,-21.0]).unwrap()),
                Obstacle::Line(LineExt::new([0.1,19.0,0.1,-21.0]).unwrap()),
                Obstacle::Line(LineExt::new([0.0,19.0,0.1,19.0]).unwrap()),

                Obstacle::Line(LineExt::new([19.0,-19.0,19.0,21.0]).unwrap()),
                Obstacle::Line(LineExt::new([19.1,-19.0,19.1,21.0]).unwrap()),
                Obstacle::Line(LineExt::new([19.0,-19.0,19.1,-19.0]).unwrap()),

                Obstacle::Line(LineExt::new([13.0,19.0,13.0,-21.0]).unwrap()),
                Obstacle::Line(LineExt::new([13.1,19.0,13.1,-21.0]).unwrap()),
                Obstacle::Line(LineExt::new([13.0,19.0,13.1,19.0]).unwrap()),

                Obstacle::Line(LineExt::new([7.0,-19.0,7.0,21.0]).unwrap()),
                Obstacle::Line(LineExt::new([7.1,-19.0,7.1,21.0]).unwrap()),
                Obstacle::Line(LineExt::new([7.0,-19.0,7.1,-19.0]).unwrap()),
            ]),
        }
    }*/
    pub fn contains_point(&self, p: [f64; 2]) -> bool {
        for obs in self.tree.locate_in_envelope_intersecting(&AABB::from_point(p)) {
            if match obs {
                Obstacle::Line(o) => o.contains_point(p),
                Obstacle::Rect(o) |
                Obstacle::RectInTime{ rect: o, .. } => o.contains_point(p),
                Obstacle::Circle(o) => o.contains_point(p),
            } { return true; }
        }
        false
    }
    pub fn intersect(&self, p1: [f64; 2], p2: [f64; 2]) -> bool {
        enum Tmp {
            Line(LineExt),
            Point([f64; 2]),
        }

        let t = match LineExt::from_to(p1,p2) {
            Ok(ln) => Tmp::Line(ln),
            Err(_) => Tmp::Point(p1),
        };
        for obs in self.tree.locate_in_envelope_intersecting(&AABB::from_corners(p1,p2)) {
            if match obs {
                Obstacle::Line(ln) => match t {
                    Tmp::Line(tmp) => ln.is_intersecting_segment(&tmp),
                    Tmp::Point(p) => ln.contains_point(p),
                },
                Obstacle::RectInTime{ rect: r, .. } |
                Obstacle::Rect(r) => match t {
                    Tmp::Line(tmp) => r.is_intersecting_segment(&tmp),
                    Tmp::Point(p) => r.contains_point(p),
                },
                Obstacle::Circle(c) => match t {
                    Tmp::Line(tmp) => c.is_intersecting_segment(&tmp),
                    Tmp::Point(p) => c.contains_point(p),
                },
            } { return true; }
        }
        false
    }
}

impl RTreeObject for Obstacle {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let r = match self {
            Obstacle::Line(o) => o.mbr(),
            Obstacle::Circle(o) => o.mbr(),
            Obstacle::RectInTime{ rect: o, .. } |
            Obstacle::Rect(o) => o.mbr(),
        };
        AABB::from_corners([r[0],r[1]],[r[0]+r[2],r[1]+r[3]])
    }
}
impl MapObject for Obstacle {
    fn rect_intersect(&self, rect: [f64; 4]) -> bool {
        match self {
            Obstacle::Line(o) => math::overlap_rectangle(o.mbr(), rect).is_some(),
            Obstacle::Circle(o) => math::overlap_rectangle(o.mbr(), rect).is_some(),
            Obstacle::RectInTime{ rect: o, .. } |
            Obstacle::Rect(o) => math::overlap_rectangle(o.mbr(), rect).is_some(),
        }
    }
}
impl DrawControl for Obstacle {
    fn draw<'t>(&mut self, c: &DrawContext, glc: &mut GlContext<'t>) {
        match self {
            Obstacle::Line(ln) => Line::new([0.0,0.0,1.0,1.0],0.03).draw_from_to(ln.from(),ln.to(),&c.draw_state,c.transform,&mut glc.gl),
            Obstacle::Rect(r) => Rectangle::new([0.0,0.0,1.0,0.7]).border(rectangle::Border{ color: [0.0,0.0,1.0,1.0], radius: 0.03 }).draw(r.rect(),&c.draw_state,c.transform,&mut glc.gl),
            Obstacle::RectInTime{ rect: r, tm: t } => Rectangle::new([0.0,0.0,1.0,0.7*(*t) as f32]).border(rectangle::Border{ color: [0.0,0.0,1.0,1.0*(*t) as f32], radius: 0.03 }).draw(r.rect(),&c.draw_state,c.transform,&mut glc.gl),
            Obstacle::Circle(cq) => Ellipse::new([0.0,0.0,1.0,0.7]).border(ellipse::Border{ color: [0.0,0.0,1.0,1.0], radius: 0.03 }).draw(cq.mbr(),&c.draw_state,c.transform,&mut glc.gl),
        }
    }
}

impl Map {
    pub fn get_cursor(&self, mut cursor: Cursor) -> Cursor {
        let mut x = cursor.cursor[0];
        let mut y = cursor.cursor[1];
        x -= self.size.size_x.0;
        y = self.size.size_y.1 - y;
        cursor.cursor = [x.floor(),y.floor()];
        cursor
    }
    pub fn next_data(&mut self, data: &Data) {
        self.dots.next(&data)
    }
    pub fn new(data: &Data) -> Map {
        let mut dx = data.width();
        let mut dy = data.height();
        
        if dx < 80.0 { dx = 80.0; }
        if dy < 60.0 { dy = 60.0; }

        let size_x = ( -dx/2.0, dx/2.0);
        let size_y = ( -dy/2.0, dy/2.0);
        
        let mut rng = rand::thread_rng();
        //let start = [-24.0,19.0];
        //let target = [24.0,-19.0];
        let mut t = std::time::Instant::now();
        let obs = Obstacles::empty();

        let dots = DotsXYT::new(data,size_x.0,size_y.1);
        
        
        let mut map = Map {
            size: MapSize {
                size_x: size_x,
                size_y: size_y,
            },
            current_view: MapSize {
                size_x: (0.0,0.0),
                size_y: (0.0,0.0),
            },
            lines: Vec::new(),//gen_n_lines(1000,&mut rng,&mut kdt,(-25.0,25.0),(-20.0,20.0),&obs,&mut dq, &mut path),
            obstacles: obs,
            dots: dots,
            rng: rng,
            lines2: Vec::new(),
            
        };
        map
    }
    pub fn mini(&self) -> MiniMap {
        let mw = self.size.size_x.1 - self.size.size_x.0;
        let mh = self.size.size_y.1 - self.size.size_y.0;
        let (dw,w,dh,h,max) = if mw > mh {
            let max = mw;
            let h = mh/max;
            (0.0,1.0,(1.0-h)/2.0,h,max)
        } else {
            let max = mh;
            let w = mw/max;
            ((1.0-w)/2.0,w,0.0,1.0,max)
        };
        let vx = (self.current_view.size_x.0 - self.size.size_x.0) / max;
        let vw = (self.current_view.size_x.1 - self.current_view.size_x.0) / max;
        let vy = (self.current_view.size_y.0 - self.size.size_y.0) / max;
        let vh = (self.current_view.size_y.1 - self.current_view.size_y.0) / max;
        MiniMap {
            map: [dw,dh,w,h],
            view: [dw+vx,dh+vy,vw,vh],
        }
    }
}
impl Map {
    pub fn select(&mut self, rect: [f64; 4]) {
        
    }
    pub fn act(&mut self, rect: [f64; 4]) {

    }
}
impl DrawControl for Map {
    fn draw<'t>(&mut self, c: &DrawContext, glc: &mut GlContext<'t>) {
        let c1 = [self.current_view.size_x.0,self.current_view.size_y.0];
        let c2 = [self.current_view.size_x.1,self.current_view.size_y.1];
        let map_rect = [self.current_view.size_x.0,self.current_view.size_y.0,self.current_view.size_x.1 - self.current_view.size_x.0,self.current_view.size_y.1 - self.current_view.size_y.0];
        for obs in self.obstacles.tree.locate_in_envelope_intersecting_mut(&AABB::from_corners(c1,c2)) {
            if obs.rect_intersect(map_rect) {
                obs.draw(c,glc);
            }
        }

        for obs in &mut self.dots.dots {
            if obs.rect_intersect(map_rect) {
                obs.draw(c,glc);
            }
        }
        
        let line = Line::new([0.7,0.7,0.7,1.0],0.03);
        for (p1,p2) in &self.lines2 {
            let r = [f64::min(p1[0],p2[0]),f64::min(p1[1],p2[1]),(p1[0]-p2[0]).abs(),(p1[1]-p2[1]).abs()];
            if math::overlap_rectangle(r, map_rect).is_some() {
                line.draw_from_to(*p1,*p2,&c.draw_state,c.transform,&mut glc.gl);
            }
        }
        let line = Line::new([1.0,1.0,0.0,1.0],0.03);
        for (p1,p2) in &self.lines {
            let r = [f64::min(p1[0],p2[0]),f64::min(p1[1],p2[1]),(p1[0]-p2[0]).abs(),(p1[1]-p2[1]).abs()];
            if math::overlap_rectangle(r, map_rect).is_some() {
                line.draw_from_to(*p1,*p2,&c.draw_state,c.transform,&mut glc.gl);
            }
        } 
    }
    
}

pub trait MapObject {
    fn rect_intersect(&self, rect: [f64; 4]) -> bool;
}
