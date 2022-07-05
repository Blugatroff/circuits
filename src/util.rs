use wasm_bindgen::{prelude::Closure, JsCast};

fn window() -> web_sys::Window {
    web_sys::window().expect("there is a window")
}

pub struct Timeout(i32);

impl Timeout {
    pub fn new(timeout: i32, f: impl FnOnce() + 'static) -> Self {
        let f = Closure::once(Box::new(f) as Box<dyn FnOnce()>);
        let f = f.into_js_value();
        window()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                f.as_ref().dyn_ref().unwrap(),
                timeout,
            )
            .map(Timeout)
            .expect("setTimeout works")
    }
    pub fn clear(&self) {
        window().clear_timeout_with_handle(self.0)
    }
}

pub struct Interval(i32);

impl Interval {
    #[must_use]
    pub fn new(period: i32, f: impl FnMut() + 'static) -> Self {
        let f = Closure::wrap(Box::new(f) as Box<dyn FnMut()>);
        let f = f.into_js_value();
        window()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                f.as_ref().dyn_ref().unwrap(),
                period,
            )
            .map(Interval)
            .expect("setInterval works")
    }
    pub fn leak(self) {
        std::mem::forget(self);
    }
}

impl Drop for Interval {
    fn drop(&mut self) {
        window().clear_interval_with_handle(self.0)
    }
}

pub struct DebugOnDrop(Option<Box<dyn FnOnce()>>);

impl DebugOnDrop {
    pub fn new(f: impl FnOnce() + 'static) -> Self {
        Self(Some(Box::new(f) as Box<dyn FnOnce()>))
    }
}

impl Drop for DebugOnDrop {
    fn drop(&mut self) {
        self.0.take().unwrap()()
    }
}
