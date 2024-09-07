use std::{
    borrow::Cow,
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
    color::{Color, WHITE},
    engine::Engine,
    handle::{Handle, HandleId},
    input::{KeyCode, KeyState},
    types::Rect,
    world::World,
};

use super::Platform;

pub struct Texture {
    canvas: HtmlCanvasElement,
}

impl Texture {
    fn tint_color(&self, color: Color, document: &Document) -> Cow<HtmlCanvasElement> {
        if color == WHITE {
            return Cow::Borrowed(&self.canvas);
        }

        // new canvas
        let canvas: HtmlCanvasElement = document
            .create_element("canvas")
            .unwrap()
            .dyn_into()
            .unwrap();
        canvas.set_width(self.canvas.width());
        canvas.set_height(self.canvas.height());

        let ctx: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        let origin_ctx: CanvasRenderingContext2d = self
            .canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();
        let image_data = origin_ctx
            .get_image_data(
                0.0,
                0.0,
                self.canvas.width() as f64,
                self.canvas.height() as f64,
            )
            .unwrap();
        let mut data = image_data.data();

        for i in (0..data.len()).step_by(4) {
            data[i] = ((data[i] as u32 * color.r as u32) / 255) as u8; // R
            data[i + 1] = ((data[i + 1] as u32 * color.g as u32) / 255) as u8; // G
            data[i + 2] = ((data[i + 2] as u32 * color.b as u32) / 255) as u8; // B
        }

        let image_data = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(data.as_ref()),
            canvas.width(),
            canvas.height(),
        )
        .unwrap();
        ctx.put_image_data(&image_data, 0.0, 0.0).unwrap();
        Cow::Owned(canvas)
    }
}

pub struct WebPlatform {
    start: f64,
    device_pixel_ratio: f64,
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
        device_pixel_ratio: f64,
    ) -> Self {
        let start = window.performance().unwrap().now();

        Self {
            window,
            device_pixel_ratio,
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
        // Clear canvas
        self.context.clear_rect(
            0.0,
            0.0,
            self.buffer_canvas.width() as f64,
            self.buffer_canvas.height() as f64,
        );
        self.buf.clear_rect(
            0.0,
            0.0,
            self.buffer_canvas.width() as f64,
            self.buffer_canvas.height() as f64,
        );
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
        texture: &Handle,
        color: Color,
        src: Option<Rect>,
        dst: Rect,
        angle: Option<f32>,
        flip_x: bool,
        flip_y: bool,
    ) {
        let Some(texture) = self.textures.get(&texture.id()) else {
            log::debug!("Can't find image data");
            return;
        };
        let canvas = texture.tint_color(color, &self.document);
        let uv_size = match src.as_ref() {
            Some(src) => src.max - src.min,
            None => Vec2::new(canvas.width() as f32, canvas.height() as f32),
        };

        let uv_offset = src.map(|src| src.min).unwrap_or_else(|| Vec2::ZERO);

        let pos = dst.min;
        let size = dst.max - dst.min;

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
            .translate(
                ((dx as f64).round() * self.device_pixel_ratio).ceil(),
                ((dy as f64).round() * self.device_pixel_ratio).ceil(),
            )
            .unwrap();
        self.buf
            .scale(
                if flip_x { -1.0 } else { 1.0 },
                if flip_y { -1.0 } else { 1.0 },
            )
            .unwrap();

        let dw = ((size.x as f64).round() * self.device_pixel_ratio).ceil();
        let dh = ((size.y as f64).round() * self.device_pixel_ratio).ceil();

        // rotate by center with angle degree in counter clock-wise
        if let Some(angle) = angle {
            let dw_hf = (dw * 0.5).round();
            let dh_hf = (dh * 0.5).round();
            // move to center
            self.buf.translate(dw_hf, dh_hf).unwrap();
            // rotate counter clockwise
            self.buf.rotate(-angle as f64).unwrap();
            self.buf.translate(-dw_hf, -dh_hf).unwrap();
        }
        self.buf
            .draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &canvas,
                uv_offset.x.round().into(),
                uv_offset.y.round().into(),
                uv_size.x.round().into(),
                uv_size.y.round().into(),
                0.0,
                0.0,
                dw,
                dh,
            )
            .unwrap();
        // clear transform
        self.buf.reset_transform().unwrap();
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

    async fn run<Setup: FnOnce(&mut Engine, &mut World)>(
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

        // Fix High Dpi blur <https://gist.github.com/callumlocke/cc258a193839691f60dd>
        let dpr = window.device_pixel_ratio();
        let width_px = (width as f64 * dpr).floor() as u32;
        let height_px = (height as f64 * dpr).floor() as u32;
        canvas.set_width(width_px);
        canvas.set_height(height_px);
        canvas
            .style()
            .set_property("width", &format!("{}px", width))
            .unwrap();
        canvas
            .style()
            .set_property("height", &format!("{}px", height))
            .unwrap();
        buffer_canvas.set_width(width_px);
        buffer_canvas.set_height(height_px);
        buffer_canvas
            .style()
            .set_property("width", &format!("{}px", width))
            .unwrap();
        buffer_canvas
            .style()
            .set_property("height", &format!("{}px", height))
            .unwrap();

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
            // disable image smoothing to fix weired tile gaps
            buffer.set_image_smoothing_enabled(false);
            let platform = WebPlatform::new(window, document, context, buffer_canvas, buffer, dpr);
            Engine::new(Box::new(platform))
        };

        engine.init(setup);

        engine.render.borrow_mut().resize(size);

        loop {
            if let Err(err) = engine.handle_assets().await {
                log::error!("Handle assets error {:?}", err);
            }
            // handle events
            handle_events(&mut engine, &mut events_receiver);
            engine.with_platform(|p| p.prepare_frame());
            engine.update();
            engine.with_platform(|p| p.end_frame());
            wait_next_frame().await.unwrap();
        }
    }
}

fn wait_next_frame() -> JsFuture {
    JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .request_animation_frame(&resolve)
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
