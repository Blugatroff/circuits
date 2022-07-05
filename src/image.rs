use js_sys::Function;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

pub struct Image {
    image: HtmlImageElement,
}

impl std::ops::Deref for Image {
    type Target = HtmlImageElement;

    fn deref(&self) -> &Self::Target {
        &self.image
    }
}

impl Image {
    pub async fn load(path: &str) -> Result<Self, JsValue> {
        let mut promise = |resolve: Function, _: Function| {
            let image = web_sys::HtmlImageElement::new().unwrap();
            image.set_src(path);
            let onload = Closure::wrap({
                let image = image.clone();
                Box::new(move || {
                    resolve.call1(&JsValue::UNDEFINED, &image).unwrap();
                }) as Box<dyn FnMut()>
            })
            .into_js_value()
            .dyn_into()
            .unwrap();
            image.set_onload(Some(&onload));
        };
        let image = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut promise))
            .await?
            .dyn_into()?;

        Ok(Self { image })
    }
    /// data in format RGBA
    pub fn new(width: u32, height: u32, data: &[u8]) -> Self {
        assert_eq!(width as usize * height as usize * 4, data.len());
        let data = wasm_bindgen::Clamped(data);
        let data =
            web_sys::ImageData::new_with_u8_clamped_array_and_sh(data, width, height).unwrap();
        Self::from_image_data(data)
    }
    pub fn from_image_data(data: web_sys::ImageData) -> Self {
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("canvas")
            .unwrap();
        let canvas: HtmlCanvasElement = canvas.dyn_into().unwrap();
        let width = data.width();
        let height = data.height();
        canvas.set_width(width);
        canvas.set_height(height);
        let ctx: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();
        ctx.put_image_data(&data, 0.0, 0.0).unwrap();
        let image = web_sys::HtmlImageElement::new().unwrap();
        image.set_src(&canvas.to_data_url().unwrap());
        Self { image }
    }
}
