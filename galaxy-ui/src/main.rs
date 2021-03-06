use std::{
    io,
    fs,
    sync::mpsc,
};

use glutin_window::{
    GlutinWindow as Window,
};
use opengl_graphics::{GlGraphics, OpenGL, GlyphCache, TextureSettings};
use piston::event_loop::{EventSettings, Events, EventLoop};
use piston::input::{RenderArgs};
use piston::window::{
    WindowSettings,Window as WindowTrait,
};

use graphics::*;

use piston::{
    Event,Loop,Input,ButtonArgs,Button,ButtonState,Motion,MouseButton,
    keyboard::Key,
};

use tokio::{
    self,
    runtime::Runtime,
    io::{
        AsyncBufReadExt,
        AsyncWriteExt,
    },
};

use futures::{
    channel::{
        mpsc::unbounded,
    },
    StreamExt,
};

use serde_derive::{
    Serialize,
    Deserialize,
};

use common::{
    vm::interpret::{
        Interpreter,
        OuterRequest,
    },
    proto::{
        galaxy,
        Session,
    },
    send::Intercom,
    encoder::{
        self,
        Modulable,
        PrettyPrintable,
    },
    code::*,
    parser::AsmParser,
};


const GRAY: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

mod map;
mod controls;
mod main_screen;
mod geom;

use crate::{
    controls::{
        Cursor,Button as CursorButton,CursorState,CursorAction,
        DrawControl,DrawContext,
    },
    map::{Map,MiniMap,MapSize},
};
use main_screen::MainScreen;

#[derive(Debug)]
pub struct Data {
    pub(crate) data: Vec<[f64; 2]>,
}
impl Data {
    pub fn width(&self) -> f64 { 400.0 }
    pub fn height(&self) -> f64 { 300.0 }
}

impl Data {
    fn from_pics(pics: Vec<Picture>) -> Vec<Data> {
        let mut dts = Vec::new();
        for p in pics {
            let Picture{ points } = p;
            let mut v = Vec::new();
            for p in points {
                let (x,y) = match p {
                    Coord { x: EncodedNumber { number: x, .. }, y: EncodedNumber { number: y, .. } } => {
                        (match x {
                            Number::Positive(PositiveNumber{ value }) => value as f64,
                            Number::Negative(NegativeNumber{ value }) => value as f64,
                        },match y {
                            Number::Positive(PositiveNumber{ value }) => value as f64,
                            Number::Negative(NegativeNumber{ value }) => value as f64,
                        })
                    },
                };
                v.push([x as f64, y as f64]);
            }
            if v.len() > 0 {
                println!("data: {}",v.len());
                dts.push(Data{ data: v });
            }
        }
        dts
    }

}

fn asm(session: &mut Session, asm: &str) -> Option<Ops> {
    //println!("ASM: {}",asm);
    match session.eval_asm(asm.clone()) {
        Ok(ops) => {
            Some(ops)
        },
        Err(e) => {
            println!("Error in asm: {:?}", e);
            println!("Asm: {:?}", asm);
            None
        },
    }
}

fn render_first(session: &mut Session, ops: &Ops) -> Option<Ops> {
    //ap render ap car ap cdr
    let mut nops = vec![
        Op::App, Op::Const(Const::Fun(Fun::Render)),
        Op::App, Op::Const(Const::Fun(Fun::Car)),
        Op::App, Op::Const(Const::Fun(Fun::Cdr)),
    ];
    nops.extend(ops.0.iter().map(|o|o.clone()));
    match session.eval_ops(Ops(nops)) {
        Ok(ops) => {
            if let Some(mut ops) =  match ops.0.len() {
                0 => None,
                1 => match &ops.0[0] {
                    Op::Const(Const::Fun(Fun::Nil)) => None,
                    _ => Some(ops),
                },
                _ => Some(ops),
            } {
                while let Some(n_ops) = render_next(session,&ops) {
                    ops = n_ops;
                }
            }
            None
        },
        Err(e) => {
            println!("Error in render_first: {:?}",e);
            None
        },
    }
}

fn render_next(session: &mut Session, ops: &Ops) -> Option<Ops> {
    //ap render
    let mut nops = vec![
        Op::App, Op::Const(Const::Fun(Fun::Render)),
    ];
    nops.extend(ops.0.iter().map(|o|o.clone()));
    match session.eval_ops(Ops(nops)) {
        Ok(ops) => {
            match ops.0.len() {
                0 => None,
                1 => match &ops.0[0] {
                    Op::Const(Const::Fun(Fun::Nil)) => None,
                    _ => Some(ops),
                },
                _ => Some(ops),
            }
        },
        Err(e) => {
            println!("Error in render_next: {:?}",e);
            None
        },
    }
}

#[derive(Serialize, Deserialize)]
struct GalaxyState {
    state: Ops,
    last_click: (i64, i64),
}

#[derive(Debug)]
enum GalaxyStateError {
    FileCreate(io::Error),
    SerdeWrite(serde_json::Error),
    FileOpen(io::Error),
    SerdeRead(serde_json::Error),
}

impl GalaxyState {
    fn save(&self) -> Result<(), GalaxyStateError> {
        let file = fs::File::create("./galaxy-state.json")
            .map_err(GalaxyStateError::FileCreate)?;
        let writer = io::BufWriter::new(file);
        serde_json::to_writer(writer, self)
            .map_err(GalaxyStateError::SerdeWrite)?;
        Ok(())
    }

    fn load() -> Result<GalaxyState, GalaxyStateError> {
        let file = fs::File::open("./galaxy-state.json")
            .map_err(GalaxyStateError::FileOpen)?;
        let reader = io::BufReader::new(file);
        serde_json::from_reader(reader)
            .map_err(GalaxyStateError::SerdeRead)
    }
}

fn extract_state(session: &mut Session, ops: Ops) -> Option<Ops> {
    let mut state_ops = Ops(vec![
        Op::App,
        Op::Const(Const::Fun(Fun::Car)),
    ]);
    state_ops.0.extend(ops.0);
    match session.eval_force_list(state_ops.clone()) {
        Ok(ops) => {
            if let [Op::Const(Const::ModulatedBits(bits))] = &*ops.0 {
                if let Ok(cons_list) = encoder::ConsList::demodulate_from_string(bits) {
                    println!(" // using current state = {}", cons_list.to_pretty_string());
                }
            }
            Some(ops)
        },
        Err(e) => {
            println!("Error in extract_state: {:?}",e);
            println!("state_ops: {:?}", state_ops);
            return None;
        },
    }
}

fn next(session: &mut Session, state_list_ops: Ops, x: i64, y: i64, valid_state: &mut GalaxyState) -> Option<Ops> {
    //ap ap ap interact galaxy nil ap ap vec 0 0

    let mut nops = Ops(vec![
        Op::App,
        Op::App,
        Op::App,
        Op::Const(Const::Fun(Fun::Interact)),
        Op::Const(Const::Fun(Fun::Galaxy)),
    ]);
    nops.0.extend(state_list_ops.0.clone());
    nops.0.extend(vec![
        Op::App,
        Op::App,
        Op::Const(Const::Fun(Fun::Vec)),
        Op::Const(Const::EncodedNumber(EncodedNumber {
            number: match x < 0 {
                true => Number::Negative(NegativeNumber{ value: x as isize }),
                false => Number::Positive(PositiveNumber{ value: x as usize }),
            },
            modulation: Modulation::Demodulated,
        })),
        Op::Const(Const::EncodedNumber(EncodedNumber {
            number: match y < 0 {
                true => Number::Negative(NegativeNumber{ value: y as isize }),
                false => Number::Positive(PositiveNumber{ value: y as usize }),
            },
            modulation: Modulation::Demodulated,
        })),
    ]);

    // println!("evaluating: {:?}", nops);

    match session.eval_ops(nops.clone()) {
        Ok(ops) => {
            valid_state.state = state_list_ops;
            valid_state.last_click = (x, y);
            Some(ops)
        },
        Err(e) => {
            println!("Error in next: {:?}",e);
            println!("nops: {:?}", nops);
            None
        },
    }
}

fn main() {
    let (picture_tx, picture_rx) = mpsc::channel();
    let (console_tx, console_rx) = mpsc::channel();
    let mut session = match session(picture_tx, console_tx) {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to create VM: {:?}",e);
            std::process::exit(1);
        },
    };

    let init_asm = "ap ap ap interact galaxy nil ap ap vec 0 0";
    //ap render ap car ap cdr
    let t = std::time::Instant::now();
    let mut current_frame_seq = None;
    let mut current = asm(&mut session,init_asm);
    println!("First step:   {:?}",t.elapsed());
    if let Some(ops) = &current {
        let t = std::time::Instant::now();
        current_frame_seq = render_first(&mut session,ops);
        println!("     render_first: {:?}",t.elapsed());
    }


    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new("View", [1280, 800])
        .graphics_api(opengl)
        //.resizable(false)
        .exit_on_esc(true)
        .build()
        .unwrap();
    //window = window.capture_cursor(false);

    let mut cursor = Cursor::new(640.0,400.0);
    window.ctx.window().set_cursor_position((640.0,400.0).into()).unwrap();
    //window.ctx.window().hide_cursor(true);

    let mut glc = GlContext {
        gl: GlGraphics::new(opengl),
        glyphs: GlyphCache::new("./Tahoma.ttf",(),TextureSettings::new()).unwrap(),
    };

    let ups = 10;
    let mut settings = EventSettings::new();
    settings.max_fps = 10;
    settings.ups = ups;
    let mut events = Events::new(settings);
    events.set_swap_buffers(false);
    let cntx = loop {
        if let Some(Event::Loop(Loop::Render(args))) = events.next(&mut window) {
            break init_render(&args, &mut glc, &mut window);
        }
    };

    let mut app = App {
        size: (1280.0,800.0),
        glc: glc,
        cursor: Cursor::new(0.0,0.0),
        main: MainScreen::new(&Data{ data: vec![] },&cntx),
    };
    app.cursor(cursor);

    let mut prev_pixels = vec![];
    let mut pictures_incoming: Option<Vec<Picture>> = None;
    let mut valid_state = GalaxyState {
        state: Ops(vec![Op::Const(Const::Fun(Fun::Nil))]),
        last_click: (0, 0),
    };

    while let Some(e) = events.next(&mut window) {
       //println!("[{:?}] {:.3} {:?}",start.elapsed(),app.rotation,e);

        match e {
            /*Event::Input(Input::Resize(args),_) => {
                window.ctx.window().set_inner_size((args.window_size[0],args.window_size[1]).into());
            },*/
            Event::Loop(Loop::Render(args)) => {
                app.render(&args,&mut window);
            },
            Event::Loop(Loop::Update(_args)) => {
                app.cursor(cursor);
                cursor.scroll = [0.0; 2];

                while let Ok(console_req) = console_rx.try_recv() {
                    match console_req {
                        ConsoleRequest::SetState { ops, } => {
                            println!("custom state installed: {:?}", ops);
                            current = next(
                                &mut session,
                                ops,
                                valid_state.last_click.0,
                                valid_state.last_click.1,
                                &mut valid_state,
                            );
                            println!("Load step:   {:?}",t.elapsed());
                            if let Some(ops) = &current {
                                app.main.scene.map.clear();
                                let t = std::time::Instant::now();
                                current_frame_seq = render_first(&mut session,ops);
                                println!("     render_first: {:?}",t.elapsed());
                            }
                        },
                    }
                }

                while let Ok(pics) = picture_rx.try_recv() {
                    match &mut pictures_incoming {
                        Some(ps) => ps.extend(pics.into_iter()),
                        None => pictures_incoming = Some(pics),
                    }
                }
                if let Some(pics) = pictures_incoming.take() {
                    prev_pixels.clear();
                    for Picture { points, } in &pics {
                        for Coord { x: EncodedNumber { number: x, .. }, y: EncodedNumber { number: y, .. } } in points {
                            prev_pixels.push((match x {
                                Number::Positive(PositiveNumber{ value }) => *value as i64,
                                Number::Negative(NegativeNumber{ value }) => *value as i64,
                            },match y {
                                Number::Positive(PositiveNumber{ value }) => *value as i64,
                                Number::Negative(NegativeNumber{ value }) => *value as i64,
                            }));
                        }
                    }

                    let datas = Data::from_pics(pics);
                    //
                    // println!("got {} pictures",datas.len());
                    for i in 0 .. datas.len() {
                        // println!("{:?}",data);
                        app.main.scene.map.next_data(&datas[datas.len() -1 -i ]);
                    }
                    if let Some(ops) = &current_frame_seq {
                        let t = std::time::Instant::now();
                        current_frame_seq = render_next(&mut session,ops);
                        println!("     render_next: {:?}",t.elapsed());
                    }
                }
            },
            Event::Input(Input::Button(ButtonArgs { state: ButtonState::Release, button, .. }),_) => match button {
                Button::Keyboard(Key::LCtrl) => { cursor.scroll_to_scale = false; },
                Button::Mouse(mb) => {
                    let but = match mb {
                        MouseButton::Left => CursorButton::Left,
                        MouseButton::Right => CursorButton::Right,
                        _ => continue,
                    };
                    cursor.state = match cursor.state {
                        CursorState::Drag { from, tm, button } if button == but => {
                            CursorState::Click{ from: from, tm: tm.elapsed(), button: but }
                        },
                        _ => CursorState::Click{ from: cursor.cursor, tm: std::time::Duration::new(0,0), button: but },
                    };
                    app.cursor(cursor);
                    {
                        let coo = app.main.scene.get_cursor().cursor;
                        if let Some(ops) = current.take() {
                            let t = std::time::Instant::now();
                            if let Some(state_list_ops) = extract_state(&mut session, ops) {
                                current = next(&mut session, state_list_ops, coo[0] as i64, coo[1] as i64, &mut valid_state);
                            }
                            //current = next(&mut session, ops,0,0);
                            println!("Next step ({:?}):   {:?}",coo,t.elapsed());
                            if let Some(ops) = &current {
                                app.main.scene.map.clear();
                                let t = std::time::Instant::now();
                                current_frame_seq = render_first(&mut session,ops);
                                println!("     render_first: {:?}",t.elapsed());
                            }
                            println!("waiting for next click...");
                            app.main.scene.map.cursor(coo);
                        }
                    }
                    cursor.state = CursorState::None;
                },
                _ => { /*println!("{:?}",button);*/ },
            },
            Event::Input(Input::Button(ButtonArgs { state: ButtonState::Press, button, .. }),_) => match button {
                Button::Keyboard(Key::A) => cursor.scroll[0] += 30.0, //app.scene.left(),
                Button::Keyboard(Key::D) => cursor.scroll[0] -= 30.0, //app.scene.right(),
                Button::Keyboard(Key::W) => cursor.scroll[1] += 30.0,
                Button::Keyboard(Key::S) => cursor.scroll[1] -= 30.0,
                Button::Keyboard(Key::LCtrl) => { cursor.scroll_to_scale = true; },
                Button::Mouse(MouseButton::Left) => { cursor.state = CursorState::Drag{ from: cursor.cursor, tm: std::time::Instant::now(), button: CursorButton::Left}; },
                Button::Mouse(MouseButton::Right) => { cursor.state = CursorState::Drag{ from: cursor.cursor, tm: std::time::Instant::now(), button: CursorButton::Right}; },
                Button::Keyboard(Key::Q) => {
                    app.cursor(cursor);
                    let coo = app.main.scene.get_cursor().cursor;
                    if let Some(ops) = current.take() {
                        let t = std::time::Instant::now();
                        if let Some(state_list_ops) = extract_state(&mut session, ops) {
                            current = next(&mut session, state_list_ops, coo[0] as i64, coo[1] as i64, &mut valid_state);
                        }
                        //current = next(&mut session, ops,0,0);
                        println!("Next step ({:?}):   {:?}",coo,t.elapsed());
                        if let Some(ops) = &current {
                            app.main.scene.map.clear();
                            let t = std::time::Instant::now();
                            current_frame_seq = render_first(&mut session,ops);
                            println!("     render_first: {:?}",t.elapsed());
                        }
                        println!("waiting for next 'q'...");
                    }
                },
                Button::Keyboard(Key::P) => {
                    use rand::Rng;

                    let mut maybe_prev_pics = None;
                    let mut rng = rand::thread_rng();
                    while !prev_pixels.is_empty() {
                        let index = rng.gen_range(0, prev_pixels.len());
                        let (x, y) = prev_pixels.swap_remove(index);
                        if let Some(ops) = current.take() {
                            let t = std::time::Instant::now();
                            if let Some(state_list_ops) = extract_state(&mut session, ops) {
                                current = next(&mut session, state_list_ops, x, y, &mut valid_state);
                            }
                            println!("Next step for ({}, {}):   {:?}", x, y, t.elapsed());
                            if let Some(ops) = &current {
                                app.main.scene.map.clear();
                                let t = std::time::Instant::now();
                                current_frame_seq = render_first(&mut session,ops);
                                println!("     render_first: {:?}",t.elapsed());
                            }
                            println!("waiting for results...");
                            match picture_rx.recv() {
                                Ok(pics) =>
                                    if let Some(prev_pics) = maybe_prev_pics.take() {
                                        if pics == prev_pics {
                                            maybe_prev_pics = Some(prev_pics);
                                        } else {
                                            println!("a new image received!");
                                            pictures_incoming = Some(pics);
                                            break;
                                        }
                                    } else {
                                        maybe_prev_pics = Some(pics);
                                    },
                                Err(..) => {
                                    println!("receive channel is broken");
                                    break;
                                },
                            }
                        }
                    }
                },
                Button::Keyboard(Key::N) => {
                    println!("saving current galaxy state");
                    if let Err(error) = valid_state.save() {
                        println!(" !! failed to save state: {:?}", error);
                    }
                },
                Button::Keyboard(Key::M) => {
                    println!("loading current galaxy state");
                    match GalaxyState::load() {
                        Ok(galaxy_state) => {
                            let t = std::time::Instant::now();
                            current = next(
                                &mut session,
                                galaxy_state.state,
                                galaxy_state.last_click.0,
                                galaxy_state.last_click.1,
                                &mut valid_state,
                            );
                            println!("Load step:   {:?}",t.elapsed());
                            if let Some(ops) = &current {
                                app.main.scene.map.clear();
                                let t = std::time::Instant::now();
                                current_frame_seq = render_first(&mut session,ops);
                                println!("     render_first: {:?}",t.elapsed());
                            }
                        },
                        Err(error) =>
                            println!(" !! failed to load state: {:?}", error),
                    }
                },

                _ => {},
            },
            Event::Input(Input::Move(Motion::MouseCursor(cur)),_) => { cursor.cursor = cur; },
            Event::Input(Input::Move(Motion::MouseScroll(delta)),_) => { cursor.scroll[0] += delta[0]; cursor.scroll[1] += delta[1]; },
            Event::Input(Input::Move(Motion::MouseRelative(_)),_) => { },
            Event::Input(_ev,_) => {
                //println!("{:?}",_ev);
            },
            _ => {}
        }
    }
}


pub struct Scene {
    left: f64,
    top: f64,
    width: f64,
    height: f64,

    transform: [[f64; 3]; 2],
    back_transform: [[f64; 3]; 2],

    scale: f64,
    cursor: Cursor,

    map: Map,
}
impl Scene {
    fn new(data: &Data, l: f64, t: f64, w: f64, h: f64, c: &DrawContext) -> Scene {
        let screen_w = c.screen_size.0;
        let screen_h = c.screen_size.1;
        let rm = f64::min(w,h);
        let sc = 300.0; //8.0;
        let transform = math::translate([(2.0*l+w)/screen_w-1.0,1.0-(2.0*t+h)/screen_h]).scale(rm/screen_w/sc,rm/screen_h/sc);
        let back = math::multiply(DrawContext::reverse(transform),c.transform);
        let mut scene = Scene {
            left: l, top: t, width: w, height: h,
            transform: transform,
            back_transform: back,
            scale: sc,
            cursor: Cursor::new(0.0,0.0),

            map: Map::new(data),
        };
        scene.scale_map(0.0);
        if scene.scale == sc { scene.move_map([0.0;2]); }
        scene
    }
    fn get_cursor(&self) -> Cursor {
        self.map.get_cursor(self.cursor)
    }
    fn current_map_size(&self) -> MapSize {
        let mins = math::transform_pos(self.back_transform,[self.left,self.top+self.height]);
        let maxs = math::transform_pos(self.back_transform,[self.left+self.width,self.top]);
        MapSize {
            size_x: (mins[0],maxs[0]),
            size_y: (mins[1],maxs[1]),
        }
    }
    pub fn mini_map(&self) -> MiniMap {
        self.map.mini()
    }
    pub fn mini_click(&mut self, pos: [f64; 2], button: CursorButton) {
        if (0.0 <= pos[0])&&(pos[0] <= 1.0)&&(0.0 <= pos[1])&&(pos[1] <= 1.0) {
            let t = math::translate([self.map.size.size_x.0,self.map.size.size_y.0])
                .scale(self.map.size.size_x.1 - self.map.size.size_x.0,self.map.size.size_y.1 - self.map.size.size_y.0);
            let pos = math::transform_pos(t,pos);
            match button {
                CursorButton::Left => {
                    let current = self.current_map_size();
                    let cntr = [(current.size_x.0 + current.size_x.1)/2.0,(current.size_y.0 + current.size_y.1)/2.0];
                    self.move_map(math::sub(cntr,pos));
                },
                CursorButton::Right => {},
            }
        }
    }
    fn move_map(&mut self, mut tmp: [f64; 2]) {
        let current = self.current_map_size();
        if (current.size_x.1 - tmp[0]) > self.map.size.size_x.1 { tmp[0] = -self.map.size.size_x.1 + current.size_x.1 }
        if (current.size_x.0 - tmp[0]) < self.map.size.size_x.0 { tmp[0] = -self.map.size.size_x.0 + current.size_x.0 }
        if (current.size_y.0 - tmp[1]) < self.map.size.size_y.0 { tmp[1] = -self.map.size.size_y.0 + current.size_y.0 }
        if (current.size_y.1 - tmp[1]) > self.map.size.size_y.1 { tmp[1] = -self.map.size.size_y.1 + current.size_y.1 }
        let old_transform = self.transform;
        self.transform = self.transform.trans_pos(tmp);
        self.back_transform = math::multiply(DrawContext::reverse(self.transform),math::multiply(old_transform,self.back_transform));
    }
    fn scale_map(&mut self, mut tmp: f64) {
        let current = self.current_map_size();
        let ws = self.scale * (self.map.size.size_x.1 - self.map.size.size_x.0)/(current.size_x.1 - current.size_x.0);
        let hs = self.scale * (self.map.size.size_y.1 - self.map.size.size_y.0)/(current.size_y.1 - current.size_y.0);
        let max_scale = f64::min(f64::min(ws,hs),200.0);

        if (self.scale + tmp) > max_scale { tmp = max_scale - self.scale; }
        let pscale = self.scale;
        self.scale += tmp;
        if self.scale < 1.0 { self.scale = 1.0; }
        if self.scale > max_scale { self.scale = max_scale; }
        let old_transform = self.transform;
        self.transform = self.transform.scale(pscale/self.scale,pscale/self.scale);
        self.back_transform = math::multiply(DrawContext::reverse(self.transform),math::multiply(old_transform,self.back_transform));
        self.move_map([0.0; 2])
    }
}




impl DrawControl for Scene {
    fn draw<'t>(&mut self, c: &DrawContext, glc: &mut GlContext<'t>) {
        let mut ds = c.draw_state;
        ds.scissor = Some(c.scissor(self.left,self.top,self.width,self.height));

        //self.back_transform = math::multiply(DrawContext::reverse(self.transform),c.transform);

        // let tr = math::multiply(DrawContext::reverse(c.transform),self.transform);

        // Draws chess background
        /*let t = c.transform.trans(self.left,self.top).scale(self.width,self.height);
        for r in &self.chess {
            rectangle([1.0, 1.0, 0.0, 0.05], *r, t, &mut glc.gl);
        }*/

        // Draws scale-rect in the center on the map
        /*
        let r = Rectangle::new([0.5,0.5,0.5,0.5]);
        r.draw([-1.0,-1.0,2.0,2.0],&ds,self.transform,&mut glc.gl);
        let r = Rectangle::new([1.0,1.0,1.0,1.0]);
        r.draw([-0.01,-0.01,0.02,0.02],&ds,self.transform,&mut glc.gl);

        Text::new_color([1.0,1.0,1.0,1.0],12).draw(&format!("({},{})",0.0,0.0),&mut glc.glyphs,&ds,
                                                   c.transform.trans_pos(math::transform_pos(tr,[0.0,0.0]))
                                                   ,&mut glc.gl).unwrap();

        Text::new_color([1.0,1.0,1.0,1.0],12).draw(&format!("({},{})",-1.0,1.0),&mut glc.glyphs,&ds,
                                                   c.transform.trans_pos(math::transform_pos(tr,[-1.0,1.0]))
                                                   ,&mut glc.gl).unwrap();
        */

        // Map draw
        let mut map_c = *c;
        map_c.transform = self.transform;
        map_c.draw_state = ds;

        let current = self.current_map_size();
        self.map.current_view = current;
        self.map.draw(&map_c,glc);

        // Cursor draw
        if let CursorState::Drag{ from, button, .. } = self.cursor.state {
            let cursor = self.cursor.cursor;
            let radius = math::transform_vec(self.back_transform,[1.0,0.0])[0];
            let r = match button {
                CursorButton::Left => Rectangle::new([0.0; 4]).border(rectangle::Border{ color: [1.0,1.0,0.0,1.0], radius: radius }),
                CursorButton::Right => Rectangle::new([0.0; 4]).border(rectangle::Border{ color: [1.0,0.0,0.0,1.0], radius: radius }),
            };
            let (x1,x2) = if from[0] < cursor[0] { (from[0],cursor[0]) } else { (cursor[0],from[0]) };
            let (y1,y2) = if from[1] < cursor[1] { (from[1],cursor[1]) } else { (cursor[1],from[1]) };
            r.draw([x1,y1,x2-x1,y2-y1],&ds,self.transform,&mut glc.gl);
        }
    }

    fn cursor(&mut self, mut cursor: Cursor) -> CursorAction {
        let cur = cursor.cursor;
        if (cur[0] >= self.left)&&((cur[0] <= (self.left+self.width)))&&
            (cur[1] >= self.top)&&((cur[1] <= (self.top+self.height))) {
                cursor.transform(self.back_transform);
                self.cursor = cursor;

                match cursor.scroll_to_scale {
                    false => if DrawContext::l1_norm(cursor.scroll) > 0.005 {
                        self.move_map(math::mul_scalar(cursor.scroll,1.5));
                    },
                    true => self.scale_map(-cursor.scroll[1]),
                }
                if let CursorState::Click{ from, button, tm:_ } = self.cursor.state {
                    let cursor = self.cursor.cursor;
                    let (x1,x2) = if from[0] < cursor[0] { (from[0],cursor[0]) } else { (cursor[0],from[0]) };
                    let (y1,y2) = if from[1] < cursor[1] { (from[1],cursor[1]) } else { (cursor[1],from[1]) };
                    match button {
                        CursorButton::Left => self.map.select([x1,y1,x2-x1,y2-y1]),
                        CursorButton::Right => self.map.act([x1,y1,x2-x1,y2-y1]),
                    }
                    //println!("Click: {:?} {:?} {:?}",button,from,cursor);
                }
                return CursorAction::Processed;
            }
        CursorAction::None
    }
}

pub struct GlContext<'t> {
    pub gl: GlGraphics,
    pub glyphs: GlyphCache<'t>,
}

pub struct App<'t> {
    size: (f64,f64),
    glc: GlContext<'t>,

    cursor: Cursor,
    main: MainScreen,
}
fn init_render<W: WindowTrait>(args: &RenderArgs, glc: &mut GlContext, window: &mut W) -> DrawContext {
    let view = args.viewport();
    let c = {
        let c = glc.gl.draw_begin(view);
        let x = view.window_size[0]/1440.0;
        let y = view.window_size[1]/900.0;

        let m = math::scale(x,y);
        DrawContext {
            screen_size: (1440.0,900.0),
            viewport: c.viewport,
            view: c.view,
            transform: c.transform.scale(x,y),
            draw_state: c.draw_state,

            original_transform: c.transform.scale(x,y),

            screen: m,
            screen_back: DrawContext::reverse(m),
        }
    };
    clear(GRAY,&mut glc.gl);
    glc.gl.draw_end();
    window.swap_buffers();
    c
}
impl<'t> App<'t> {
    fn render<W: WindowTrait>(&mut self, args: &RenderArgs, window: &mut W) {
        let view = args.viewport();
        self.size = (view.window_size[0],view.window_size[1]);
        let c = {
            let c = self.glc.gl.draw_begin(view);
            let x = view.window_size[0]/1440.0;
            let y = view.window_size[1]/900.0;

            let m = math::scale(x,y);
            DrawContext {
                screen_size: (1440.0,900.0),
                viewport: c.viewport,
                view: c.view,
                transform: c.transform.scale(x,y),
                draw_state: c.draw_state,

                original_transform: c.transform.scale(x,y),

                screen: m,
                screen_back: DrawContext::reverse(m),
            }
        };

        {
            clear(GRAY,&mut self.glc.gl);
            self.main.draw(&c,&mut self.glc);
        }
        /*{
            let el = Ellipse::new([0.0; 4]).border(ellipse::Border{ color: [1.0; 4], radius: 1.0}).resolution(20);
            el.draw([-10.0,-10.0,20.0,20.0],&c.draw_state,c.view.trans_pos(self.cursor.cursor),&mut self.glc.gl);
        }*/
        self.glc.gl.draw_end();
        self.glc.gl.use_draw_state(&c.draw_state);
        window.swap_buffers();
    }

    fn cursor(&mut self, cursor: Cursor) {
        self.cursor = cursor;
        self.main.cursor(self.cursor);
    }
}

fn session(sender: mpsc::Sender<Vec<Picture>>, console_tx: mpsc::Sender<ConsoleRequest>) -> Result<Session,common::proto::Error> {
    let (outer_tx, mut outer_rx) = unbounded();

    let session = Session::with_interpreter(
        galaxy(),
        Interpreter::with_outer_channel(outer_tx),
    )?;

    std::thread::spawn(move || {
        let intercom = Intercom::proxy();
        let mut runtime = Runtime::new().unwrap();

        runtime.spawn(async move {
            if let Err(error) = run_server(console_tx).await {
                println!(" !! server has crashed with: {:?}", error);
            }
        });

        runtime.block_on(async {
            while let Some(request) = outer_rx.next().await {
                match request {
                    OuterRequest::RenderPictures { pictures } => { sender.send(pictures).ok(); },
                    OuterRequest::ProxySend { modulated_req, modulated_rep, } => {
                        println!("** >> transmission rq: {:?}", modulated_req);
                        match intercom.async_send(modulated_req).await {
                            Ok(response) => {
                                println!("** << transmission rp: {:?}", response);
                                if let Err(..) = modulated_rep.send(response) {
                                    println!("interpreter has gone, quitting");
                                    break;
                                }
                            },
                            Err(error) => {
                                println!("intercom send failed: {:?}, quitting", error);
                                break;
                            },
                        }
                    },
                }
            }
        });
        println!("intercom task termination");
    });

    Ok(session)
}

enum ConsoleRequest {
    SetState { ops: Ops, },
}

use std::net::{
    IpAddr,
    Ipv4Addr,
    SocketAddr,
};
use tokio::net::{
    TcpStream,
    TcpListener,
};

#[derive(Debug)]
enum ServerError {
    TcpListenerBind(io::Error),
    Accept(io::Error),
}

async fn run_server(request_tx: mpsc::Sender<ConsoleRequest>) -> Result<(), ServerError> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 36142);
    let mut listener = TcpListener::bind(addr).await
        .map_err(ServerError::TcpListenerBind)?;

    println!("server: running on {:?}", addr);

    let mut incoming = listener.incoming();
    while let Some(maybe_socket) = incoming.next().await {
        let socket = maybe_socket.map_err(ServerError::Accept)?;
        let client_peer = socket.peer_addr();
        println!("server: accepted connection from {:?}", client_peer);
        let client_request_tx = request_tx.clone();
        tokio::spawn(async move {
            if let Err(error) = run_client(socket, client_request_tx).await {
                println!(" !! client {:?} has crashed with: {:?}", client_peer, error);
            }
        });
    }

    Ok(())
}

#[derive(Debug)]
enum ClientError {
    ReadLine(io::Error),
    WriteReply(io::Error),
    FlushReply(io::Error),
    RequestChannelClosed,
}

async fn run_client(mut socket: TcpStream, request_tx: mpsc::Sender<ConsoleRequest>) -> Result<(), ClientError> {
    let (reader, mut writer) = socket.split();
    let mut buf_reader = tokio::io::BufReader::new(reader);

    let parser = AsmParser::new();
    let mut line_buf = String::new();
    loop {
        line_buf.clear();
        let bytes_read = buf_reader.read_line(&mut line_buf).await
            .map_err(ClientError::ReadLine)?;
        if bytes_read == 0 {
            break;
        }
        match line_buf[.. bytes_read].trim() {
            command if command.starts_with("/set state:") =>
                match parser.parse_expression(line_buf[11 .. bytes_read].trim()) {
                    Ok(ops) => {
                        request_tx.send(ConsoleRequest::SetState { ops, })
                            .map_err(|_| ClientError::RequestChannelClosed)?;
                        writer.write_all("OK\n".as_bytes())
                            .await.map_err(ClientError::WriteReply)?;
                        writer.flush()
                            .await.map_err(ClientError::FlushReply)?;
                    }
                    Err(error) => {
                        writer.write_all(format!("PARSE ERROR: {:?}\n", error).as_bytes())
                            .await.map_err(ClientError::WriteReply)?;
                        writer.flush()
                            .await.map_err(ClientError::FlushReply)?;
                    },
                },
            command => {
                writer.write_all(format!("INVALID COMMAND: {:?}\n", command).as_bytes()).await
                    .map_err(ClientError::WriteReply)?;
                writer.flush().await
                    .map_err(ClientError::FlushReply)?;
            },
        }
    }

    Ok(())
}
