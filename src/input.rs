use std::collections::{HashMap, HashSet};

use glam::Vec2;

const INPUT_DEADZONE: f32 = 0.1;
// const INPUT_DEADZONE_CAPTURE: f32 = 0.5;

/// KeyCode
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum KeyCode {
    Invalid = 0,
    KeyA = 4,
    KeyB = 5,
    KeyC = 6,
    KeyD = 7,
    KeyE = 8,
    KeyF = 9,
    KeyG = 10,
    KeyH = 11,
    KeyI = 12,
    KeyJ = 13,
    KeyK = 14,
    KeyL = 15,
    KeyM = 16,
    KeyN = 17,
    KeyO = 18,
    KeyP = 19,
    KeyQ = 20,
    KeyR = 21,
    KeyS = 22,
    KeyT = 23,
    KeyU = 24,
    KeyV = 25,
    KeyW = 26,
    KeyX = 27,
    KeyY = 28,
    KeyZ = 29,
    Digit1 = 30,
    Digit2 = 31,
    Digit3 = 32,
    Digit4 = 33,
    Digit5 = 34,
    Digit6 = 35,
    Digit7 = 36,
    Digit8 = 37,
    Digit9 = 38,
    Digit0 = 39,
    Return = 40,
    Escape = 41,
    BackSpace = 42,
    Tab = 43,
    Space = 44,
    Minus = 45,
    Equals = 46,
    LeftBracket = 47,
    RightBracket = 48,
    BackSlash = 49,
    Hash = 50,
    SemiColon = 51,
    Apostrophe = 52,
    Tilde = 53,
    Comma = 54,
    Period = 55,
    Slash = 56,
    CapsLock = 57,
    F1 = 58,
    F2 = 59,
    F3 = 60,
    F4 = 61,
    F5 = 62,
    F6 = 63,
    F7 = 64,
    F8 = 65,
    F9 = 66,
    F10 = 67,
    F11 = 68,
    F12 = 69,
    PrintScreen = 70,
    ScrollLock = 71,
    Pause = 72,
    Insert = 73,
    Home = 74,
    PageUp = 75,
    Delete = 76,
    End = 77,
    PageDown = 78,
    Right = 79,
    Left = 80,
    Down = 81,
    Up = 82,
    NumLock = 83,
    NumpadDiv = 84,
    NumpadMulti = 85,
    NumpadMinus = 86,
    NumpadPlus = 87,
    NumpadEnter = 88,
    Numpad1 = 89,
    Numpad2 = 90,
    Numpad3 = 91,
    Numpad4 = 92,
    Numpad5 = 93,
    Numpad6 = 94,
    Numpad7 = 95,
    Numpad8 = 96,
    Numpad9 = 97,
    Numpad0 = 98,
    NumpadPeriod = 99,

    LeftControl = 100,
    LeftShift = 101,
    LeftAlt = 102,
    LeftGui = 103,
    RightControl = 104,
    RightShift = 105,
    RightAlt = 106,

    Max = 107,

    GamepadA = 108,
    GamepadY = 109,
    GamepadB = 110,
    GamepadX = 111,
    GamepadLShoulder = 112,
    GamepadRShoulder = 113,
    GamepadLTrigger = 114,
    GamepadRTrigger = 115,
    GamepadSelect = 116,
    GamepadStart = 117,
    GamepadLStickPress = 118,
    GamepadRStickPress = 119,
    GamepadDpadUp = 120,
    GamepadDpadDown = 121,
    GamepadDpadLeft = 122,
    GamepadDpadRight = 123,
    GamepadHome = 124,
    GamepadLStickUp = 125,
    GamepadLStickDown = 126,
    GamepadLStickLeft = 127,
    GamepadLStickRight = 128,
    GamepadRStickUp = 129,
    GamepadRStickDown = 130,
    GamepadRStickLeft = 131,
    GamepadRStickRight = 132,

    MouseLeft = 134,
    MouseMiddle = 135,
    MouseRight = 136,
    MouseWheelUp = 137,
    MouseWheelDown = 138,

    KeyMax = 139,
}

impl From<u8> for KeyCode {
    fn from(value: u8) -> Self {
        use KeyCode::*;

        match value {
            4 => KeyA,
            5 => KeyB,
            6 => KeyC,
            7 => KeyD,
            8 => KeyE,
            9 => KeyF,
            10 => KeyG,
            11 => KeyH,
            12 => KeyI,
            13 => KeyJ,
            14 => KeyK,
            15 => KeyL,
            16 => KeyM,
            17 => KeyN,
            18 => KeyO,
            19 => KeyP,
            20 => KeyQ,
            21 => KeyR,
            22 => KeyS,
            23 => KeyT,
            24 => KeyU,
            25 => KeyV,
            26 => KeyW,
            27 => KeyX,
            28 => KeyY,
            29 => KeyZ,
            30 => Digit1,
            31 => Digit2,
            32 => Digit3,
            33 => Digit4,
            34 => Digit5,
            35 => Digit6,
            36 => Digit7,
            37 => Digit8,
            38 => Digit9,
            39 => Digit0,
            40 => Return,
            41 => Escape,
            42 => BackSpace,
            43 => Tab,
            44 => Space,
            45 => Minus,
            46 => Equals,
            47 => LeftBracket,
            48 => RightBracket,
            49 => BackSlash,
            50 => Hash,
            51 => SemiColon,
            52 => Apostrophe,
            53 => Tilde,
            54 => Comma,
            55 => Period,
            56 => Slash,
            57 => CapsLock,
            58 => F1,
            59 => F2,
            60 => F3,
            61 => F4,
            62 => F5,
            63 => F6,
            64 => F7,
            65 => F8,
            66 => F9,
            67 => F10,
            68 => F11,
            69 => F12,
            70 => PrintScreen,
            71 => ScrollLock,
            72 => Pause,
            73 => Insert,
            74 => Home,
            75 => PageUp,
            76 => Delete,
            77 => End,
            78 => PageDown,
            79 => Right,
            80 => Left,
            81 => Down,
            82 => Up,
            83 => NumLock,
            84 => NumpadDiv,
            85 => NumpadMulti,
            86 => NumpadMinus,
            87 => NumpadPlus,
            88 => NumpadEnter,
            89 => Numpad1,
            90 => Numpad2,
            91 => Numpad3,
            92 => Numpad4,
            93 => Numpad5,
            94 => Numpad6,
            95 => Numpad7,
            96 => Numpad8,
            97 => Numpad9,
            98 => Numpad0,
            99 => NumpadPeriod,

            100 => LeftControl,
            101 => LeftShift,
            102 => LeftAlt,
            103 => LeftGui,
            104 => RightControl,
            105 => RightShift,
            106 => RightAlt,

            107 => Max,

            108 => GamepadA,
            109 => GamepadY,
            110 => GamepadB,
            111 => GamepadX,
            112 => GamepadLShoulder,
            113 => GamepadRShoulder,
            114 => GamepadLTrigger,
            115 => GamepadRTrigger,
            116 => GamepadSelect,
            117 => GamepadStart,
            118 => GamepadLStickPress,
            119 => GamepadRStickPress,
            120 => GamepadDpadUp,
            121 => GamepadDpadDown,
            122 => GamepadDpadLeft,
            123 => GamepadDpadRight,
            124 => GamepadHome,
            125 => GamepadLStickUp,
            126 => GamepadLStickDown,
            127 => GamepadLStickLeft,
            128 => GamepadLStickRight,
            129 => GamepadRStickUp,
            130 => GamepadRStickDown,
            131 => GamepadRStickLeft,
            132 => GamepadRStickRight,

            134 => MouseLeft,
            135 => MouseMiddle,
            136 => MouseRight,
            137 => MouseWheelUp,
            138 => MouseWheelDown,

            139 => KeyMax,
            _ => Invalid,
        }
    }
}

/// ActionId represent a player's action
#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct ActionId(pub u8);

/// KeyState
#[derive(Default, Debug, Clone, Copy)]
pub struct KeyState(pub f32);

impl KeyState {
    /// Build KetState from a float
    pub fn new(state: f32) -> Self {
        let state = if state > INPUT_DEADZONE { state } else { 0.0 };
        Self(state)
    }

    /// Down state
    pub fn down() -> Self {
        Self(1.0)
    }

    /// Up state
    pub fn up() -> Self {
        Self(0.0)
    }

    /// Key is up
    pub fn is_up(&self) -> bool {
        self.0 == 0.0
    }

    /// Key is down
    pub fn is_down(&self) -> bool {
        self.0 != 0.0
    }
}

impl From<f32> for KeyState {
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

/// Manage input state
#[derive(Default)]
pub struct InputState {
    bindings: HashMap<KeyCode, ActionId>,
    expected: HashMap<ActionId, KeyCode>,
    actions_state: HashMap<ActionId, KeyState>,
    actions_pressed: HashSet<ActionId>,
    actions_released: HashSet<ActionId>,
    mouse: Vec2,
}
impl InputState {
    pub(crate) fn set_input_state(&mut self, key: KeyCode, state: KeyState) {
        if key == KeyCode::Invalid {
            eprintln!("Invalid input button");
            return;
        }

        let Some(action) = self.bindings.get(&key) else {
            return;
        };

        let expected = self.expected.get(action);
        if expected.is_none() || expected.is_some_and(|e| *e == key) {
            if state.is_down()
                && self
                    .actions_state
                    .get(action)
                    .cloned()
                    .unwrap_or_default()
                    .is_up()
            {
                self.actions_pressed.insert(*action);
                self.expected.insert(*action, key);
            } else if state.is_up()
                && self
                    .actions_state
                    .get(action)
                    .cloned()
                    .unwrap_or_default()
                    .is_down()
            {
                self.actions_released.insert(*action);
                self.expected.remove(action);
            }
            self.actions_state.insert(*action, state);
        }
    }

    pub(crate) fn set_mouse_pos(&mut self, pos: Vec2) {
        self.mouse = pos;
    }

    /// Bind a keycode to an action
    pub fn bind(&mut self, code: KeyCode, action: ActionId) {
        if code == KeyCode::Invalid {
            eprintln!("bind: Invalid key code");
            return;
        }
        self.actions_state.insert(action, KeyState::default());
        self.bindings.insert(code, action);
    }

    /// Get action of a keycode
    pub fn action_for_keycode(&self, code: KeyCode) -> Option<&ActionId> {
        self.bindings.get(&code)
    }

    /// Unbind a KeyCode
    pub fn unbind(&mut self, code: KeyCode) {
        self.bindings.remove(&code);
    }

    /// Unbind all KeyCode
    pub fn unbind_all(&mut self) {
        self.bindings.clear();
    }

    /// Get key state
    pub fn get_state(&self, action: &ActionId) -> Option<&KeyState> {
        self.actions_state.get(action)
    }

    /// Key is pressed
    pub fn pressed(&self, action: &ActionId) -> bool {
        self.get_state(action).is_some_and(|s| s.is_down())
    }

    /// Key is released
    pub fn released(&self, action: &ActionId) -> bool {
        self.get_state(action).map(|s| s.is_up()).unwrap_or(true)
    }

    /// Key is just pressed
    pub fn just_pressed(&self, action: &ActionId) -> bool {
        self.actions_pressed.contains(action)
    }

    /// Key is just released
    pub fn just_released(&self, action: &ActionId) -> bool {
        self.actions_released.contains(action)
    }

    /// Get mouse pos
    pub fn get_mouse_pos(&self) -> Vec2 {
        self.mouse
    }

    /// Clear state
    pub fn clear(&mut self) {
        self.actions_pressed.clear();
        self.actions_released.clear();
    }

    pub fn text_input(&self, _text: String) {
        // eprintln!("Input receive text: {text}")
    }
}
