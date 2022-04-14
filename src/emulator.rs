use crate::ram::{Ram, START_ADDRESS};
use crate::screen::{Screen, SCREEN_HEIGHT, SCREEN_WIDTH};
use rand::random;

const NUMBER_OF_V_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const NUMBER_OF_KEYS: usize = 16;

pub struct Emulator {
    program_counter: u16,
    ram: Ram,
    screen: Screen,
    v_registers: [u8; NUMBER_OF_V_REGISTERS],
    i_register: u16,
    stack_pointer: u16,
    stack: [u16; STACK_SIZE],
    keys_pressed: [bool; NUMBER_OF_KEYS],
    delay_timer: u8,
    sound_timer: u8,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            program_counter: START_ADDRESS,
            ram: Ram::new(),
            screen: Screen::new(),
            v_registers: [0; NUMBER_OF_V_REGISTERS],
            i_register: 0,
            stack_pointer: 0,
            stack: [0; STACK_SIZE],
            keys_pressed: [false; NUMBER_OF_KEYS],
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    pub fn cycle(&mut self) {
        //Fetch
        let op_code = self.fetch();

        //Decode & Execute
        self.decode_and_execute(op_code);
    }

    fn fetch(&mut self) -> u16 {
        let op_code = self.ram.fetch_opcode(self.program_counter as usize);
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
            (0, 0, 0xE, 0) => self.screen.clear_screen(),
            //RET
            (0, 0, 0xE, 0xE) => self.return_from_subroutine(),
            //JMP NNN
            (1, _, _, _) => self.jump_to_location(op_code & 0x0FFF),
            //CALL NNN
            (2, _, _, _) => self.call_subroutine_at_location(op_code & 0x0FFF),
            //SKIP VX == NN
            (3, _, _, _) => {
                self.skip_instruction_if_vx_equals_nn(digit2 as usize, (op_code & 0x00FF) as u8)
            }
            //SKIP VX != NN
            (4, _, _, _) => {
                self.skip_instruction_if_vx_not_equals_nn(digit2 as usize, (op_code & 0x00FF) as u8)
            }
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
            (8, _, _, 7) => {
                self.subtract_vy_with_vx_store_in_vx_with_borrow(digit2 as usize, digit3 as usize)
            }
            //VX <<= 1
            (8, _, _, 0xE) => self.bit_shift_left_vx(digit2 as usize),
            //SKIP VX != VY
            (9, _, _, 0) => {
                self.skip_instruction_if_vx_not_equals_vy(digit2 as usize, digit3 as usize)
            }
            // I = NNN
            (0xA, _, _, _) => self.set_i(op_code & 0x0FFF),
            //JMP V0 + NNN
            (0xB, _, _, _) => self.jump_to_location_plus_v0(op_code & 0x0FFf),
            //VX = rand() & NN
            (0xC, _, _, _) => {
                self.set_vx_with_random_value_and_nn(digit2 as usize, (op_code & 0x00FF) as u8)
            }
            //DRAW
            (0xD, _, _, _) => self.draw_sprite(digit2 as usize, digit3 as usize, digit4 as u8),
            //SKIP KEY PRESS
            (0xE, _, 9, 0xE) => self.skip_instruction_if_key_vx_is_pressed(digit2 as usize),
            //SKIP KEY RELEASE
            (0xE, _, 0xA, 1) => self.skip_instruction_if_key_vx_is_not_pressed(digit2 as usize),
            //VX = DT
            (0xF, _, 0, 7) => self.copy_delay_timer_into_vx(digit2 as usize),
            //WAIT KEY
            (0xF, _, 0, 0xA) => self.wait_for_key_press_and_store_key_value_in_vx(digit2 as usize),
            //DT = VX
            (0xF, _, 1, 5) => self.copy_vx_into_delay_timer(digit2 as usize),
            //ST = VX
            (0xF, _, 1, 8) => self.copy_vx_into_sound_timer(digit2 as usize),
            //I += VX
            (0xF, _, 1, 0xE) => self.add_i_with_vx(digit2 as usize),
            // I = FONT
            (0xF, _, 2, 9) => self.set_location_of_sprite_for_vx(digit2 as usize),
            // BCD
            (0xF, _, 3, 3) => self.store_bcd_of_vx_in_ram(digit2 as usize),
            //STORE V0 - VX
            (0xF, _, 5, 5) => self.write_registers_v0_to_vx_in_ram_starting_at_i(digit2 as usize),
            //LOAD V0 - VX
            (0xF, _, 6, 5) => self.read_registers_v0_to_vx_from_ram_starting_at_i(digit2 as usize),
            //should not happen
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {:#04x}", op_code),
        }
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

    fn bit_shift_left_vx(&mut self, vx: usize) {
        let most_significant_bit = (self.v_registers[vx] >> 7) & 1;

        self.v_registers[vx] <<= 1;
        self.v_registers[0xF] = most_significant_bit;
    }

    fn skip_instruction_if_vx_not_equals_vy(&mut self, vx: usize, vy: usize) {
        if self.v_registers[vx] != self.v_registers[vy] {
            self.program_counter += 2;
        }
    }

    fn set_i(&mut self, value: u16) {
        self.i_register = value;
    }

    fn jump_to_location_plus_v0(&mut self, location: u16) {
        self.program_counter = location.wrapping_add(self.v_registers[0] as u16);
    }

    fn set_vx_with_random_value_and_nn(&mut self, vx: usize, nn: u8) {
        self.v_registers[vx] = random::<u8>() & nn;
    }

    fn draw_sprite(&mut self, vx: usize, vy: usize, sprite_height: u8) {
        //start coordinates of the sprite
        let sprite_origin_x_cord = self.v_registers[vx];
        let sprite_origin_y_cord = self.v_registers[vy];

        let mut any_pixels_flipped = false;

        for sprite_row_index in 0..sprite_height {
            let sprite_row_address = self.i_register + sprite_row_index as u16;
            let sprite_pixels_in_row = self.ram.read_byte(sprite_row_address as usize);

            //Each Sprite row is 8 pixels wide
            for pixel_index in 0..8 {
                if sprite_pixels_in_row & (0b1000_0000 >> pixel_index) != 0 {
                    let x_pixel_cord = (sprite_origin_x_cord + pixel_index) as usize % SCREEN_WIDTH;
                    let y_pixel_cord =
                        (sprite_origin_y_cord + sprite_row_index) as usize % SCREEN_HEIGHT;

                    let screen_index = x_pixel_cord + y_pixel_cord * SCREEN_WIDTH;

                    any_pixels_flipped |= self.screen.get_pixel(screen_index);
                    self.screen.xor_pixel(screen_index, true);
                }
            }
        }

        self.v_registers[0xF] = if any_pixels_flipped { 1 } else { 0 };
    }

    fn skip_instruction_if_key_vx_is_pressed(&mut self, vx: usize) {
        if self.keys_pressed[self.v_registers[vx] as usize] {
            self.program_counter += 2;
        }
    }

    fn skip_instruction_if_key_vx_is_not_pressed(&mut self, vx: usize) {
        if !self.keys_pressed[self.v_registers[vx] as usize] {
            self.program_counter += 2;
        }
    }

    fn copy_delay_timer_into_vx(&mut self, vx: usize) {
        self.v_registers[vx] = self.delay_timer;
    }

    fn wait_for_key_press_and_store_key_value_in_vx(&mut self, vx: usize) {
        //cannot be implemented by loop, because the code for detecting a key press would never be executed
        //Would need async execution of op codes and detecting key presses to do it with loop
        //This solution is a bit inefficient, but simple
        let mut pressed = false;
        for i in 0..self.keys_pressed.len() {
            if self.keys_pressed[i] {
                self.v_registers[vx] = i as u8;
                pressed = true;
                break;
            }
        }

        if !pressed {
            self.program_counter -= 2;
        }
    }

    fn copy_vx_into_delay_timer(&mut self, vx: usize) {
        self.delay_timer = self.v_registers[vx];
    }

    fn copy_vx_into_sound_timer(&mut self, vx: usize) {
        self.sound_timer = self.v_registers[vx];
    }

    fn add_i_with_vx(&mut self, vx: usize) {
        self.i_register = self.i_register.wrapping_add(self.v_registers[vx] as u16);
    }

    fn set_location_of_sprite_for_vx(&mut self, vx: usize) {
        //The font sprite for the number 0 is stored at ram address 0
        //The font sprite for the number 1 is stored at ram address 5
        //...
        //The font sprite for the number F(15) is stored at ram address 75
        //Address for the font sprites can be understood from init_fontset function
        self.i_register = (self.v_registers[vx] * 5) as u16;
    }

    fn store_bcd_of_vx_in_ram(&mut self, vx: usize) {
        //BCD = Binary Coded Decimal
        //Basically Stores the digits of a u8 number as three bytes in ram, starting a i register value
        let vx_val = self.v_registers[vx];

        //Not the most efficient way of BCD
        let hundreds = vx_val / 100;
        let tenths = (vx_val / 10) % 10;
        let ones = vx_val as u8 % 10;

        self.ram.write_byte(self.i_register as usize, hundreds);
        self.ram.write_byte(self.i_register as usize + 1, tenths);
        self.ram.write_byte(self.i_register as usize + 2, ones);
    }

    fn write_registers_v0_to_vx_in_ram_starting_at_i(&mut self, vx: usize) {
        for i in 0..=vx {
            self.ram
                .write_byte(self.i_register as usize + i, self.v_registers[i]);
        }
    }

    fn read_registers_v0_to_vx_from_ram_starting_at_i(&mut self, vx: usize) {
        for i in 0..=vx {
            self.v_registers[i] = self.ram.read_byte(self.i_register as usize + i);
        }
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

    pub fn get_display(&self) -> &[bool] {
        self.screen.get_pixels()
    }

    pub fn set_key_press(&mut self, index: usize, pressed: bool) {
        self.keys_pressed[index] = pressed;
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.ram.load_rom(rom);
    }
}
