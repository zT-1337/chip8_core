pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

const START_ADDRESS: u16 = 0x200;
const RAM_SIZE: usize = 4096;
const NUMBER_OF_V_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;

pub struct Emulator {
    program_counter: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_registers: [u8; NUMBER_OF_V_REGISTERS],
    i_register: u16,
    stack_pointer: u16,
    stack: [u16; STACK_SIZE],
    delay_timer: u8,
    sound_timer: u8,
}

impl Emulator {
    pub fn new() -> Self {
        let mut new_emulator = Self {
            program_counter: START_ADDRESS,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_registers: [0; NUMBER_OF_V_REGISTERS],
            i_register: 0,
            stack_pointer: 0,
            stack: [0; STACK_SIZE],
            delay_timer: 0,
            sound_timer: 0,
        };

        new_emulator.init_fontset();

        new_emulator
    }

    pub fn reset(&mut self) {
        self.program_counter = START_ADDRESS;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_registers = [0; NUMBER_OF_V_REGISTERS];
        self.i_register = 0;
        self.stack_pointer = 0;
        self.stack = [0; STACK_SIZE];
        self.delay_timer = 0;
        self.sound_timer = 0;

        self.init_fontset();
    }

    fn init_fontset(&mut self) {
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn cycle(&mut self) {
        //Fetch
        let op_code = self.fetch();

        //Decode & Execute
        self.decode_and_execute(op_code);
    }

    fn fetch(&mut self) -> u16 {
        //Chip-8 is designed to be a big endian system
        let higher_byte = self.ram[self.program_counter as usize] as u16;
        let lower_byte = self.ram[(self.program_counter + 1) as usize] as u16;
        let op_code = (higher_byte << 8) | lower_byte;

        self.program_counter += 2;

        op_code
    }

    fn decode_and_execute(&mut self, op_code: u16) {
        let digit1 = (op_code & 0xF000) >> 12;
        let digit2 = (op_code & 0x0F00) >> 8;
        let digit3 = (op_code & 0x00F0) >> 4;
        let digit4 = op_code & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            //NOP
            (0, 0, 0, 0) => return,
            //CLS
            (0, 0, 0xE, 0) => self.clear_screen(),
            //RET
            (0, 0, 0xE, 0xE) => self.return_from_subroutine(),
            //JMP NNN
            (1, _, _, _) => self.jump_to_location(op_code & 0x0FFF),
            //CALL NNN
            (2, _, _, _) => self.call_subroutine_at_location(op_code & 0x0FFF),
            //SKIP VX == NN
            (3, _, _, _) => self.skip_instruction_if_vx_equals_nn(digit2 as usize, (op_code & 0x00FF) as u8),
            //SKIP VX != NN
            (4, _, _, _) => self.skip_instruction_if_vx_not_equals_nn(digit2 as usize, (op_code & 0x00FF) as u8),
            //SKIP VX == VY
            (5, _, _, 0) => self.skip_instruction_if_vx_equals_vy(digit2 as usize, digit3 as usize),
            //VX = NN
            (6, _, _, _) => self.set_vx(digit2 as usize, (op_code & 0x00FF) as u8),
            //VX += NN
            (7, _, _, _) => self.add_vx(digit2 as usize, (op_code & 0x00FF) as u8),
            //VX = VY
            (8, _, _, 0) => self.copy_vy_into_vx(digit2 as usize, digit3 as usize),
            //VX |= VY
            (8, _, _, 1) => self.bitwise_or_vx_with_vy(digit2 as usize, digit3 as usize),
            //VX &= VY
            (8, _, _, 2) => self.bitwise_and_vx_with_vy(digit2 as usize, digit3 as usize),
            //VX ^= VY
            (8, _, _, 3) => self.bitwise_xor_vx_with_vy(digit2 as usize, digit3 as usize),
            //VX += VY
            (8, _, _, 4) => self.add_vx_with_vy_with_carry(digit2 as usize, digit3 as usize),
            //VX -= VY
            (8, _, _, 5) => self.subtract_vx_with_vy_with_borrow(digit2 as usize, digit3 as usize),
            //Vx >>= 1
            (8, _, _, 6) => self.bit_shift_right_vx(digit2 as usize),
            //VX = VY - VX
            (8, _, _, 7) => self.subtract_vy_with_vx_store_in_vx_with_borrow(digit2 as usize, digit3 as usize),
            //VX <<= 1
            (8, _, _, 0xE) => return,
            //SKIP VX != VY
            (9, _, _, 0) => return,
            // I = NNN
            (0xA, _, _, _) => return,
            //JMP V0 + NNN
            (0xB, _, _, _) => return,
            //VX = rand() & NN
            (0xC, _, _, _) => return,
            //DRAW
            (0xD, _, _, _) => return,
            //SKIP KEY PRESS
            (0xE, _, 9, 0xE) => return,
            //SKIP KEY RELEASE
            (0xE, _, 0xA, 1) => return,
            //VX = DT
            (0xF, _, 0, 7) => return,
            //WAIT KEY
            (0xF, _, 0, 0xA) => return,
            //DT = VX
            (0xF, _, 1, 5) => return,
            //ST = VX
            (0xF, _, 1, 8) => return,
            //I += VX
            (0xF, _, 1, 0xE) => return,
            // I = FONT
            (0xF, _, 2, 9) => return,
            // BCD
            (0xF, _, 3, 3) => return,
            //STORE V0 - VX
            (0xF, _, 5, 5) => return,
            //LOAD V0 - VX
            (0xF, _, 6, 5) => return,
            //should not happen
            (_, _, _, _) => unimplemented!(
                "Unimplemented opcode: {:#04x}",
                op_code
            ),
        }
    }

    fn clear_screen(&mut self) {
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
    }

    fn return_from_subroutine(&mut self) {
        let return_address = self.pop();
        self.program_counter = return_address;
    }

    fn jump_to_location(&mut self, location: u16) {
        self.program_counter = location;
    }

    fn call_subroutine_at_location(&mut self, location: u16) {
        self.push(self.program_counter);
        self.program_counter = location;
    }

    fn skip_instruction_if_vx_equals_nn(&mut self, vx: usize, nn: u8) {
        if self.v_registers[vx] == nn {
            self.program_counter += 2;
        }
    }

    fn skip_instruction_if_vx_not_equals_nn(&mut self, vx: usize, nn: u8) {
        if self.v_registers[vx] != nn {
            self.program_counter += 2;
        }
    }

    fn skip_instruction_if_vx_equals_vy(&mut self, vx: usize, vy: usize) {
        if self.v_registers[vx] == self.v_registers[vy] {
            self.program_counter += 2;
        }
    }

    fn set_vx(&mut self, vx: usize, nn: u8) {
        self.v_registers[vx] = nn;
    }

    fn add_vx(&mut self, vx: usize, nn: u8) {
        //In case of an overflow rust would panic, while CHIP-8 would not
        self.v_registers[vx] = self.v_registers[vx].wrapping_add(nn);
    }

    fn copy_vy_into_vx(&mut self, vx: usize, vy: usize) {
        self.v_registers[vx] = self.v_registers[vy];
    }

    fn bitwise_or_vx_with_vy(&mut self, vx: usize, vy: usize) {
        self.v_registers[vx] |= self.v_registers[vy];
    }

    fn bitwise_and_vx_with_vy(&mut self, vx: usize, vy: usize) {
        self.v_registers[vx] &= self.v_registers[vy];
    }

    fn bitwise_xor_vx_with_vy(&mut self, vx: usize, vy: usize) {
        self.v_registers[vx] ^= self.v_registers[vy];
    }

    fn add_vx_with_vy_with_carry(&mut self, vx: usize, vy: usize) {
        let (result, is_overflown) = self.v_registers[vx].overflowing_add(self.v_registers[vy]);

        self.v_registers[vx] = result;
        self.v_registers[0xF] = if is_overflown { 1 } else { 0 };
    }

    fn subtract_vx_with_vy_with_borrow(&mut self, vx: usize, vy: usize) {
        let (result, is_underflown) = self.v_registers[vx].overflowing_sub(self.v_registers[vy]);

        self.v_registers[vx] = result;
        self.v_registers[0xF] = if is_underflown { 0 } else { 1 };
    }

    fn bit_shift_right_vx(&mut self, vx: usize) {
        let least_significant_bit = self.v_registers[vx] & 1;

        self.v_registers[vx] >>= 1;
        self.v_registers[0xF] = least_significant_bit;
    }

    fn subtract_vy_with_vx_store_in_vx_with_borrow(&mut self, vx: usize, vy: usize) {
        let (result, is_underflown) = self.v_registers[vy].overflowing_sub(self.v_registers[vx]);

        self.v_registers[vx] = result;
        self.v_registers[0xF] = if is_underflown { 0 } else { 1 };
    }

    fn push(&mut self, value: u16) {
        self.stack[self.stack_pointer as usize] = value;
        self.stack_pointer += 1;
    }

    fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer == 1 {
            //BEEP Sound
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
}
