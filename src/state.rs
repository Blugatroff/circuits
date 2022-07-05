use crate::{
    canvas::Canvas,
    event_loop::{Event, EventLoop, Key, MouseButton, Quit},
    grid::{Cell, Direction, Grid},
    image::Image,
    PrintOnDrop,
};
use glam::DVec2;
use std::collections::HashMap;
use wasm_bindgen::JsValue;

struct KeyState<K> {
    keys: HashMap<K, bool>,
}

impl<K> Default for KeyState<K> {
    fn default() -> Self {
        Self {
            keys: Default::default(),
        }
    }
}

impl<K: std::hash::Hash + std::cmp::Eq> std::ops::Index<K> for KeyState<K> {
    type Output = bool;

    fn index(&self, index: K) -> &Self::Output {
        self.keys.get(&index).unwrap_or(&false)
    }
}

impl<K: std::hash::Hash + std::cmp::Eq> KeyState<K> {
    pub fn set(&mut self, key: K, state: bool) {
        self.keys.insert(key, state);
    }
}

const CELLS: &'static [Cell] = &[
    Cell::And {
        active: false,
        direction: Direction::Up,
    },
    Cell::Cable {
        active: false,
        direction: Direction::Up,
    },
    Cell::Not {
        active: false,
        direction: Direction::Up,
    },
    Cell::Tee {
        active: false,
        direction: Direction::Up,
    },
    Cell::Point {
        active: true,
        marked: 0,
    },
];

struct Rect {
    pos: DVec2,
    size: DVec2,
}

impl Rect {
    pub fn contains(&self, p: DVec2) -> bool {
        self.pos.x <= p.x
            && self.pos.y <= p.y
            && p.x <= self.pos.x + self.size.x
            && p.y <= self.pos.y + self.size.y
    }
}

struct CellRectsIter {
    start: DVec2,
    current: usize,
}

impl CellRectsIter {
    pub fn new(screen_size: DVec2, size: f64) -> Self {
        let start = DVec2::new(
            screen_size.x / 2.0 - CELLS.len() as f64 * size / 2.0,
            screen_size.y - size,
        );
        Self { start, current: 0 }
    }
}

impl Iterator for CellRectsIter {
    type Item = (Rect, &'static Cell);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == CELLS.len() {
            return None;
        }
        let pos = self.start + DVec2::new(self.current as f64 * 50.0, 0.0);
        let size = DVec2::new(50.0, 50.0);
        let cell = &CELLS[self.current];
        self.current += 1;
        Some((Rect { pos, size }, cell))
    }
}

pub struct State {
    canvas: Canvas,
    mouse_pos: DVec2,
    cable_image: Image,
    and_image: Image,
    not_image: Image,
    tee_image: Image,
    red_image: Image,
    dark_red_image: Image,
    ui_backgroud_image: Image,
    tick: u64,
    cam: CamRect,
    panning: Option<DVec2>,
    grid: Box<Grid>,
    hand: Cell,
    screen_size: DVec2,
    quit: bool,
    keys: KeyState<Key>,
    time: f64,
    running: bool,
}

impl State {
    pub async fn new() -> Result<Self, JsValue> {
        let canvas = Canvas::new();
        let cable_image = Image::load("/assets/line.png").await?;
        let and_image = Image::load("/assets/and.png").await?;
        let not_image = Image::load("/assets/not.png").await?;
        let tee_image = Image::load("/assets/tee.png").await?;
        let red_image = Image::new(1, 1, &[0xFF, 0, 0, 0xFF]);
        let dark_red_image = Image::new(1, 1, &[0x88, 0, 0, 0xFF]);
        let ui_backgroud_image = Image::new(1, 1, &[0x59, 0x3b, 0x13, 0xFF]);

        let mouse_pos = DVec2::new(0.0, 0.0);
        let tick = 0;
        let cam = CamRect {
            pos: DVec2::new(0.0, 0.0),
            size: DVec2::new(10.0, 10.0),
            screen: DVec2::new(0.0, 0.0),
        };
        let panning = None;
        let screen_size = canvas.size();
        let keys = KeyState::default();
        let time = 0.0;
        let running = false;
        let location = web_sys::window()
            .expect("there is a window")
            .document()
            .expect("there is a document")
            .location()
            .unwrap();
        let search = location.search().unwrap();
        let grid = web_sys::UrlSearchParams::new_with_str(&search)
            .unwrap()
            .get("save")
            .and_then(|str| {
                crate::dbg!(&str);
                Grid::deserialize(str.as_bytes().into_iter().map(|b| *b - 33)).ok()
            })
            .unwrap_or_else(|| Grid::new(10, 10));
        let grid = Box::new(grid);
        Ok(Self {
            canvas,
            cable_image,
            and_image,
            not_image,
            tee_image,
            mouse_pos,
            tick,
            cam,
            panning,
            grid,
            hand: Cell::Empty,
            screen_size,
            quit: false,
            red_image,
            dark_red_image,
            ui_backgroud_image,
            keys,
            time,
            running,
        })
    }
}

impl Drop for PrintOnDrop {
    fn drop(&mut self) {
        crate::print(format_args!("{:?}", &self.0))
    }
}

impl EventLoop for State {
    fn update(&mut self, dt: f64) -> Quit {
        self.time += dt;
        if self.quit {
            return Quit::Yes;
        }
        if let Some(o) = self.panning {
            self.cam
                .make_equal(o, self.cam.screen_to_world(self.mouse_pos))
        }
        if self.keys[Key::E] {
            self.make_active(|_| true)
        }
        if self.running {
            self.grid.simulate();
        }
        self.tick += 1;
        Quit::No
    }
    fn event(&mut self, event: Event) {
        match event {
            Event::Resize(width, height) => {
                self.canvas.resize(width, height);
                self.screen_size = DVec2::new(width as f64, height as f64);
                self.cam.screen = DVec2::new(
                    self.screen_size.min_element(),
                    self.screen_size.min_element(),
                );
            }
            Event::MouseMove(x, y) => self.mouse_pos = DVec2::new(x as f64, y as f64),
            Event::MouseDown(button) => match button {
                MouseButton::Secondary => {
                    self.panning = Some(self.cam.screen_to_world(self.mouse_pos));
                }
                MouseButton::Primary => {
                    let mut rects = CellRectsIter::new(self.screen_size, 50.0);
                    let clicked_on_hotbar = loop {
                        let (rect, cell) = match rects.next() {
                            Some(rect) => rect,
                            None => break false,
                        };
                        if rect.contains(self.mouse_pos) {
                            self.hand = *cell;
                            break true;
                        }
                    };
                    if !clicked_on_hotbar {
                        let mouse_pos = self.cam.screen_to_world(self.mouse_pos).floor().as_uvec2();
                        if mouse_pos.x < self.grid.width() as u32
                            && mouse_pos.y < self.grid.height() as u32
                        {
                            if self.keys[Key::Shift] {
                                self.grid[[mouse_pos.x as usize, mouse_pos.y as usize]] =
                                    Cell::Empty;
                            } else if self.hand != Cell::Empty {
                                self.grid[[mouse_pos.x as usize, mouse_pos.y as usize]] = self.hand;
                            }
                        }
                    }
                }
                _ => {}
            },
            Event::MouseUp(button) => match button {
                MouseButton::Secondary => {
                    self.panning = None;
                }
                MouseButton::Middle => {
                    self.canvas.detach();
                    self.quit = true;
                }
                _ => {}
            },
            Event::MouseWheel(_, dy) => {
                let o = self.cam.screen_to_world(self.mouse_pos);
                if dy > 0.0 {
                    self.cam.size *= 1.1;
                } else if dy < 0.0 {
                    self.cam.size *= 0.9;
                }
                self.cam.size.x = self.cam.size.y;
                self.cam
                    .make_equal(o, self.cam.screen_to_world(self.mouse_pos));
            }
            Event::KeyDown(key) => {
                self.keys.set(key, true);
                match key {
                    Key::R => {
                        if self.hand != Cell::Empty {
                            if let Some(direction) = self.hand.direction_mut() {
                                *direction = direction.rotate_cw();
                            }
                        } else {
                            let mouse_pos =
                                self.cam.screen_to_world(self.mouse_pos).floor().as_uvec2();
                            if mouse_pos.x < self.grid.width() as u32
                                && mouse_pos.y < self.grid.height() as u32
                            {
                                self.grid[[mouse_pos.x as usize, mouse_pos.y as usize]].rotate();
                            }
                        }
                    }
                    Key::E => {
                        if self.hand != Cell::Empty {
                            self.hand.set(!self.hand.is_active())
                        } else {
                            self.make_active(|b| !b);
                        }
                    }
                    Key::G => {
                        crate::dbg!(&self.grid.marker);
                    }
                    Key::Q => {
                        if self.hand == Cell::Empty {
                            let mouse_pos =
                                self.cam.screen_to_world(self.mouse_pos).floor().as_uvec2();
                            self.hand = *self
                                .grid
                                .get(mouse_pos.x as usize, mouse_pos.y as usize)
                                .unwrap_or(&Cell::Empty);
                        } else {
                            self.hand = Cell::Empty;
                        }
                    }
                    Key::Right => {
                        self.grid.simulate();
                    }
                    Key::Space => {
                        self.running = !self.running;
                    }
                    Key::S => {
                        let save =
                            String::from_utf8(self.grid.serialize().map(|b| b + 33).collect())
                                .unwrap();
                        let params = web_sys::UrlSearchParams::new().unwrap();
                        params.set("save", &save);
                        web_sys::window()
                            .unwrap()
                            .document()
                            .unwrap()
                            .location()
                            .unwrap()
                            .set_search(&String::from(params.to_string()))
                            .unwrap();
                        let save = String::from(js_sys::encode_uri_component(&save));
                        wasm_bindgen_futures::spawn_local(async move {
                            wasm_bindgen_futures::JsFuture::from(
                                web_sys::window()
                                    .unwrap()
                                    .navigator()
                                    .clipboard()
                                    .unwrap()
                                    .write_text(&save),
                            )
                            .await
                            .unwrap();
                            crate::print(format_args!("{}", save));
                        });
                    }
                    _ => {}
                }
            }
            Event::KeyUp(key) => {
                self.keys.set(key, false);
            }
        }
    }
    fn render(&self) {
        self.canvas.set_image_smoothing_enabled(false);
        let w = self.canvas.width() as f64;
        let h = self.canvas.height() as f64;
        let black = JsValue::from("#111111");
        self.canvas.set_fill_style(&black);
        self.canvas.fill_rect(0.0, 0.0, w, h);

        for x in 0..10 + 1 {
            let x = x as f64;
            let p1 = self.cam.world_to_screen(DVec2::new(x, 0.0));
            let p2 = self.cam.world_to_screen(DVec2::new(x, 10.0));
            self.canvas.draw_line("green", p1, p2);
        }
        for y in 0..10 + 1 {
            let y = y as f64;
            let p1 = self.cam.world_to_screen(DVec2::new(0.0, y));
            let p2 = self.cam.world_to_screen(DVec2::new(10.0, y));
            self.canvas.draw_line("green", p1, p2);
        }

        let block_size = self.cam.screen / self.cam.size;
        for ([x, y], cell) in &*self.grid {
            let pos = DVec2::new(x as f64, y as f64);
            let pos = self.cam.world_to_screen(pos);
            if *cell != Cell::Empty {
                if cell.is_active() {
                    self.canvas
                        .draw_image(&self.red_image, pos, block_size, 0.0, 1.0);
                }
            }
            self.draw_cell(cell, pos, block_size, 1.0);
        }

        let red = JsValue::from("red");
        self.canvas.set_fill_style(&red);
        self.canvas.begin_path();
        self.canvas.fill();
        let pos = self
            .cam
            .world_to_screen(self.cam.screen_to_world(self.mouse_pos).floor());
        if self.hand.is_active() {
            self.canvas
                .draw_image(&self.red_image, pos, block_size, 0.0, 0.5);
        }
        self.draw_cell(&self.hand, pos, block_size, 1.0);
        let start = DVec2::new(
            self.screen_size.x / 2.0 - CELLS.len() as f64 * 50.0 / 2.0,
            self.screen_size.y - 50.0,
        );
        self.canvas.draw_image(
            &self.ui_backgroud_image,
            start - DVec2::new(5.0, 5.0),
            DVec2::new(CELLS.len() as f64 * 50.0 + 10.0, 60.0),
            0.0,
            1.0,
        );
        for (i, cell) in CELLS.into_iter().enumerate() {
            let pos = start + DVec2::new(i as f64 * 50.0, 0.0);
            self.draw_cell(cell, pos, DVec2::new(50.0, 50.0), 1.0);
        }
    }
}

impl State {
    fn draw_cell(&self, cell: &Cell, pos: DVec2, size: DVec2, alpha: f64) {
        match cell {
            Cell::Empty => {}
            Cell::Cable { direction, .. } => {
                self.canvas
                    .draw_image(&self.cable_image, pos, size, direction.angle(), alpha)
            }
            Cell::And { direction, .. } => {
                self.canvas
                    .draw_image(&self.and_image, pos, size, direction.angle(), alpha)
            }
            Cell::Not { direction, .. } => {
                self.canvas
                    .draw_image(&self.not_image, pos, size, direction.angle(), alpha)
            }
            Cell::Tee { direction, .. } => {
                self.canvas
                    .draw_image(&self.tee_image, pos, size, direction.angle(), alpha)
            }
            Cell::Point { active, .. } => match active {
                true => self.canvas.draw_image(&self.red_image, pos, size, 0.0, 1.0),
                false => self
                    .canvas
                    .draw_image(&self.dark_red_image, pos, size, 0.0, 1.0),
            },
        };
    }
    fn make_active(&mut self, f: impl Fn(bool) -> bool) {
        let mouse_pos = self.cam.screen_to_world(self.mouse_pos).floor().as_uvec2();
        if mouse_pos.x < self.grid.width() as u32 && mouse_pos.y < self.grid.height() as u32 {
            let cell = &mut self.grid[[mouse_pos.x as usize, mouse_pos.y as usize]];
            match &*cell {
                Cell::Point { .. } => return,
                _ => {}
            }
            cell.set(f(cell.is_active()));
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct CamRect {
    pub pos: DVec2,
    pub size: DVec2,
    pub screen: DVec2,
}

impl CamRect {
    pub fn screen_to_world(&self, pos: DVec2) -> DVec2 {
        pos / self.screen * self.size + self.pos
    }
    pub fn world_to_screen(&self, pos: DVec2) -> DVec2 {
        (pos - self.pos) / self.size * self.screen
    }
    pub fn make_equal(&mut self, o: DVec2, m: DVec2) {
        self.pos += o - m;
    }
}
