pub use input::Keyboard;
pub use input::Key;

pub mod input {
    use sdl2::keyboard::Keycode;
    use std::collections::HashMap;
    use strum::IntoEnumIterator;
    use strum_macros::EnumIter;
    use derivative::Derivative;

    pub struct Keyboard {
        state: HashMap<Key, bool>,
    }

    #[derive(EnumIter, Eq, Derivative)]
    #[derivative(PartialEq, Hash)]
    pub enum Key {
        Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
        A, B, C, D, E, F
    }

    impl Key {
        pub fn from_chip8_code(code: u8) -> Option<Key> {
            match code {
                0 => Some(Key::Num0),
                1 => Some(Key::Num1),
                2 => Some(Key::Num2),
                3 => Some(Key::Num3),
                4 => Some(Key::Num4),
                5 => Some(Key::Num5),
                6 => Some(Key::Num6),
                7 => Some(Key::Num7),
                8 => Some(Key::Num8),
                9 => Some(Key::Num9),
                0xa => Some(Key::A),
                0xb => Some(Key::B),
                0xc => Some(Key::C),
                0xd => Some(Key::D),
                0xe => Some(Key::E),
                0xf => Some(Key::F),
                _ => None
            }
        }

        pub fn from_keycode(code: Keycode) -> Option<Key> {
            match code {
                Keycode::Num1 => Some(Key::Num1),
                Keycode::Num2 => Some(Key::Num2),
                Keycode::Num3 => Some(Key::Num3),
                Keycode::Num4 => Some(Key::C),
                Keycode::Q => Some(Key::Num4),
                Keycode::W => Some(Key::Num5),
                Keycode::E => Some(Key::Num6),
                Keycode::R => Some(Key::D),
                Keycode::A => Some(Key::Num7),
                Keycode::S => Some(Key::Num8),
                Keycode::D => Some(Key::Num9),
                Keycode::F => Some(Key::E),
                Keycode::Z => Some(Key::A),
                Keycode::X => Some(Key::Num0),
                Keycode::C => Some(Key::B),
                Keycode::V => Some(Key::F),
                _ => None
            }
        }

        pub fn chip8_code(&self) -> u8 {
            match *self {
                Key::Num0 => 0,
                Key::Num1 => 1,
                Key::Num2 => 2,
                Key::Num3 => 3,
                Key::Num4 => 4,
                Key::Num5 => 5,
                Key::Num6 => 6,
                Key::Num7 => 7,
                Key::Num8 => 8,
                Key::Num9 => 9,
                Key::A => 0xa,
                Key::B => 0xb,
                Key::C => 0xc,
                Key::D => 0xd,
                Key::E => 0xe,
                Key::F => 0xf,
            }
        }
    }

    impl Keyboard {
        pub fn new() -> Keyboard {
            let mut kb = Keyboard { state: HashMap::new() };
            for key in Key::iter() {
                kb.state.insert(key, false);
            }
            kb
        }

        pub fn key_down(&mut self, code: Keycode) {
            if let Some(key) = Key::from_keycode(code) {
                self.state.insert(key, true);
            }
        }

        pub fn key_up(&mut self, code: Keycode) {
            if let Some(key) = Key::from_keycode(code) {
                self.state.insert(key, false);
            }
        }

        pub fn is_key_pressed(&self, code: u8) -> bool {
            match Key::from_chip8_code(code) {
                Some(key) => *self.state.get(&key).unwrap(),
                None => false,
            }
        }

        pub fn any_pressed_key(&self) -> Option<u8> {
            Some(Key::iter().find(|key| *self.state.get(key).unwrap())?.chip8_code())
        }
    }
}