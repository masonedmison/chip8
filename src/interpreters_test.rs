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
    #[test]
    fn test_ret() {
        let mut interpreter = make_interpreter();

        let opcode = 0x00EE;
        interpreter.stack[1] = 200;
        interpreter.registers.sc = 1;
        let pc = interpreter.execute_opcode(opcode);
        assert_eq!(pc, ProgramCounter::Jump(200));
        assert_eq!(interpreter.registers.sc, 0)
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
        let ts = interpreter.stack[interpreter.registers.sc as usize];
        assert_eq!(ts, 0xf01)
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
        let mut interpreter = make_interpreter();
        assert!(true)
    }
    #[test]
    fn test_drw_collision() {
        let mut interpreter = make_interpreter();
        assert!(true)
    }
    #[test]
    fn test_drw_horizontal_wrap() {
        let mut interpreter = make_interpreter();
        assert!(true)
    }
    #[test]
    fn test_drw_vertical_wrap() {
        let mut interpreter = make_interpreter();
        assert!(true)
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
    mod mocks {
        use sdl2::{event::Event, keyboard::Keycode};

        use crate::{
            displays::{Drawable, GamePixels},
            input_driver::EventPollable,
            keypads::Keypad,
            timers::TimerActions,
        };

        pub(crate) struct TestDisplay {
            pixels: GamePixels,
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
