use graphics::*;
use graphics::math::Matrix2d;

use crate::{
    GlContext,
};

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum Button {
    Left,
    Right,
}

#[derive(Debug,Clone,Copy)]
pub enum CursorState {
    None,
    Drag{ from: [f64; 2], tm: std::time::Instant, button: Button },
    Click{ from: [f64; 2], tm: std::time::Duration, button: Button },
}

#[derive(Debug,Clone)]
pub enum CursorAction {
    None,
    Processed,
    Click{ relative: [f64; 2], button: Button },
}
impl CursorAction {
    pub fn as_bool(&self) -> bool {
        match self {
            CursorAction::None => false,
            _ => true,
        }
    }
}

#[derive(Debug,Clone,Copy)]
pub struct Cursor {
    pub cursor: [f64; 2],
    pub scroll: [f64; 2],
    pub scroll_to_scale: bool,
    pub state: CursorState,
}
impl Cursor {
    pub fn new(x: f64, y: f64) -> Cursor {
        Cursor {
            cursor: [x, y],
            scroll: [0.0; 2],
            scroll_to_scale: false,
            state: CursorState::None,
        }
    }
    pub fn transform(&mut self, t: Matrix2d) {
        self.cursor = math::transform_pos(t,self.cursor);
        self.scroll = math::transform_vec(t,self.scroll);
        match &mut self.state {
            CursorState::Drag{ from, .. } => { *from = math::transform_pos(t,*from); },
            CursorState::Click{ from, .. } => { *from = math::transform_pos(t,*from); },
            CursorState::None => {},
        }
    }
}

#[derive(Debug,Clone,Copy)]
pub struct DrawContext {
    // piston context
    pub screen_size: (f64,f64),
    pub viewport: Option<Viewport>,
    pub view: Matrix2d,
    pub transform: Matrix2d,
    pub draw_state: DrawState,

    pub original_transform: Matrix2d,

    pub screen: Matrix2d,
    pub screen_back: Matrix2d,
}
impl DrawContext {
    pub fn scissor(&self, l: f64, t: f64, w: f64, h: f64) -> [u32; 4] {
        let sc1 = math::transform_pos(self.screen,[l,t]);
        let sc2 = math::transform_pos(self.screen,[l+w,t+h]);
        [sc1[0] as u32,sc1[1] as u32,(sc2[0]-sc1[0]) as u32,(sc2[1]-sc1[1]) as u32]
    }
    pub fn straight(&self, l: f64, t: f64, w: f64, h: f64) -> Matrix2d {
        let rm = f64::min(w,h);
        math::translate([(2.0*l+w)/self.screen_size.0-1.0,1.0-(2.0*t+h)/self.screen_size.1]).scale(rm/self.screen_size.0,rm/self.screen_size.1)
        //math::scale(rm/self.screen_size.0,rm/self.screen_size.1)
    }
    pub fn reverse(m: Matrix2d) -> Matrix2d {
        [[1.0/m[0][0],0.0,-m[0][2]/m[0][0]],[0.0,1.0/m[1][1],-m[1][2]/m[1][1]]]
    }
    pub fn l1_norm(v: [f64; 2]) -> f64 {
        v[0].abs() + v[1].abs()
    }
    /*pub fn l2_norm(v: [f64; 2]) -> f64 {
        (v[0].powi(2) + v[1].powi(2)).powf(0.5)
    }*/
}


pub trait DrawControl {
    fn draw<'t>(&mut self, c: &DrawContext, glc: &mut GlContext<'t>);

    fn cursor(&mut self, _cursor: Cursor) -> CursorAction { CursorAction::None }
}

pub struct Panel {
    color: [f32;4],
    bgcolor: [f32;4],
    rect: Rectangle,
    pub(crate) left: f64,
    pub(crate) top: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
    border: f64,
}
impl Panel {
    pub fn new(left: f64, top: f64, width: f64, height: f64) -> Panel {
        Panel {
            color: [0.0;4],
            bgcolor: [0.0;4],
            rect: Rectangle::new([0.0;4]),
            left: left,
            top: top,
            width: width,
            height: height,
            border: 3.0,
        }
    }
    pub fn with_color(mut self, color: [f32; 4]) -> Panel {
        self.color = color;
        self.rect = Rectangle::new(self.color).border(rectangle::Border{ color: self.bgcolor, radius: self.border });
        self
    }
    pub fn with_border_color(mut self, color: [f32; 4]) -> Panel {
        self.bgcolor = color;
        self.rect = Rectangle::new(self.color).border(rectangle::Border{ color: self.bgcolor, radius: self.border });
        self
    }
}
impl DrawControl for Panel {
    fn draw<'t>(&mut self, c: &DrawContext, glc: &mut GlContext<'t>) {
        let mut ds = c.draw_state;
        ds.scissor = Some(c.scissor(self.left,self.top,self.width,self.height));
        self.rect.draw([self.left,self.top,self.width,self.height],&ds,c.transform,&mut glc.gl);
    }
    fn cursor(&mut self, cursor: Cursor) -> CursorAction {
        let cursor = cursor.cursor;
        if (cursor[0] >= self.left)&&((cursor[0] <= (self.left+self.width)))&&
            (cursor[1] >= self.top)&&((cursor[1] <= (self.top+self.height))) {
                return CursorAction::Processed;
            }
        CursorAction::None
    }
}

/*pub struct ControlMap(pub BTreeMap<&'static str, Control>);
impl DrawControl for ControlMap {
    fn draw<'t>(&mut self, screen: Matrix2d, glyphs: &mut GlyphCache<'t>, c: &Context, gl: &mut GlGraphics) {
        for (_,child) in &mut self.0 {
            child.draw(screen,glyphs,c,gl);
        }
    }
}*/

pub struct TextData {
    data: Text,
    pos: [f64; 2],
    text: String,
}
impl TextData {
    pub fn new(color: [f32; 4], font_size: u32) -> TextData {
        TextData {
            data: Text::new_color(color,font_size),
            pos: [0.0, 0.0],
            text: "".to_string(),
        }
    }
    pub fn with_pos(mut self, pos: [f64; 2]) -> TextData {
        self.pos = pos;
        self
    }
    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }
}
impl DrawControl for TextData {
    fn draw<'t>(&mut self, c: &DrawContext, glc: &mut GlContext<'t>) {
        self.data.draw(&self.text,&mut glc.glyphs,&c.draw_state,c.transform.trans_pos(self.pos),&mut glc.gl).unwrap();
    }
}

/*
pub enum Control {
    Panel{ panel: Panel, children: ControlMap },
    Scene(Scene),
    Text(TextData),
}
impl DrawControl for Control {
    fn draw<'t>(&mut self, screen: Matrix2d, glyphs: &mut GlyphCache<'t>, c: &Context, gl: &mut GlGraphics) {
        match self {
            Control::Panel{ panel, children } => {
                panel.draw(screen,glyphs,c,gl);
                let mut c = *c;
                c.transform = c.transform.trans(panel.left,panel.top);
                children.draw(screen,glyphs,&c,gl);
            },
            Control::Scene(scene) => scene.draw(screen,glyphs,c,gl),
            Control::Text(text) => text.draw(screen,glyphs,c,gl),
        }
    }
}
*/
