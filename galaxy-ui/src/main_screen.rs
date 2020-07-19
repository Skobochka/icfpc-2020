use graphics::*;

use crate::{
    Data,
    GlContext,Scene,
    controls::{
        Cursor,CursorAction,
        DrawContext,
        DrawControl,
        Panel,TextData
    },
};


pub struct MainScreen {
    menu: Panel,
    info: Panel,
    top: Panel,
    left: Panel,
    right: Panel,

    help: [TextData; 6],
    native_x: TextData,
    native_y: TextData,
    draw_x: TextData,
    draw_y: TextData,
    scene_x: TextData,
    scene_y: TextData,
    scene_z: TextData,

    pub scene: Scene,

    drawn: DrawContext,
}
impl MainScreen {
    pub fn new(data: Data, c: &DrawContext) -> MainScreen {
        let mut scr = MainScreen {
            menu: Panel::new(0.0,0.0,270.0,180.0)
                .with_color([0.75,0.0,0.0,1.0])
                .with_border_color([1.0,0.0,0.0,1.0]),
            info: Panel::new(1260.0,0.0,180.0,180.0)
                .with_color([0.75,0.0,0.0,1.0])
                .with_border_color([1.0,0.0,0.0,1.0]),
            top: Panel::new(270.0,0.0,990.0,180.0)
                .with_color([0.0,0.75,0.0,1.0])
                .with_border_color([0.0,1.0,0.0,1.0]),
            left: Panel::new(0.0,180.0,270.0,720.0)
                .with_color([0.0,0.0,0.75,1.0])
                .with_border_color([0.0,0.0,1.0,1.0]),
            right: Panel::new(1260.0,180.0,180.0,720.0)
                .with_color([0.0,0.0,0.75,1.0])
                .with_border_color([0.0,0.0,1.0,1.0]),
            
            scene: Scene::new(data,270.0,180.0,990.0,720.0,c),

            help: [TextData::new([1.0; 4],14).with_pos([8.0, 24.0]),
                   TextData::new([1.0; 4],14).with_pos([28.0, 44.0]),
                   TextData::new([1.0; 4],14).with_pos([48.0, 64.0]),
                   TextData::new([1.0; 4],14).with_pos([48.0, 84.0]),
                   TextData::new([1.0; 4],14).with_pos([28.0, 104.0]),
                   TextData::new([1.0; 4],14).with_pos([48.0, 124.0])],
            
            native_x: TextData::new([1.0; 4],14).with_pos([8.0, 24.0]),
            native_y: TextData::new([1.0; 4],14).with_pos([8.0, 44.0]),
            draw_x: TextData::new([1.0; 4],14).with_pos([8.0, 84.0]),
            draw_y: TextData::new([1.0; 4],14).with_pos([8.0, 104.0]),
            
            scene_x: TextData::new([1.0; 4],14).with_pos([8.0, 144.0]),
            scene_y: TextData::new([1.0; 4],14).with_pos([8.0, 164.0]),
            scene_z: TextData::new([1.0; 4],14).with_pos([8.0, 184.0]),

            drawn: *c,
        };
        scr.help[0].set_text("Controls:".to_string());
        scr.help[1].set_text("Map:".to_string());
        scr.help[2].set_text("Move: Vertical Scroll, Horisontal Scroll".to_string());
        scr.help[3].set_text("Zoom: Ctrl + Vertical Scroll".to_string());
        scr.help[4].set_text("Minimap:".to_string());
        scr.help[5].set_text("Relocate main map: Click".to_string());
        scr
    }
}

impl DrawControl for MainScreen {
    fn draw<'t>(&mut self, c: &DrawContext, glc: &mut GlContext<'t>) {
        self.drawn = *c;
        self.menu.draw(c,glc);
        self.info.draw(c,glc);
        self.top.draw(c,glc); {
            let mut c = *c;
            c.transform = c.transform.trans(self.top.left,self.top.top);
            for h in self.help.iter_mut() {
                h.draw(&c,glc);
            }
        }
        self.left.draw(c,glc);
        self.right.draw(c,glc); {
            let mut c = *c;
            c.transform = c.transform.trans(self.right.left,self.right.top);               
            self.native_x.draw(&c,glc);
            self.native_y.draw(&c,glc);
            self.draw_x.draw(&c,glc);
            self.draw_y.draw(&c,glc);
            self.scene_x.draw(&c,glc);
            self.scene_y.draw(&c,glc);
            self.scene_z.draw(&c,glc);
        }
        self.scene.draw(c,glc); {
            let mut c = *c;
            c.transform = c.straight(self.info.left,self.info.top,self.info.width,self.info.height);
            c.draw_state.scissor = Some(c.scissor(self.info.left,self.info.top,self.info.width,self.info.height));
            self.scene.mini_map().draw(&c,glc);
        }
    }
    fn cursor(&mut self, mut cursor: Cursor) -> CursorAction {
        self.native_x.set_text(format!("{:.3}",cursor.cursor[0]));
        self.native_y.set_text(format!("{:.3}",cursor.cursor[1]));
        cursor.transform(self.drawn.screen_back);
        self.draw_x.set_text(format!("{:.3}",cursor.cursor[0]));
        self.draw_y.set_text(format!("{:.3}",cursor.cursor[1]));
        if self.info.cursor(cursor).as_bool() {
            let transform = self.drawn.straight(self.info.left,self.info.top,self.info.width,self.info.height);
            cursor.transform(math::multiply(DrawContext::reverse(transform),self.drawn.transform));
            match self.scene.mini_map().cursor(cursor) {
                CursorAction::Click{ relative, button } => self.scene.mini_click(relative,button),
                _ => {},
            }
            return CursorAction::Processed;
        }
        if self.scene.cursor(cursor).as_bool() {
            let cur = self.scene.get_cursor();
            //self.scene_x.set_text(format!("{:.3}",self.scene.cursor.cursor[0]));
            //self.scene_y.set_text(format!("{:.3}",self.scene.cursor.cursor[1]));
            self.scene_x.set_text(format!("{:.1}",cur.cursor[0]));
            self.scene_y.set_text(format!("{:.1}",cur.cursor[1]));
            self.scene_z.set_text(format!("{:.3}",self.scene.scale));
            return CursorAction::Processed;
        }
        let mut r = false;
        r |= self.menu.cursor(cursor).as_bool();
        r |= self.top.cursor(cursor).as_bool();
        r |= self.left.cursor(cursor).as_bool();
        r |= self.right.cursor(cursor).as_bool();
        match r {
            true => CursorAction::Processed,
            false => CursorAction::None,
        }
    }
}
