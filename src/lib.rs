mod canvas;
mod event_loop;
mod grid;
mod image;
mod state;
mod util;
use state::State;
use wasm_bindgen::prelude::*;

#[macro_export]
macro_rules! dbg {
    ( $val:expr ) => {{
        match $val {
            tmp => {
                crate::print(format_args!(
                    "[{}:{}] {} = {:#?}",
                    file!(),
                    line!(),
                    stringify!($val),
                    &tmp
                ));
                tmp
            }
        }
    }};
}

#[wasm_bindgen]
pub async fn start() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let state = State::new().await?;
    event_loop::start(state).await;
    Ok(())
}

pub fn print(args: std::fmt::Arguments) {
    let value = JsValue::from(format!("{args}"));
    web_sys::console::log_1(&value)
}

pub struct PrintOnDrop(Box<dyn std::fmt::Debug>);

impl PrintOnDrop {
    pub fn new(v: impl std::fmt::Debug + 'static) -> Self {
        Self(Box::new(v) as Box<dyn std::fmt::Debug>)
    }
}
