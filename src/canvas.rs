use crate::image::Image;
use glam::DVec2;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub struct Canvas {
    ctx: CanvasRenderingContext2d,
    canvas: HtmlCanvasElement,
    width: u32,
    height: u32,
}

impl std::ops::Deref for Canvas {
    type Target = CanvasRenderingContext2d;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl Canvas {
    pub fn new() -> Self {
        let window = web_sys::window().expect("there is a window");
        let document = window.document().expect("there is a document");
        let body = document.body().expect("there is a body");
        let canvas = document
            .create_element("canvas")
            .expect("i can create a canvas element");
        body.append_child(&canvas)
            .expect("canvas element can be child of body");
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into()
            .expect("canvas element is actually canvas");
        let ctx: web_sys::CanvasRenderingContext2d = canvas
            .get_context("2d")
            .expect("a 2d canvas context can be created")
            .expect("there is a 2d canvas context")
            .dyn_into()
            .expect("rendering context object is of right type");
        let width = canvas.width();
        let height = canvas.height();
        Self {
            ctx,
            canvas,
            width,
            height,
        }
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn size(&self) -> DVec2 {
        DVec2::new(self.width as f64, self.height as f64)
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.canvas.set_width(self.width);
        self.canvas.set_height(self.height);
    }
    pub fn draw_image(&self, image: &Image, pos: DVec2, size: DVec2, angle: f64, alpha: f64) {
        self.ctx.save();
        self.ctx
            .translate(pos.x + size.x * 0.5, pos.y + size.y * 0.5)
            .unwrap();
        self.ctx.rotate(angle).unwrap();
        self.ctx.set_global_alpha(alpha);
        self.ctx
            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                image,
                0.0,
                0.0,
                image.width() as f64,
                image.height() as f64,
                size.x * -0.5,
                size.y * -0.5,
                size.x,
                size.y,
            )
            .unwrap();
        self.ctx.restore();
    }
    pub fn draw_line(&self, color: &'static str, p1: DVec2, p2: DVec2) {
        self.ctx.begin_path();
        let color = JsValue::from(color);
        self.ctx.set_stroke_style(&color);
        self.ctx.move_to(p1.x, p1.y);
        self.ctx.line_to(p2.x, p2.y);
        self.ctx.stroke();
    }
    pub fn detach(&mut self) {
        self.canvas.remove();
    }
}
