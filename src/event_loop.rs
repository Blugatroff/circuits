use js_sys::Function;
use std::{cell::RefCell, rc::Rc, str::FromStr};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Shift,
    Control,
    Space,
    Escape,
    Alt,
    Right,
    Left,
    Up,
    Down,
}

impl FromStr for Key {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "KeyA" => Self::A,
            "KeyB" => Self::B,
            "KeyC" => Self::C,
            "KeyD" => Self::D,
            "KeyE" => Self::E,
            "KeyF" => Self::F,
            "KeyG" => Self::G,
            "KeyH" => Self::H,
            "KeyI" => Self::I,
            "KeyJ" => Self::J,
            "KeyK" => Self::K,
            "KeyL" => Self::L,
            "KeyM" => Self::M,
            "KeyN" => Self::N,
            "KeyO" => Self::O,
            "KeyP" => Self::P,
            "KeyQ" => Self::Q,
            "KeyR" => Self::R,
            "KeyS" => Self::S,
            "KeyT" => Self::T,
            "KeyU" => Self::U,
            "KeyV" => Self::V,
            "KeyW" => Self::W,
            "KeyX" => Self::X,
            "KeyY" => Self::Y,
            "KeyZ" => Self::Z,
            "Digit0" => Self::Zero,
            "Digit1" => Self::One,
            "Digit2" => Self::Two,
            "Digit3" => Self::Three,
            "Digit4" => Self::Four,
            "Digit5" => Self::Five,
            "Digit6" => Self::Six,
            "Digit7" => Self::Seven,
            "Digit8" => Self::Eight,
            "Digit9" => Self::Nine,
            "ShiftLeft" => Self::Shift,
            "ControlLeft" => Self::Control,
            "Space" => Self::Space,
            "Escape" => Self::Escape,
            "AltLeft" => Self::Alt,
            "ArrowRight" => Self::Right,
            "ArrowLeft" => Self::Left,
            "ArrowUp" => Self::Up,
            "ArrowDown" => Self::Down,
            _ => return Err(()),
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Resize(u32, u32),
    MouseMove(i32, i32),
    MouseDown(MouseButton),
    MouseUp(MouseButton),
    MouseWheel(f64, f64),
    KeyDown(Key),
    KeyUp(Key),
}

#[derive(Clone, Copy, Debug)]
pub enum MouseButton {
    Primary,
    Secondary,
    Middle,
    Back,
    Forward,
}

pub enum Quit {
    Yes,
    No,
}

pub trait EventLoop {
    fn update(&mut self, delta_time: f64) -> Quit;
    fn render(&self);
    fn event(&mut self, event: Event);
}

pub async fn start(state: impl EventLoop + 'static) {
    let state = Rc::new(RefCell::new(state));
    let mut f = move |resolve: Function, _| {
        let window = web_sys::window().expect("there is a window");
        let document = window.document().expect("there is a document");

        let update = Rc::new(RefCell::new(None));
        *update.borrow_mut() = {
            let window = window.clone();
            let update = update.clone();
            let state = state.clone();
            let mut prev = js_sys::Date::new_0().get_time();
            let mut has_focus = false;
            Closure::wrap(Box::new(move || {
                let now = js_sys::Date::new_0().get_time();
                let elapsed = now - prev;
                prev = now;
                let mut state = state.borrow_mut();
                let quit = state.update(elapsed * 0.001);
                let has_focus_now = document.has_focus().unwrap();
                if has_focus_now {
                    if has_focus != has_focus_now {
                        document.set_title("Circuits");
                    }
                    state.render();
                } else {
                    if has_focus != has_focus_now {
                        document.set_title("[PAUSE] Circuits");
                    }
                }
                has_focus = has_focus_now;
                match quit {
                    Quit::Yes => {
                        *update.borrow_mut() = None;
                        window.set_onmousemove(None);
                        window.set_onresize(None);
                        window.set_onwheel(None);
                        window.set_onmousedown(None);
                        window.set_onmouseup(None);
                        window.set_onkeydown(None);
                        window.set_onkeyup(None);
                        resolve.call0(&JsValue::NULL).unwrap();
                        return;
                    }
                    Quit::No => {}
                }
                let update = update.borrow();
                let update = update.as_ref().unwrap();
                window.request_animation_frame(update).unwrap();
            }) as Box<dyn FnMut()>)
            .into_js_value()
            .dyn_into()
            .map(Some)
            .unwrap()
        };
        window
            .request_animation_frame(update.borrow().as_ref().unwrap())
            .unwrap();

        let on_resize = Closure::wrap(Box::new({
            let window = window.clone();
            let state = state.clone();
            let f = move || {
                let w = window.inner_width().unwrap().as_f64().unwrap();
                let h = window.inner_height().unwrap().as_f64().unwrap();
                state.borrow_mut().event(Event::Resize(w as u32, h as u32))
            };
            f();
            f
        }) as Box<dyn FnMut()>)
        .into_js_value()
        .dyn_into()
        .map(Some)
        .unwrap();
        window.set_onresize(on_resize.as_ref());

        let on_mouse_move = Closure::wrap(Box::new({
            let state = state.clone();
            move |e: web_sys::MouseEvent| {
                let x: i32 = e.client_x();
                let y: i32 = e.client_y();
                state.borrow_mut().event(Event::MouseMove(x, y))
            }
        }) as Box<dyn Fn(web_sys::MouseEvent)>)
        .into_js_value()
        .dyn_into()
        .map(Some)
        .unwrap();
        window.set_onmousemove(on_mouse_move.as_ref());

        let on_mouse_down = Closure::wrap(Box::new({
            let state = state.clone();
            move |e: web_sys::MouseEvent| {
                let button = e.button();
                let button = match button {
                    0 => MouseButton::Primary,
                    1 => MouseButton::Middle,
                    2 => MouseButton::Secondary,
                    3 => MouseButton::Back,
                    4 => MouseButton::Forward,
                    _ => return,
                };
                state.borrow_mut().event(Event::MouseDown(button));
            }
        }) as Box<dyn Fn(web_sys::MouseEvent)>)
        .into_js_value()
        .dyn_into()
        .map(Some)
        .unwrap();
        window.set_onmousedown(on_mouse_down.as_ref());

        let on_mouse_up = Closure::wrap(Box::new({
            let state = state.clone();
            move |e: web_sys::MouseEvent| {
                let button = e.button();
                let button = match button {
                    0 => MouseButton::Primary,
                    1 => MouseButton::Middle,
                    2 => MouseButton::Secondary,
                    3 => MouseButton::Back,
                    4 => MouseButton::Forward,
                    _ => return,
                };
                state.borrow_mut().event(Event::MouseUp(button));
            }
        }) as Box<dyn Fn(web_sys::MouseEvent)>)
        .into_js_value()
        .dyn_into()
        .map(Some)
        .unwrap();
        window.set_onmouseup(on_mouse_up.as_ref());

        let on_wheel = Closure::wrap(Box::new({
            let state = state.clone();
            move |e: web_sys::WheelEvent| {
                let dy = e.delta_y();
                let dx = e.delta_x();
                state.borrow_mut().event(Event::MouseWheel(dx, dy))
            }
        }) as Box<dyn Fn(web_sys::WheelEvent)>)
        .into_js_value()
        .dyn_into()
        .map(Some)
        .unwrap();
        window.set_onwheel(on_wheel.as_ref());

        let on_contextmenu = Closure::wrap(
            Box::new(|e: web_sys::Event| e.prevent_default()) as Box<dyn Fn(web_sys::Event)>
        )
        .into_js_value()
        .dyn_into()
        .map(Some)
        .unwrap();
        window.set_oncontextmenu(on_contextmenu.as_ref());

        let on_keydown = Closure::wrap(Box::new({
            let state = state.clone();
            move |e: web_sys::KeyboardEvent| {
                if e.repeat() {
                    return;
                }
                let key = match e.code().parse() {
                    Ok(key) => key,
                    Err(()) => {
                        crate::print(format_args!("{}", e.code()));
                        return;
                    }
                };
                state.borrow_mut().event(Event::KeyDown(key))
            }
        }) as Box<dyn Fn(web_sys::KeyboardEvent)>)
        .into_js_value()
        .dyn_into()
        .map(Some)
        .unwrap();
        window.set_onkeydown(on_keydown.as_ref());

        let on_keyup = Closure::wrap(Box::new({
            let state = state.clone();
            move |e: web_sys::KeyboardEvent| {
                let key = match e.code().parse() {
                    Ok(key) => key,
                    Err(()) => {
                        crate::print(format_args!("{}", e.code()));
                        return;
                    }
                };
                state.borrow_mut().event(Event::KeyUp(key))
            }
        }) as Box<dyn Fn(web_sys::KeyboardEvent)>)
        .into_js_value()
        .dyn_into()
        .map(Some)
        .unwrap();
        window.set_onkeyup(on_keyup.as_ref());
    };

    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut f))
        .await
        .unwrap();
}
