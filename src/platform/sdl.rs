use std::{collections::HashMap, time::Instant};

use anyhow::{anyhow, Result};
use glam::{UVec2, Vec2};
use sdl2::{
    controller::{Axis, Button, GameController},
    event::{Event, WindowEvent},
    keyboard::{Mod, Scancode},
    mouse::MouseButton,
    pixels::PixelFormatEnum,
    render::{Canvas, Texture, TextureCreator},
    surface::Surface,
    video::{FullscreenType, Window, WindowContext},
    GameControllerSubsystem, Sdl,
};

use crate::{
    color::Color,
    engine::Engine,
    handle::{Handle, HandleId},
    input::{KeyCode, KeyState},
    types::Rect,
};

use super::Platform;

impl From<Color> for sdl2::pixels::Color {
    fn from(value: Color) -> Self {
        let Color { r, g, b, a } = value;
        Self::RGBA(r, g, b, a)
    }
}

struct ScreenBuffer {
    pub(crate) texture_creator: TextureCreator<WindowContext>,
    pub(crate) canvas: Canvas<Window>,
}

impl ScreenBuffer {
    pub(crate) fn new(canvas: Canvas<Window>) -> Self {
        let texture_creator = canvas.texture_creator();
        Self {
            canvas,
            texture_creator,
        }
    }

    pub(crate) fn present(&mut self) {
        self.canvas.present();
    }

    pub(crate) fn clear(&mut self) {
        self.canvas.clear();
    }
}

impl From<Rect> for sdl2::rect::Rect {
    fn from(value: Rect) -> Self {
        let size = value.max - value.min;
        sdl2::rect::Rect::new(
            value.min.x.floor() as i32,
            value.min.y.floor() as i32,
            size.x.ceil() as u32,
            size.y.ceil() as u32,
        )
    }
}

pub struct SDLPlatform {
    screen_buffer: ScreenBuffer,
    textures: HashMap<u64, Texture>,
    start: Instant,
}

impl SDLPlatform {
    fn new(screen_buffer: ScreenBuffer) -> Self {
        Self {
            screen_buffer,
            textures: Default::default(),
            start: Instant::now(),
        }
    }
}

impl Platform for SDLPlatform {
    fn now(&mut self) -> f32 {
        self.start.elapsed().as_secs_f32()
    }

    fn prepare_frame(&mut self) {
        self.screen_buffer.clear();
    }

    fn end_frame(&mut self) {
        self.screen_buffer.present();
    }

    fn cleanup(&mut self) {}

    fn draw(
        &mut self,
        handle: &Handle,
        color: Color,
        src: Option<Rect>,
        dst: Rect,
        angle: Option<f32>,
        flip_x: bool,
        flip_y: bool,
    ) {
        let Some(texture) = self.textures.get_mut(&handle.id()) else {
            log::debug!("Failed to get texture {}", handle.id());
            return;
        };

        let src: Option<sdl2::rect::Rect> = src.map(Into::into);
        let dst: sdl2::rect::Rect = dst.into();
        texture.set_color_mod(color.r, color.g, color.b);

        self.screen_buffer
            .canvas
            .copy_ex(
                texture,
                src,
                dst,
                angle.unwrap_or_default().into(),
                None,
                flip_x,
                flip_y,
            )
            .unwrap();
    }

    fn create_texture(&mut self, handle: Handle, mut data: Vec<u8>, size: UVec2) {
        let UVec2 {
            x: width,
            y: height,
        } = size;
        let pitch = width * 4;
        let surface = Surface::from_data(&mut data, width, height, pitch, PixelFormatEnum::RGBA32)
            .map_err(|err| anyhow!(err))
            .unwrap();

        let texture = self
            .screen_buffer
            .texture_creator
            .create_texture_from_surface(surface)
            .unwrap();

        self.textures.insert(handle.id(), texture);
    }

    fn remove_texture(&mut self, handle_id: HandleId) {
        self.textures.remove(&handle_id);
    }

    async fn run<Setup: FnOnce(&mut Engine)>(
        title: String,
        width: u32,
        height: u32,
        vsync: bool,
        setup: Setup,
    ) -> Result<()> {
        let sdl_ctx = sdl2::init().map_err(|err| anyhow!(err))?;

        let controller_subsystem = sdl_ctx.game_controller().map_err(|err| anyhow!(err))?;
        let video_subsystem = sdl_ctx.video().map_err(|err| anyhow!(err))?;
        let window = video_subsystem
            .window(&title, width, height)
            .position_centered()
            .opengl()
            .resizable()
            .allow_highdpi()
            .build()
            .map_err(|err| anyhow!(err))?;
        let screen_buffer = {
            let mut builder = window.clone().into_canvas();
            if vsync {
                builder = builder.present_vsync();
            }
            let canvas = builder.build().map_err(|e| anyhow!(e))?;
            ScreenBuffer::new(canvas)
        };

        let mut event_handler = SDLEventHandler {
            sdl: sdl_ctx.clone(),
            window: window.clone(),
            controller_subsystem,
            gamepad: None,
            wants_to_exit: false,
        };

        event_handler.find_gamepad();

        // Init engine
        let mut engine = {
            let platform = SDLPlatform::new(screen_buffer);
            Engine::new(Box::new(platform))
        };
        setup(&mut engine);

        // Obtained samplerate might be different from requested
        // platform_output_samplerate = obtained_spec.freq;
        engine.init();
        engine
            .render
            .borrow_mut()
            .resize(event_handler.window.drawable_size().into());

        while !event_handler.wants_to_exit {
            if let Err(err) = engine.handle_assets().await {
                log::error!("Handle assets error {:?}", err);
            }
            event_handler
                .pump_events(&mut engine)
                .map_err(|err| anyhow!(err))?;
            engine.with_platform(|p| p.prepare_frame());
            engine.update();
            engine.with_platform(|p| p.end_frame());
        }

        engine.cleanup();
        engine.with_platform(|p| p.cleanup());

        Ok(())
    }
}

struct SDLEventHandler {
    sdl: Sdl,
    window: Window,
    // video_subsystem: VideoSubsystem,
    controller_subsystem: GameControllerSubsystem,
    gamepad: Option<GameController>,
    wants_to_exit: bool,
}

impl SDLEventHandler {
    fn pump_events(&mut self, engine: &mut Engine) -> Result<()> {
        let mut event_pump = self.sdl.event_pump().map_err(|err| anyhow!(err))?;
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    scancode, keymod, ..
                } if scancode.is_some_and(|code| code == Scancode::Return)
                    && (keymod.contains(Mod::LALTMOD) || keymod.contains(Mod::RALTMOD)) =>
                {
                    // Detect ALT+Enter press to toggle fullscreen
                    match self.window.fullscreen_state() {
                        FullscreenType::True => {
                            self.window
                                .set_fullscreen(FullscreenType::Off)
                                .map_err(|err| anyhow!(err))?;
                        }
                        FullscreenType::Desktop | FullscreenType::Off => {
                            self.window
                                .set_fullscreen(FullscreenType::True)
                                .map_err(|err| anyhow!(err))?;
                        }
                    }
                }
                Event::KeyUp { scancode, .. } | Event::KeyDown { scancode, .. } => {
                    // Input Keyboard
                    let state = if matches!(event, Event::KeyDown { .. }) {
                        KeyState::down()
                    } else {
                        KeyState::up()
                    };
                    let Some(code) = scancode else {
                        continue;
                    };
                    if code as i32 >= Scancode::LCtrl as i32 && code as i32 <= Scancode::RAlt as i32
                    {
                        let code = (code as i32 - Scancode::LCtrl as i32
                            + KeyCode::LeftControl as i32) as u8;
                        engine.input.set_input_state(code.into(), state);
                    } else if (code as i32 > KeyCode::Invalid as i32)
                        && ((code as i32) < (KeyCode::KeyMax as i32))
                    {
                        engine
                            .input
                            .set_input_state((code as i32 as u8).into(), state);
                    }
                }
                Event::TextInput { text, .. } => engine.input.text_input(text),
                Event::ControllerDeviceAdded { which, .. } => {
                    // Gamepads connect/disconnect
                    match self.controller_subsystem.open(which) {
                        Ok(gamepad) => {
                            self.gamepad = Some(gamepad);
                        }
                        Err(err) => {
                            log::error!("SDL open controller {err:?}");
                        }
                    }
                }
                Event::ControllerDeviceRemoved { which, .. } => {
                    if self
                        .gamepad
                        .as_ref()
                        .is_some_and(|gamepad| gamepad.instance_id() == which)
                    {
                        self.gamepad.take();
                        self.find_gamepad();
                    }
                }
                Event::ControllerButtonDown { button, .. }
                | Event::ControllerButtonUp { button, .. } => {
                    // Input Gamepad Buttons
                    let code: KeyCode = button.into();
                    if code != KeyCode::Invalid {
                        let state = if matches!(event, Event::ControllerButtonDown { .. }) {
                            KeyState::down()
                        } else {
                            KeyState::up()
                        };
                        engine.input.set_input_state(code, state);
                    }
                }
                Event::ControllerAxisMotion { axis, value, .. } => {
                    // Input Gamepad Axis
                    let code: KeyCode = axis.into();
                    let state = value as f32 / 32767.0;

                    if matches!(code, KeyCode::GamepadLTrigger | KeyCode::GamepadRTrigger) {
                        engine.input.set_input_state(code, state.into());
                    } else if state > 0.0 {
                        engine.input.set_input_state(code, 0.0.into());
                        engine
                            .input
                            .set_input_state((code as u8 + 1).into(), state.into());
                    } else {
                        engine.input.set_input_state(code, (-state).into());
                        engine
                            .input
                            .set_input_state((code as u8 + 1).into(), 0.0.into());
                    }
                }
                Event::MouseButtonDown { mouse_btn, .. }
                | Event::MouseButtonUp { mouse_btn, .. } => {
                    // Mouse buttons
                    let code = match mouse_btn {
                        MouseButton::Left => KeyCode::MouseLeft,
                        MouseButton::Middle => KeyCode::MouseMiddle,
                        MouseButton::Right => KeyCode::MouseRight,
                        _ => KeyCode::Invalid,
                    };
                    if code != KeyCode::Invalid {
                        let state = if matches!(event, Event::MouseButtonDown { .. }) {
                            KeyState::down()
                        } else {
                            KeyState::up()
                        };
                        engine.input.set_input_state(code, state);
                    }
                }
                Event::MouseWheel { y, .. } => {
                    // Mouse wheel
                    let code = if y > 0 {
                        KeyCode::MouseWheelUp
                    } else {
                        KeyCode::MouseWheelDown
                    };
                    engine.input.set_input_state(code, 1.0.into());
                    engine.input.set_input_state(code, 0.0.into());
                }
                Event::MouseMotion { x, y, .. } => {
                    // Mouse move
                    engine.input.set_mouse_pos(Vec2::new(x as f32, y as f32));
                }
                Event::Quit { .. } => {
                    self.wants_to_exit = true;
                }
                Event::Window { win_event, .. } => {
                    if matches!(
                        win_event,
                        WindowEvent::SizeChanged(..) | WindowEvent::Resized(..)
                    ) {
                        let size = self.window.drawable_size();
                        engine.render.borrow_mut().resize(size.into());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn find_gamepad(&mut self) {
        let num = match self.controller_subsystem.num_joysticks() {
            Ok(num) => num,
            Err(err) => {
                log::error!("SDL joysticks error {err}");
                return;
            }
        };
        for i in 0..num {
            if self.controller_subsystem.is_game_controller(i) {
                match self.controller_subsystem.open(i) {
                    Ok(gamepad) => {
                        self.gamepad = Some(gamepad);
                    }
                    Err(err) => {
                        log::error!("SDL Open controller {err}");
                    }
                }
                break;
            }
        }
        if self.gamepad.is_none() {
            log::warn!("No game controller");
        }
    }
}

impl From<Button> for KeyCode {
    fn from(value: Button) -> Self {
        match value {
            Button::A => KeyCode::GamepadA,
            Button::B => KeyCode::GamepadB,
            Button::X => KeyCode::GamepadX,
            Button::Y => KeyCode::GamepadY,
            Button::Back => KeyCode::GamepadSelect,
            Button::Guide => KeyCode::GamepadHome,
            Button::Start => KeyCode::GamepadStart,
            Button::LeftStick => KeyCode::GamepadLStickPress,
            Button::RightStick => KeyCode::GamepadRStickPress,
            Button::LeftShoulder => KeyCode::GamepadLShoulder,
            Button::RightShoulder => KeyCode::GamepadRShoulder,
            Button::DPadLeft => KeyCode::GamepadDpadLeft,
            Button::DPadRight => KeyCode::GamepadDpadRight,
            Button::DPadUp => KeyCode::GamepadDpadUp,
            Button::DPadDown => KeyCode::GamepadDpadDown,
            _ => KeyCode::Invalid,
        }
    }
}

impl From<Axis> for KeyCode {
    fn from(value: Axis) -> Self {
        match value {
            Axis::LeftX => KeyCode::GamepadLStickLeft,
            Axis::LeftY => KeyCode::GamepadLStickUp,
            Axis::RightX => KeyCode::GamepadRStickLeft,
            Axis::RightY => KeyCode::GamepadRStickUp,
            Axis::TriggerLeft => KeyCode::GamepadLTrigger,
            Axis::TriggerRight => KeyCode::GamepadRTrigger,
        }
    }
}
