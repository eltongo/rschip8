pub use cpu::CPU;

pub mod cpu {
    use std::{fs::File, io::Read, convert::TryInto};
    use rand::Rng;
    use crate::emulator::{Chip8Result, input::Keyboard, ui};

    const LOAD_ADDRESS: usize = 0x200;
    const MEMORY_SIZE: usize = 4096;
    const MAX_PROGRAM_SIZE: usize = MEMORY_SIZE - LOAD_ADDRESS;
    const MAX_STACK_DEPTH: usize = 16;

    const SPRITE_SIZE: u16 = 5;
    const SPRITES: [u8; 80] = [
        0xf0, 0x90, 0x90, 0x90, 0xf0,
        0x20, 0x60, 0x20, 0x20, 0x70,
        0xf0, 0x10, 0xf0, 0x80, 0xf0,
        0xf0, 0x10, 0xf0, 0x10, 0xf0,
        0x90, 0x90, 0xf0, 0x10, 0x10,
        0xf0, 0x80, 0xf0, 0x10, 0xf0,
        0xf0, 0x80, 0xf0, 0x90, 0xf0,
        0xf0, 0x10, 0x20, 0x40, 0x40,
        0xf0, 0x90, 0xf0, 0x90, 0xf0,
        0xf0, 0x90, 0xf0, 0x10, 0xf0,
        0xf0, 0x90, 0xf0, 0x90, 0x90,
        0xe0, 0x90, 0xe0, 0x90, 0xe0,
        0xf0, 0x80, 0x80, 0x80, 0xf0,
        0xe0, 0x90, 0x90, 0x90, 0xe0,
        0xf0, 0x80, 0xf0, 0x80, 0xf0,
        0xf0, 0x80, 0xf0, 0x80, 0x80,
    ];

    pub struct CPU {
        pc: usize,
        memory: [u8; MEMORY_SIZE],
        registers: [u8; 16],
        i_register: u16,
        delay_register: u8,
        sound_register: u8,
        stack_pointer: usize,
        stack: [u16; MAX_STACK_DEPTH],
    }

    impl CPU {
        pub fn from_file(filename: &str) -> Chip8Result<CPU> {
            let mut file = File::open(filename)?;
            let mut buffer = vec![0; LOAD_ADDRESS];
            for (i, byte) in SPRITES.iter().enumerate() {
                buffer[i] = *byte;
            }
            file.read_to_end(&mut buffer)?;

            if buffer.len() == LOAD_ADDRESS {
                Err(format!("{} is empty", filename).into())
            } else if buffer.len() > MEMORY_SIZE {
                Err(format!("Tried to load {} bytes, which is more than the allowed maximum of {} bytes",
                    buffer.len(), MAX_PROGRAM_SIZE).into())
            } else {
                for _ in buffer.len()..MEMORY_SIZE {
                    buffer.push(0);
                }
                Ok(CPU::new(buffer.try_into().unwrap()))
            }
        }

        fn new(memory: [u8; MEMORY_SIZE]) -> CPU {
            CPU {
                pc: LOAD_ADDRESS,
                memory,
                registers: [0; 16],
                i_register: 0,
                delay_register: 0,
                sound_register: 0,
                stack_pointer: 0,
                stack: [0; MAX_STACK_DEPTH],
            }
        }

        fn jump(&mut self, addr: usize) -> Chip8Result<()> {
            self.pc = addr;
            Ok(())
        }

        fn call(&mut self, addr: usize) -> Chip8Result<()> {
            if self.stack_pointer >= MAX_STACK_DEPTH {
                return Err("Stack overflow".into());
            }

            self.stack[self.stack_pointer] = self.pc as u16 + 2;
            self.stack_pointer += 1;

            self.jump(addr)
        }

        fn ret(&mut self) -> Chip8Result<()> {
            if self.stack_pointer == 0 {
                return Err("Attempting return but call stack is empty".into());
            }

            self.stack_pointer -= 1;
            self.jump(self.stack[self.stack_pointer] as usize)
        }

        fn increment_pc(&mut self) {
            self.pc += 2;
        }

        fn skip_reg_imm_eq(&mut self, register: u8, byte: u8) -> Chip8Result<()> {
            if self.registers[register as usize] == byte {
                self.increment_pc();
            }
            self.increment_pc();
            Ok(())
        }

        fn skip_reg_imm_neq(&mut self, register: u8, byte: u8) -> Chip8Result<()> {
            if self.registers[register as usize] != byte {
                self.increment_pc();
            }
            self.increment_pc();
            Ok(())
        }

        fn skip_reg_reg_eq(&mut self, register1: u8, register2: u8) -> Chip8Result<()> {
            if self.registers[register1 as usize] == self.registers[register2 as usize] {
                self.increment_pc();
            }
            self.increment_pc();
            Ok(())
        }

        fn skip_reg_reg_neq(&mut self, register1: u8, register2: u8) -> Chip8Result<()> {
            if self.registers[register1 as usize] != self.registers[register2 as usize] {
                self.increment_pc();
            }
            self.increment_pc();
            Ok(())
        }

        fn load_imm(&mut self, register: u8, byte: u8) -> Chip8Result<()> {
            self.registers[register as usize] = byte;
            self.increment_pc();
            Ok(())
        }

        fn add_imm(&mut self, register: u8, byte: u8) -> Chip8Result<()> {
            let (result, _) = byte.overflowing_add(self.registers[register as usize]);
            self.registers[register as usize] = result;
            self.increment_pc();
            Ok(())
        }

        fn load_reg(&mut self, register1: u8, register2: u8) -> Chip8Result<()> {
            self.registers[register1 as usize] = self.registers[register2 as usize];
            self.increment_pc();
            Ok(())
        }

        fn or_reg(&mut self, register1: u8, register2: u8) -> Chip8Result<()> {
            self.registers[register1 as usize] |= self.registers[register2 as usize];
            self.increment_pc();
            Ok(())
        }

        fn and_reg(&mut self, register1: u8, register2: u8) -> Chip8Result<()> {
            self.registers[register1 as usize] &= self.registers[register2 as usize];
            self.increment_pc();
            Ok(())
        }

        fn xor_reg(&mut self, register1: u8, register2: u8) -> Chip8Result<()> {
            self.registers[register1 as usize] ^= self.registers[register2 as usize];
            self.increment_pc();
            Ok(())
        }

        fn add_reg(&mut self, register1: u8, register2: u8) -> Chip8Result<()> {
            let (result, overflow) =
                self.registers[register1 as usize].overflowing_add(self.registers[register2 as usize]);
            self.registers[register1 as usize] = result;
            self.registers[0xf] = overflow as u8;
            self.increment_pc();
            Ok(())
        }

        fn sub_reg_with_dest(&mut self, register1: u8, register2: u8, dest: u8) -> Chip8Result<()> {
            let (result, overflow) =
                self.registers[register1 as usize].overflowing_sub(self.registers[register2 as usize]);
            self.registers[dest as usize] = result;
            self.registers[0xf] = !overflow as u8;
            self.increment_pc();
            Ok(())
        }

        fn sub_reg(&mut self, register1: u8, register2: u8) -> Chip8Result<()> {
            self.sub_reg_with_dest(register1, register2, register1)
        }

        fn subn_reg(&mut self, register1: u8, register2: u8) -> Chip8Result<()> {
            self.sub_reg_with_dest(register2, register1, register1)
        }

        fn shr_reg(&mut self, register: u8) -> Chip8Result<()> {
            self.registers[0xf] = self.registers[register as usize] & 0x1;
            self.registers[register as usize] >>= 1;
            self.increment_pc();
            Ok(())
        }

        fn shl_reg(&mut self, register: u8) -> Chip8Result<()> {
            self.registers[0xf] = self.registers[register as usize] >> 7;
            self.registers[register as usize] <<= 1;
            self.increment_pc();
            Ok(())
        }

        fn set_i(&mut self, addr: u16) -> Chip8Result<()> {
            self.i_register = addr;
            self.increment_pc();
            Ok(())
        }

        fn jump_v0(&mut self, addr: usize) -> Chip8Result<()> {
            self.jump(self.registers[0] as usize + addr)
        }

        fn load_and_rnd_imm(&mut self, register: u8, byte: u8) -> Chip8Result<()> {
            let random_byte = rand::thread_rng().gen_range(0..=255);
            self.registers[register as usize] = random_byte & byte;
            self.increment_pc();
            Ok(())
        }

        fn draw_byte(&mut self, x: usize, y: usize, mut byte: u8, display_buffer: &mut ui::DisplayBuffer) -> bool {
            let mut collided = false;
            byte = byte.reverse_bits();
            for bit in 0..8 {
                let i = y % ui::HEIGHT as usize;
                let j = (x + bit) % ui::WIDTH as usize;
                let prev = display_buffer.buffer[i][j];

                display_buffer.buffer[i][j] = (prev as u8 ^ (byte & 1)) != 0;

                if prev && !display_buffer.buffer[i][j] {
                    collided = true;
                }

                byte >>= 1;
            }

            collided
        }

        fn draw_sprite(&mut self, register1: u8, register2: u8, bytes: u8, display_buffer: &mut ui::DisplayBuffer) -> Chip8Result<()> {
            if self.i_register as usize + bytes as usize > MEMORY_SIZE {
                return Err(
                    format!(
                        "Cannot read {} byte sprite starting from 0x{:0x}", bytes, self.i_register
                    ).into()
                );
            }

            let x = self.registers[register1 as usize] as usize;
            let mut y = self.registers[register2 as usize] as usize;
            let i = self.i_register as usize;

            let bytes: Vec<u8> = self.memory[i..i + bytes as usize].iter().cloned().collect();

            let mut collided = false;

            for byte in bytes {
                if self.draw_byte(x, y, byte, display_buffer) {
                    collided = true;
                }
                y += 1;
            }

            self.registers[0xf] = collided as u8;
            display_buffer.is_dirty = true;
            self.increment_pc();

            Ok(())
        }

        fn skip_keydown(&mut self, register: u8, keyboard: &Keyboard) -> Chip8Result<()> {
            if keyboard.is_key_pressed(self.registers[register as usize]) {
                self.increment_pc();
            }
            self.increment_pc();
            Ok(())
        }

        fn skip_not_keydown(&mut self, register: u8, keyboard: &Keyboard) -> Chip8Result<()> {
            if !keyboard.is_key_pressed(self.registers[register as usize]) {
                self.increment_pc();
            }
            self.increment_pc();
            Ok(())
        }


        fn load_delay_timer(&mut self, register: u8) -> Chip8Result<()> {
            self.registers[register as usize] = self.delay_register;
            self.increment_pc();
            Ok(())
        }

        fn set_delay_timer(&mut self, register: u8) -> Chip8Result<()> {
            self.delay_register = self.registers[register as usize];
            self.increment_pc();
            Ok(())
        }

        fn set_sound_timer(&mut self, register: u8) -> Chip8Result<()> {
            self.sound_register = self.registers[register as usize];
            self.increment_pc();
            Ok(())
        }

        fn wait_keypress(&mut self, register: u8, keyboard: &Keyboard) -> Chip8Result<()> {
            if let Some(code) = keyboard.any_pressed_key() {
                self.registers[register as usize] = code;
                self.increment_pc();
            }
            Ok(())
        }

        fn add_i_reg(&mut self, register: u8) -> Chip8Result<()> {
            self.i_register = self.i_register.overflowing_add(self.registers[register as usize] as u16).0;
            self.increment_pc();
            Ok(())
        }

        fn load_sprite_address(&mut self, register: u8) -> Chip8Result<()> {
            let digit = self.registers[register as usize] as u16;
            self.i_register = digit * SPRITE_SIZE;
            self.increment_pc();
            Ok(())
        }

        fn store_bcd_representation(&mut self, register: u8) -> Chip8Result<()> {
            let i = self.i_register as usize;
            if i >= MEMORY_SIZE - 2 {
                return Err(
                    format!("Cannot store BCD representation at address 0x{:0x}", i).into()
                );
            }
            let mut value = self.registers[register as usize];
            self.memory[i + 2] = value % 10;
            value /= 10;
            self.memory[i + 1] = value % 10;
            value /= 10;
            self.memory[i] = value % 10;

            self.increment_pc();
            Ok(())
        }

        fn store_registers(&mut self, last_register: u8) -> Chip8Result<()> {
            let i = self.i_register as usize;
            let last_register = last_register as usize;

            if i + last_register >= MEMORY_SIZE {
                return Err(
                    format!("Cannot store {} registers starting at address 0x{:0x}",
                        last_register + 1, i).into()
                );
            }

            for r in 0..=last_register {
                self.memory[i + r] = self.registers[r];
            }

            self.increment_pc();
            Ok(())
        }

        fn read_registers(&mut self, last_register: u8) -> Chip8Result<()> {
            let i = self.i_register as usize;
            let last_register = last_register as usize;

            if i + last_register >= MEMORY_SIZE {
                return Err(
                    format!("Cannot read {} registers starting from address 0x{:0x}",
                        last_register + 1, i).into()
                );
            }

            for r in 0..=last_register {
                self.registers[r] = self.memory[i + r];
            }

            self.increment_pc();
            Ok(())
        }

        fn clear_screen(&mut self, display_buffer: &mut ui::DisplayBuffer) -> Chip8Result<()> {
            for i in 0..display_buffer.buffer.len() {
                for j in 0..display_buffer.buffer[i].len() {
                    display_buffer.buffer[i][j] = false;
                }
            }
            display_buffer.is_dirty = true;
            self.increment_pc();
            Ok(())
        }

        fn noop(&mut self) ->Chip8Result<()> {
            self.increment_pc();
            Ok(())
        }

        pub fn tick(&mut self, keyboard: &Keyboard, display_buffer: &mut ui::DisplayBuffer, decrement_timers: bool) -> Chip8Result<()> {
            if self.pc + 1 >= MEMORY_SIZE {
                return Err(
                    format!("PC out of bounds: 0x{:0x}", self.pc).into()
                )
            }

            if decrement_timers {
                if self.delay_register > 0 { self.delay_register -= 1; }
                if self.sound_register > 0 { self.sound_register -= 1; }
            }

            let high = self.memory[self.pc];
            let low = self.memory[self.pc + 1];
            let a = (high & 0xf0) >> 4;
            let x = high & 0xf;
            let y = (low & 0xf0) >> 4;
            let b = low & 0xf;
            let nnn = (((high as usize) & 0xf) << 8) | low as usize;

            match (a, x, y, b) {
                (0, 0, 0xe, 0) => self.clear_screen(display_buffer),
                (0, 0, 0xe, 0xe) => self.ret(),
                (0, _, _, _) => self.noop(),
                (1, _, _, _) => self.jump(nnn),
                (2, _, _, _) => self.call(nnn),
                (3, _, _, _) => self.skip_reg_imm_eq(x, low),
                (4, _, _, _) => self.skip_reg_imm_neq(x, low),
                (5, _, _, 0) => self.skip_reg_reg_eq(x, y),
                (6, _, _, _) => self.load_imm(x, low),
                (7, _, _, _) => self.add_imm(x, low),
                (8, _, _, 0) => self.load_reg(x, y),
                (8, _, _, 1) => self.or_reg(x, y),
                (8, _, _, 2) => self.and_reg(x, y),
                (8, _, _, 3) => self.xor_reg(x, y),
                (8, _, _, 4) => self.add_reg(x, y),
                (8, _, _, 5) => self.sub_reg(x, y),
                (8, _, _, 6) => self.shr_reg(x),
                (8, _, _, 7) => self.subn_reg(x, y),
                (8, _, _, 0xe) => self.shl_reg(x),
                (9, _, _, 0) => self.skip_reg_reg_neq(x, y),
                (0xa, _, _, _) => self.set_i(nnn as u16),
                (0xb, _, _, _) => self.jump_v0(nnn),
                (0xc, _, _, _) => self.load_and_rnd_imm(x, low),
                (0xd, _, _, _) => self.draw_sprite(x, y, b, display_buffer),
                (0xe, _, 9, 0xe) => self.skip_keydown(x, keyboard),
                (0xe, _, 0xa, 1) => self.skip_not_keydown(x, keyboard),
                (0xf, _, 0, 7) => self.load_delay_timer(x),
                (0xf, _, 0, 0xa) => self.wait_keypress(x, keyboard),
                (0xf, _, 1, 5) => self.set_delay_timer(x),
                (0xf, _, 1, 8) => self.set_sound_timer(x),
                (0xf, _, 1, 0xe) => self.add_i_reg(x),
                (0xf, _, 2, 9) => self.load_sprite_address(x),
                (0xf, _, 3, 3) => self.store_bcd_representation(x),
                (0xf, _, 5, 5) => self.store_registers(x),
                (0xf, _, 6, 5) => self.read_registers(x),
                _ => Err(
                    format!(
                        "Invalid instruction: 0x{:0x}, PC=0x{:0x}\n\t{:0x} {:0x} {:0x} {:0x}",
                        ((high as u32) << 8) | low as u32,
                        self.pc,
                        a, x, y, b
                    ).into()
                )
            }
        }
    }
}