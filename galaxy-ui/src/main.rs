use glutin_window::{
    GlutinWindow as Window,
};
use opengl_graphics::{GlGraphics, OpenGL, GlyphCache, TextureSettings};
use piston::event_loop::{EventSettings, Events, EventLoop};
use piston::input::{RenderArgs, UpdateArgs};
use piston::window::{
    WindowSettings,Window as WindowTrait,
};

use graphics::*;

use piston::{
    Event,Loop,Input,ButtonArgs,Button,ButtonState,Motion,MouseButton,
    keyboard::Key,
};

use tokio::{self,runtime::Runtime};
use futures::{
    channel::{
        oneshot,
        mpsc::unbounded,
    },
    StreamExt,
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
    code::{Op,Ops,Picture,Coord,EncodedNumber,Const,Number,PositiveNumber,NegativeNumber},
};


const GRAY: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

mod kdtree;
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
    pub fn width(&self) -> f64 { 80.0 }
    pub fn height(&self) -> f64 { 60.0 }
}

fn tmp_data(dx: f64, dy: f64) -> Data {
     Data {
        data: {
            let mut v = Vec::new();
            let szx = 20;
            let szy = 10;
            for x in 0 .. szx {
                for y in 0 .. szy {
                    if (x==0)||(x==szx-1)||(y==0)||(y==szy-1)||((x%szy) == (szy-y-1)) {
                        v.push([x as f64 + dx, y as f64 + dy]);
                    }
                }
            }
            v
        },
    }
}

impl Data {
    fn from_ops(ops: Ops) -> Option<Data> {     
        for o in ops.0 {
            if let Op::Const(Const::Picture(Picture{ points })) = o {
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
                return Some(Data{ data: v });
            }
        }
        None
    }

}

fn asm_to_opt_data(session: &mut Session, asm: &str) -> Option<Data> {
    match session.eval_asm(asm) {
        Ok(ops) => Data::from_ops(ops),
        Err(e) => {
            println!("Error: {:?}",e);
            None
        },
    }
}


fn main() {
    let mut session = match session() {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to create VM: {:?}",e);
            std::process::exit(1);
        },
    };

    let init_asm = "ap draw ((1,1),(2,10))";
    
    let init_data = match asm_to_opt_data(&mut session, init_asm) {
        Some(data) => data,
        None => Data{ data: vec![] }, 
    };
    
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
        glyphs: GlyphCache::new("/Library/Fonts/Tahoma.ttf",(),TextureSettings::new()).unwrap(),
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
        main: MainScreen::new(&init_data,&cntx),
    };
    app.cursor(cursor);
    
    let mut t = std::time::Instant::now();
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
                //if t.elapsed() > std::time::Duration::new(0,500_000_000) {
                //    app.main.scene.map.test_step();
                //    t = std::time::Instant::now();
                //}
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
                        let asm = format!("ap draw (({},{}))",coo[0],coo[1]);
                        if let Some(data) = asm_to_opt_data(&mut session, &asm) {
                            app.main.scene.map.next_data(&data);
                        }
                        //println!("Click: {:?}",app.main.scene.get_cursor());
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
    
    ratio: (f64,f64),
    transform: [[f64; 3]; 2],
    back_transform: [[f64; 3]; 2],

    scale: f64,
    cursor: Cursor,

    map: Map,
    
    chess: Vec<[f64; 4]>,
}
impl Scene {
    fn new(data: &Data, l: f64, t: f64, w: f64, h: f64, c: &DrawContext) -> Scene {
        let screen_w = c.screen_size.0;
        let screen_h = c.screen_size.1;
        let rm = f64::min(w,h);
        let ratio = (w/rm, h/rm);
        let sc = 30.0; //8.0;
        let transform = math::translate([(2.0*l+w)/screen_w-1.0,1.0-(2.0*t+h)/screen_h]).scale(rm/screen_w/sc,rm/screen_h/sc);
        let back = math::multiply(DrawContext::reverse(transform),c.transform);
        let mut scene = Scene {
            left: l, top: t, width: w, height: h,
            ratio: ratio,
            transform: transform,
            back_transform: back,
            scale: sc,
            cursor: Cursor::new(0.0,0.0),

            map: Map::new(data),
            
            chess: {
                let r = 72.0;
                let ratio = (ratio.0 * r, ratio.1 * r);
                let mut v = Vec::new();
                let dx = 1.0 / ratio.0;
                let dy = 1.0 / ratio.1;
                for i in 0 .. (ratio.0.round() as usize) {
                    for j in 0 .. (ratio.1.round() as usize) {
                        if (i+j)%2 == 1 {
                            v.push(rectangle::rectangle_by_corners(i as f64*dx,j as f64*dy,(i+1) as f64*dx,(j+1) as f64*dy));
                        }
                    }
                }
                v
            },
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
        let max_scale = f64::min(f64::min(ws,hs),100.0);
                                
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

        let tr = math::multiply(DrawContext::reverse(c.transform),self.transform);

        // Draws chess background
        let t = c.transform.trans(self.left,self.top).scale(self.width,self.height);      
        for r in &self.chess {
            rectangle([1.0, 1.0, 0.0, 0.05], *r, t, &mut glc.gl);
        }

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

    fn update(&mut self, _args: &UpdateArgs) {
        
    }

    fn cursor(&mut self, cursor: Cursor) {
        self.cursor = cursor;
        self.main.cursor(self.cursor);
    }
}

fn session() -> Result<Session,common::proto::Error> {
    let (outer_tx, mut outer_rx) = unbounded();

    let mut session = Session::with_interpreter(
        galaxy(),
        Interpreter::with_outer_channel(outer_tx),
    )?;

    std::thread::spawn(move || {
        let intercom = Intercom::proxy();
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(async {
            while let Some(request) = outer_rx.next().await {
                match request {
                    OuterRequest::ProxySend { modulated_req, modulated_rep, } => {
                        match intercom.async_send(modulated_req).await {
                            Ok(response) => {
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

