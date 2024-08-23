use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver},
};

use glam::{UVec2, Vec2};
use log::Level;
use wasm_bindgen::{
    prelude::{Closure, JsCast},
    Clamped,
};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    CanvasRenderingContext2d, Document, HtmlCanvasElement, ImageData, KeyboardEvent, MouseEvent,
    Window,
};

use crate::{
    engine::Engine,
    handle::{Handle, HandleId},
    input::{KeyCode, KeyState},
};

use super::Platform;

const TARGET_FRAME_SECS: f32 = 1. / 60.;

pub struct Texture {
    canvas: HtmlCanvasElement,
}

pub struct WebPlatform {
    start: f64,
    window: Window,
    document: Document,
    buffer_canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    buf: CanvasRenderingContext2d,
    textures: HashMap<HandleId, Texture>,
}

impl WebPlatform {
    fn new(
        window: Window,
        document: Document,
        context: CanvasRenderingContext2d,
        buffer_canvas: HtmlCanvasElement,
        buf: CanvasRenderingContext2d,
    ) -> Self {
        let start = window.performance().unwrap().now();
        Self {
            window,
            document,
            context,
            buffer_canvas,
            buf,
            textures: Default::default(),
            start,
        }
    }
}

impl Platform for WebPlatform {
    fn prepare_frame(&mut self) {
        self.buf.reset();
    }

    fn end_frame(&mut self) {
        self.context
            .draw_image_with_html_canvas_element(&self.buffer_canvas, 0.0, 0.0)
            .unwrap();
    }

    fn cleanup(&mut self) {
        // Nothing todo
    }

    fn draw(
        &mut self,
        texture: &crate::handle::Handle,
        _color: crate::prelude::Color,
        pos: glam::Vec2,
        size: glam::Vec2,
        uv_offset: glam::Vec2,
        uv_size: Option<glam::Vec2>,
        _angle: f32,
        flip_x: bool,
        flip_y: bool,
    ) {
        let Some(texture) = self.textures.get(&texture.id()) else {
            log::debug!("Can't find image data");
            return;
        };
        let uv_size = uv_size.unwrap_or_else(|| {
            Vec2::new(
                texture.canvas.width() as f32,
                texture.canvas.height() as f32,
            )
        });
        let mut dx = pos.x;
        let mut dy = pos.y;
        // flip
        if flip_x {
            dx += size.x;
        }
        if flip_y {
            dy += size.y;
        }
        self.buf
            .translate(dx.round().into(), dy.round().into())
            .unwrap();
        self.buf
            .scale(if flip_x { -1. } else { 1. }, if flip_y { -1. } else { 1. })
            .unwrap();
        self.buf
            .draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &texture.canvas,
                uv_offset.x.round().into(),
                uv_offset.y.round().into(),
                uv_size.x.round().into(),
                uv_size.y.round().into(),
                0.0,
                0.0,
                size.x.round().into(),
                size.y.round().into(),
            )
            .unwrap();
        // clear transform
        self.buf
            .set_transform_with_default_dom_matrix_2d_init()
            .unwrap();
    }

    fn create_texture(&mut self, handle: Handle, data: Vec<u8>, size: glam::UVec2) {
        let canvas: HtmlCanvasElement = self
            .document
            .create_element("canvas")
            .unwrap()
            .dyn_into()
            .unwrap();
        canvas.set_width(size.x);
        canvas.set_height(size.y);
        let ctx: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();
        let data = Clamped(data.as_slice());
        let image_data = ImageData::new_with_u8_clamped_array_and_sh(data, size.x, size.y).unwrap();
        ctx.put_image_data(&image_data, 0.0, 0.0).unwrap();
        let texture = Texture { canvas };
        self.textures.insert(handle.id(), texture);
    }

    fn remove_texture(&mut self, handle_id: HandleId) {
        self.textures.remove(&handle_id);
    }

    fn now(&mut self) -> f32 {
        let now = self.window.performance().unwrap().now();
        ((now - self.start) / 1000.0) as f32
    }

    async fn run<Setup: FnOnce(&mut crate::prelude::Engine)>(
        _title: String,
        width: u32,
        height: u32,
        _vsync: bool,
        setup: Setup,
    ) -> anyhow::Result<()> {
        console_log::init_with_level(Level::Debug).unwrap();
        #[cfg(feature = "web-debug")]
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        let window = web_sys::window().expect("Get window");
        let document = window.document().expect("Get document");
        let canvas = document
            .get_element_by_id("roast-2d-canvas")
            .expect("Get canvas #roast-2d-canvas");
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .expect("cast canvas");

        let buffer_canvas: HtmlCanvasElement = document
            .create_element("canvas")
            .unwrap()
            .dyn_into()
            .unwrap();

        canvas.set_width(width);
        canvas.set_height(height);
        buffer_canvas.set_width(width);
        buffer_canvas.set_height(height);

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .expect("2d context");

        let buffer = buffer_canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .expect("2d context");

        let size = UVec2::new(canvas.scroll_width() as u32, canvas.scroll_height() as u32);

        // event listener
        let (events_sender, mut events_receiver) = channel();

        // Listen events, the closure must be lived in memory, so we define them here, otherwise we need to fight with Rust compiler
        // KeyDown
        let key_down = Closure::<dyn FnMut(_)>::new({
            let events = events_sender.clone();
            move |event: KeyboardEvent| {
                if is_ime(&event) {
                    return;
                }
                let _ = events.send(Event::KeyDown {
                    alt: event.alt_key(),
                    ctrl: event.ctrl_key(),
                    key_code: event.key_code(),
                });
            }
        });
        // KeyUp
        let key_up = Closure::<dyn FnMut(_)>::new({
            let events = events_sender.clone();
            move |event: KeyboardEvent| {
                if is_ime(&event) {
                    return;
                }
                let _ = events.send(Event::KeyUp {
                    alt: event.alt_key(),
                    ctrl: event.ctrl_key(),
                    key_code: event.key_code(),
                });
            }
        });
        // MouseDown
        let mouse_down = Closure::<dyn FnMut(_)>::new({
            let events = events_sender.clone();
            move |event: MouseEvent| {
                let _ = events.send(Event::MouseDown {
                    button: event.button(),
                });
            }
        });

        let mouse_up = Closure::<dyn FnMut(_)>::new({
            let events = events_sender.clone();
            move |event: MouseEvent| {
                let _ = events.send(Event::MouseUp {
                    button: event.button(),
                });
            }
        });

        let mouse_move = Closure::<dyn FnMut(_)>::new({
            let events = events_sender.clone();
            move |event: MouseEvent| {
                let _ = events.send(Event::MouseMove {
                    x: event.offset_x(),
                    y: event.offset_y(),
                });
            }
        });

        // listen
        canvas
            .add_event_listener_with_callback("keydown", key_down.as_ref().unchecked_ref())
            .unwrap();
        canvas
            .add_event_listener_with_callback("keyup", key_up.as_ref().unchecked_ref())
            .unwrap();
        canvas
            .add_event_listener_with_callback("mousedown", mouse_down.as_ref().unchecked_ref())
            .unwrap();
        canvas
            .add_event_listener_with_callback("mouseup", mouse_up.as_ref().unchecked_ref())
            .unwrap();
        canvas
            .add_event_listener_with_callback("mousemove", mouse_move.as_ref().unchecked_ref())
            .unwrap();

        // setup
        let mut engine = {
            let platform = WebPlatform::new(window, document, context, buffer_canvas, buffer);
            Engine::new(Box::new(platform))
        };

        setup(&mut engine);
        engine.init();

        engine.render.resize(size);

        loop {
            let frame_start = engine.platform_mut().now();
            if let Err(err) = engine.handle_assets().await {
                log::error!("Handle assets error {:?}", err);
            }
            // handle events
            handle_events(&mut engine, &mut events_receiver);
            engine.platform_mut().prepare_frame();
            engine.update();
            engine.platform_mut().end_frame();
            let frame_secs = engine.platform_mut().now() - frame_start;
            let sleep_secs = TARGET_FRAME_SECS - frame_secs;
            if sleep_secs > 0.0 {
                sleep((sleep_secs * 1000.) as i32).await.unwrap();
            }
        }
    }
}

fn sleep(ms: i32) -> JsFuture {
    JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
            .unwrap();
    }))
}

#[allow(dead_code)]
#[derive(Debug)]
enum Event {
    KeyDown {
        alt: bool,
        ctrl: bool,
        key_code: u32,
    },
    KeyUp {
        alt: bool,
        ctrl: bool,
        key_code: u32,
    },
    MouseDown {
        button: i16,
    },
    MouseUp {
        button: i16,
    },
    MouseMove {
        x: i32,
        y: i32,
    },
}

// Is keyboard event is from IME <https://developer.mozilla.org/en-US/docs/Web/API/Element/keydown_event#keydown_events_with_ime>
fn is_ime(event: &KeyboardEvent) -> bool {
    event.is_composing() || event.key_code() == 229
}

fn handle_events(engine: &mut Engine, events_receiver: &mut Receiver<Event>) {
    while let Ok(event) = events_receiver.try_recv() {
        match event {
            Event::KeyUp { key_code, .. } | Event::KeyDown { key_code, .. } => {
                // Input Keyboard
                let state = if matches!(event, Event::KeyDown { .. }) {
                    KeyState::down()
                } else {
                    KeyState::up()
                };
                let code: KeyCode;
                if key_code > 64 && key_code < 91 {
                    // KeyA ~ KeyZ
                    code = (key_code as u8 - 65 + (KeyCode::KeyA as u8)).into();
                } else if key_code > 47 && key_code < 58 {
                    // Digit 0 ~ 9
                    code = if key_code == 48 {
                        KeyCode::Digit0
                    } else {
                        ((key_code as u8 - 49) + KeyCode::Digit1 as u8).into()
                    }
                } else if key_code > 36 && key_code < 41 {
                    // Arrow
                    code = match key_code {
                        37 => KeyCode::Left,
                        38 => KeyCode::Up,
                        39 => KeyCode::Right,
                        40 => KeyCode::Down,
                        _ => KeyCode::Invalid,
                    };
                } else if key_code == 13 {
                    code = KeyCode::Return;
                } else if key_code == 27 {
                    code = KeyCode::Escape;
                } else if key_code == 8 {
                    code = KeyCode::BackSpace;
                } else if key_code == 9 {
                    code = KeyCode::Tab;
                } else if key_code == 32 {
                    code = KeyCode::Space;
                } else if key_code == 63 {
                    code = KeyCode::Minus;
                } else if key_code == 187 {
                    code = KeyCode::Equals;
                } else if key_code == 219 {
                    code = KeyCode::LeftBracket;
                } else if key_code == 221 {
                    code = KeyCode::RightBracket;
                } else if key_code == 220 {
                    code = KeyCode::BackSlash;
                } else if key_code == 163 {
                    code = KeyCode::Hash;
                } else if key_code == 59 {
                    code = KeyCode::SemiColon;
                } else if key_code == 223 {
                    code = KeyCode::Tilde;
                } else if key_code == 188 {
                    code = KeyCode::Comma;
                } else if key_code == 190 {
                    code = KeyCode::Period;
                } else if key_code == 191 {
                    code = KeyCode::Slash;
                } else if key_code == 20 {
                    code = KeyCode::CapsLock;
                } else {
                    code = KeyCode::Invalid;
                }
                if code != KeyCode::Invalid {
                    engine.input.set_input_state(code, state);
                }
            }
            Event::MouseDown { button, .. } | Event::MouseUp { button, .. } => {
                // Mouse buttons
                let code = match button {
                    0 => KeyCode::MouseLeft,
                    1 => KeyCode::MouseMiddle,
                    2 => KeyCode::MouseRight,
                    _ => KeyCode::Invalid,
                };
                if code != KeyCode::Invalid {
                    let state = if matches!(event, Event::MouseDown { .. }) {
                        KeyState::down()
                    } else {
                        KeyState::up()
                    };
                    engine.input.set_input_state(code, state);
                }
            }
            Event::MouseMove { x, y, .. } => {
                // Mouse move
                engine.input.set_mouse_pos(Vec2::new(x as f32, y as f32));
            }
        }
    }
}
