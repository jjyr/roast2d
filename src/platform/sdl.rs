use anyhow::{anyhow, Result};
use glam::Vec2;
use sdl2::{
    controller::{Axis, Button, GameController},
    event::{Event, WindowEvent},
    keyboard::{Mod, Scancode},
    mouse::MouseButton,
    render::{Canvas, TextureCreator},
    video::{FullscreenType, Window, WindowContext},
    GameControllerSubsystem, Sdl,
};

use crate::{
    color::Color,
    engine::Engine,
    input::{KeyCode, KeyState},
    render::Render,
};

impl From<Color> for sdl2::pixels::Color {
    fn from(value: Color) -> Self {
        let Color { r, g, b, a } = value;
        Self::RGBA(r, g, b, a)
    }
}

pub(crate) struct ScreenBuffer {
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

struct PlatformSDL {
    sdl: Sdl,
    window: Window,
    // video_subsystem: VideoSubsystem,
    controller_subsystem: GameControllerSubsystem,
    gamepad: Option<GameController>,
    wants_to_exit: bool,
}

impl PlatformSDL {
    fn video_init(&mut self, vsync: bool) -> Result<ScreenBuffer> {
        let mut builder = self.window.clone().into_canvas();
        if vsync {
            builder = builder.present_vsync();
        }
        let canvas = builder.build().map_err(|e| anyhow!(e))?;
        let screen_buffer = ScreenBuffer::new(canvas);
        Ok(screen_buffer)
    }
    fn prepare_frame(&mut self, render: &mut Render) {
        if let Some(screen_buffer) = render.screen_buffer_mut() {
            screen_buffer.clear();
        }
    }
    fn end_frame(&mut self, render: &mut Render) {
        if let Some(screen_buffer) = render.screen_buffer_mut() {
            screen_buffer.present();
        }
    }
    fn video_cleanup(&mut self, render: &mut Render) {
        render.screen_buffer_mut().take();
    }

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
                            eprintln!("SDL open controller {err:?}");
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
                        engine.render.resize(size.into());
                    }
                }
                _ => {}
            }

            // // Window Events
            // if (ev.type == SDL_QUIT) {
            // 	wants_to_exit = true;
            // }
            // else if (
            // 	ev.type == SDL_WINDOWEVENT &&
            // 	(
            // 		ev.window.event == SDL_WINDOWEVENT_SIZE_CHANGED ||
            // 		ev.window.event == SDL_WINDOWEVENT_RESIZED
            // 	)
            // ) {
            // 	engine_resize(platform_screen_size());
            // }
        }
        Ok(())
    }

    fn find_gamepad(&mut self) {
        let num = match self.controller_subsystem.num_joysticks() {
            Ok(num) => num,
            Err(err) => {
                eprintln!("SDL joysticks error {err}");
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
                        eprintln!("SDL Open controller {err}");
                    }
                }
                break;
            }
        }
        if self.gamepad.is_none() {
            eprintln!("No game controller");
        }
    }
}

pub(crate) fn init<Setup: FnOnce(&mut Engine)>(
    title: String,
    width: u32,
    height: u32,
    vsync: bool,
    setup: Setup,
) -> Result<()> {
    let sdl_ctx = sdl2::init().map_err(|err| anyhow!(err))?;
    // SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO | SDL_INIT_JOYSTICK | SDL_INIT_GAMECONTROLLER);

    // Figure out the absolute asset and userdata paths. These may either be
    // supplied at build time through -DPATH_ASSETS=.. and -DPATH_USERDATA=..
    // or received at runtime from SDL. Note that SDL may return NULL for these.
    // We fall back to the current directory (i.e. just "") in this case.

    // char *sdl_path_assets = NULL;
    // #ifdef PATH_ASSETS
    // 	path_assets = TOSTRING(PATH_ASSETS);
    // #else
    // 	sdl_path_assets = SDL_GetBasePath();
    // 	if (sdl_path_assets) {
    // 		path_assets = sdl_path_assets;
    // 	}
    // #endif

    // char *sdl_path_userdata = NULL;
    // #ifdef PATH_USERDATA
    // 	path_userdata = TOSTRING(PATH_USERDATA);
    // #else
    // 	sdl_path_userdata = SDL_GetPrefPath(GAME_VENDOR, GAME_NAME);
    // 	if (sdl_path_userdata) {
    // 		path_userdata = sdl_path_userdata;
    // 	}
    // #endif

    // Reserve some space for concatenating the asset and userdata paths with
    // local filenames.
    // temp_path = bump_alloc(max(strlen(path_assets), strlen(path_userdata)) + 64);

    // Load gamecontrollerdb.txt if present.
    // FIXME: Should this load from userdata instead?
    // char *gcdb_path = strcat(strcpy(temp_path, path_assets), "gamecontrollerdb.txt");
    // int gcdb_res = SDL_GameControllerAddMappingsFromFile(gcdb_path);
    // if (gcdb_res < 0) {
    // 	printf("Failed to load gamecontrollerdb.txt\n");
    // }
    // else {
    // 	printf("load gamecontrollerdb.txt\n");
    // }

    let controller_subsystem = sdl_ctx.game_controller().map_err(|err| anyhow!(err))?;
    // perf_freq = SDL_GetPerformanceFrequency();

    // SDL_AudioSpec obtained_spec;
    // audio_device = SDL_OpenAudioDevice(NULL, 0, &(SDL_AudioSpec){
    // 	.freq = platform_output_samplerate,
    // 	.format = AUDIO_F32SYS,
    // 	.channels = 2,
    // 	.samples = 1024,
    // 	.callback = platform_audio_callback
    // }, &obtained_spec, 0);

    let video_subsystem = sdl_ctx.video().map_err(|err| anyhow!(err))?;
    let window = video_subsystem
        .window(&title, width, height)
        .position_centered()
        .opengl()
        .resizable()
        .allow_highdpi()
        .build()
        .map_err(|err| anyhow!(err))?;
    // window = SDL_CreateWindow(
    //     WINDOW_TITLE,
    //     SDL_WINDOWPOS_CENTERED,
    //     SDL_WINDOWPOS_CENTERED,
    //     WINDOW_WIDTH,
    //     WINDOW_HEIGHT,
    //     SDL_WINDOW_SHOWN | SDL_WINDOW_RESIZABLE | PLATFORM_WINDOW_FLAGS | SDL_WINDOW_ALLOW_HIGHDPI,
    // );

    let mut platform = PlatformSDL {
        sdl: sdl_ctx,
        window,
        // video_subsystem,
        controller_subsystem,
        gamepad: None,
        wants_to_exit: false,
    };
    platform.find_gamepad();
    let screen_buffer = platform.video_init(vsync).map_err(|err| anyhow!(err))?;

    // Init engine
    let mut engine = Engine::default();
    engine.render.set_screen_buffer(screen_buffer);
    setup(&mut engine);

    // Obtained samplerate might be different from requested
    // platform_output_samplerate = obtained_spec.freq;
    engine.init();
    engine.render.resize(platform.window.drawable_size().into());

    while !platform.wants_to_exit {
        platform
            .pump_events(&mut engine)
            .map_err(|err| anyhow!(err))?;
        platform.prepare_frame(&mut engine.render);
        engine.update();
        platform.end_frame(&mut engine.render);
    }

    engine.cleanup();
    platform.video_cleanup(&mut engine.render);

    // SDL_DestroyWindow(window);

    // close gamepad

    // if (sdl_path_assets) {
    //     SDL_free(sdl_path_assets);
    // }
    // if (sdl_path_userdata) {
    //     SDL_free(sdl_path_userdata);
    // }

    // SDL_CloseAudioDevice(audio_device);
    Ok(())
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
