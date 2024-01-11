mod tests {
    use mocks::*;

    use crate::{
        hexadecimal_sprites::SEVEN,
        interpreters::{Interpreter, ProgramCounter},
        memory::Memory,
    };
    fn make_interpreter() -> Interpreter<TestDisplay, TestInput, TestTimer, TestTimer> {
        let mem = Memory::empty();
        Interpreter::new(
            mem,
            TestDisplay::new(),
            TestInput::new(),
            TestTimer::new(42),
            TestTimer::new(42),
        )
    }
    fn byte_to_bit_array(byte: u8) -> [u8; 8] {
        let mut arr = [0; 8];
        for i in 0..8 {
            arr[i] = byte >> (7 - i) & 1;
        }
        arr
    }
    #[test]
    fn test_ret() {
        let mut interpreter = make_interpreter();

        let opcode = 0x00EE;
        interpreter.stack[1] = 200;
        interpreter.registers.sp = 2;
        let pc = interpreter.execute_opcode(opcode);
        assert_eq!(pc, ProgramCounter::Jump(200));
        assert_eq!(interpreter.registers.sp, 1)
    }
    #[test]
    fn test_jpaddr() {
        let opcode = 0x1fff;
        let mut interpreter = make_interpreter();
        let pc = interpreter.execute_opcode(opcode);
        assert_eq!(pc, ProgramCounter::Jump(0xfff))
    }
    #[test]
    fn test_call_addr() {
        let opcode = 0x2f02;
        let mut interpreter = make_interpreter();
        interpreter.registers.pc = 0xf01;
        let pc = interpreter.execute_opcode(opcode);
        assert_eq!(pc, ProgramCounter::Jump(0xf02));
        let ts = interpreter.stack[interpreter.registers.sp - 1 as usize];
        assert_eq!(ts, 0xf03)
    }
    #[test]
    fn test_sevxbyte() {
        let opcode = 0x3122;
        let mut interpreter = make_interpreter();
        interpreter.registers.v[1] = 0x22;
        let pc = interpreter.execute_opcode(opcode);
        assert_eq!(pc, ProgramCounter::Skip(true))
    }
    #[test]
    fn test_snevxbyte() {
        let opcode = 0x41ff;
        let mut interpreter = make_interpreter();
        interpreter.registers.v[1] = 0x22;
        let pc = interpreter.execute_opcode(opcode);
        assert_eq!(pc, ProgramCounter::Skip(true))
    }
    #[test]
    fn test_sevxvy() {
        let opcode = 0x5120;
        let mut interpreter = make_interpreter();
        interpreter.registers.v[1] = 0x22;
        interpreter.registers.v[2] = 0x22;
        let pc = interpreter.execute_opcode(opcode);
        assert_eq!(pc, ProgramCounter::Skip(true))
    }
    #[test]
    fn test_ldvxbyte() {
        let opcode = 0x6122;
        let mut interpreter = make_interpreter();
        interpreter.execute_opcode(opcode);
        let actual = interpreter.registers.v[1];
        assert_eq!(actual, 0x22)
    }
    #[test]
    fn test_addvxvy() {
        let opcode = 0x8124;
        let mut interpreter = make_interpreter();
        interpreter.registers.v[1] = 128;
        interpreter.registers.v[2] = 129;
        interpreter.execute_opcode(opcode);
        assert_eq!(interpreter.registers.v[1], 1);
        assert_eq!(interpreter.registers.v[0xf], 1)
    }
    #[test]
    fn test_ldvxdt() {
        let opcode = 0xf107;
        let mut interpreter = make_interpreter();
        interpreter.execute_opcode(opcode);

        assert_eq!(interpreter.registers.v[1], 42)
    }
    #[test]
    fn test_drw_simple() {
        let opcode = 0xD122;
        let mut interpreter = make_interpreter();
        let i: usize = 0x200;
        interpreter.registers.i = i as u16;

        let byte = 0x9f;
        let bit_arr = byte_to_bit_array(byte);

        interpreter.memory.value[i] = 0x9f;
        interpreter.memory.value[i + 1] = 0x9f;

        let x: usize = 0;
        let y: usize = 0;
        interpreter.registers.v[1] = x as u8;
        interpreter.registers.v[2] = y as u8;

        interpreter.execute_opcode(opcode);

        let row1 = &interpreter.display.pixels.value[y][x..x + 8];
        let row2 = &interpreter.display.pixels.value[y + 1][x..x + 8];
        assert_eq!(row1, bit_arr);
        assert_eq!(row2, bit_arr);
        assert_eq!(interpreter.registers.v[0xf], 0)
    }
    #[test]
    fn test_drw_collision() {
        let opcode = 0xD122;
        let mut interpreter = make_interpreter();
        let i: usize = 0x200;
        interpreter.registers.i = i as u16;

        // first call to draw
        let byte_fw = 0x9f;
        let bit_arr_fw = byte_to_bit_array(byte_fw);

        interpreter.memory.value[i] = 0x9f;
        interpreter.memory.value[i + 1] = 0x9f;

        interpreter.registers.v[1] = 0 as u8;
        interpreter.registers.v[2] = 0 as u8;

        interpreter.execute_opcode(opcode);

        // second call to draw
        let byte_sw = 0xfc;
        let bit_arr_sw = byte_to_bit_array(byte_fw ^ byte_sw);
        interpreter.memory.value[i] = byte_sw;

        interpreter.registers.v[1] = 0 as u8;
        interpreter.registers.v[2] = 1 as u8;
        let opcode = 0xD121;
        interpreter.execute_opcode(opcode);

        let row1 = &interpreter.display.pixels.value[0][0..8];
        let row2 = &interpreter.display.pixels.value[1][0..8];
        assert_eq!(row1, bit_arr_fw);
        assert_eq!(row2, bit_arr_sw);
        assert_eq!(interpreter.registers.v[0xf], 1)
    }
    #[test]
    fn test_drw_horizontal_wrap() {
        let opcode = 0xD121;
        let mut interpreter = make_interpreter();
        let i: usize = 0x200;
        interpreter.registers.i = i as u16;

        let byte = 0x9f;
        let bit_arr = byte_to_bit_array(byte);

        interpreter.memory.value[i] = 0x9f;

        let x: usize = 62;
        let y: usize = 0;
        interpreter.registers.v[1] = x as u8;
        interpreter.registers.v[2] = y as u8;

        interpreter.execute_opcode(opcode);

        let first_row_part = &interpreter.display.pixels.value[y][0..6];
        let second_row_part = &interpreter.display.pixels.value[y][62..];
        assert_eq!(first_row_part, &bit_arr[2..]);
        assert_eq!(second_row_part, &bit_arr[0..2]);
        assert_eq!(interpreter.registers.v[0xf], 0)
    }
    #[test]
    fn test_drw_vertical_wrap() {
        let opcode = 0xD123;
        let mut interpreter = make_interpreter();
        let i: usize = 0x200;
        interpreter.registers.i = i as u16;

        let byte = 0x9f;
        let bit_arr = byte_to_bit_array(byte);

        interpreter.memory.value[i] = 0x9f;
        interpreter.memory.value[i + 1] = 0x9f;
        interpreter.memory.value[i + 2] = 0x9f;

        let x: usize = 0;
        let y: usize = 31;
        interpreter.registers.v[1] = x as u8;
        interpreter.registers.v[2] = y as u8;

        interpreter.execute_opcode(opcode);

        let first_row = &interpreter.display.pixels.value[0][..8];
        let second_row = &interpreter.display.pixels.value[1][..8];
        let last_row = &interpreter.display.pixels.value[y][..8];
        assert_eq!(first_row, bit_arr);
        assert_eq!(second_row, bit_arr);
        assert_eq!(last_row, bit_arr);
        assert_eq!(interpreter.registers.v[0xf], 0)
    }
    #[test]
    fn test_ldfvx() {
        let mut interpreter = make_interpreter();
        let opcode = 0xf129;
        interpreter.registers.v[1] = 7;
        interpreter.execute_opcode(opcode);
        let i = interpreter.registers.i as usize;
        let read_mem = &interpreter.memory.value[i..i + 5];

        assert_eq!(read_mem, SEVEN);
    }
    #[test]
    fn test_ldbvx() {
        let opcode = 0xf633;
        let mut interpreter = make_interpreter();
        interpreter.registers.v[6] = 253;
        let i: usize = 0x200;
        interpreter.registers.i = i as u16;
        interpreter.execute_opcode(opcode);
        assert_eq!(interpreter.memory.value[i], 2);
        assert_eq!(interpreter.memory.value[i + 1], 5);
        assert_eq!(interpreter.memory.value[i + 2], 3);
    }
    #[test]
    fn test_ldvxi() {
        let opcode = 0xf365;
        let mut interpreter = make_interpreter();
        let i = 0x200 as usize;
        interpreter.registers.i = i as u16;
        interpreter.memory.value[i] = 10;
        interpreter.memory.value[i + 1] = 11;
        interpreter.memory.value[i + 2] = 12;
        interpreter.memory.value[i + 3] = 13;

        interpreter.execute_opcode(opcode);
        assert_eq!(interpreter.registers.v[0], 10);
        assert_eq!(interpreter.registers.v[1], 11);
        assert_eq!(interpreter.registers.v[2], 12);
        assert_eq!(interpreter.registers.v[3], 13);
    }
    // TODO test keyboard instructions
    mod mocks {
        use sdl2::{event::Event, keyboard::Keycode};

        use crate::{
            displays::{Drawable, GamePixels},
            input_driver::EventPollable,
            keypads::Keypad,
            timers::TimerActions,
        };

        pub(crate) struct TestDisplay {
            pub(crate) pixels: GamePixels,
        }
        impl TestDisplay {
            pub(crate) fn new() -> TestDisplay {
                TestDisplay {
                    pixels: GamePixels::new(),
                }
            }
        }
        impl Drawable for TestDisplay {
            fn draw_at(&mut self, bytes: &[u8], start_coord: (usize, usize)) -> bool {
                self.pixels.fill_in_bytes(bytes, start_coord)
            }

            fn clear(&mut self) {
                self.pixels = GamePixels::new()
            }
        }

        pub(crate) struct TestInput {
            events: Vec<Keycode>,
        }
        impl TestInput {
            pub(crate) fn new() -> TestInput {
                TestInput { events: vec![] }
            }
        }
        impl EventPollable for TestInput {
            fn poll(&mut self) -> Result<crate::keypads::Keypad, ()> {
                Ok(Keypad {
                    keypad: [false; 16],
                    last_pressed: None,
                })
            }
        }

        pub(crate) struct TestTimer {
            value: u8,
        }
        impl TestTimer {
            pub(crate) fn new(value: u8) -> TestTimer {
                TestTimer { value: value }
            }
        }
        impl TimerActions for TestTimer {
            fn set(&mut self, value: u8) {
                self.value = value;
            }

            fn decrement(&mut self) {
                if self.value > 0 {
                    self.value -= 1
                }
            }

            fn get(&self) -> u8 {
                self.value
            }
        }
    }
}
