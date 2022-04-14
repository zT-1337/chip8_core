const RAM_SIZE: usize = 4096;
pub const START_ADDRESS: u16 = 0x200;

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

pub struct Ram {
    memory: [u8; RAM_SIZE],
}

impl Ram {
    pub fn new() -> Self {
        let mut new_ram = Self {
            memory: [0; RAM_SIZE],
        };

        new_ram.write_fontset();
        new_ram
    }

    fn write_fontset(&mut self) {
        self.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn fetch_opcode(&self, index: usize) -> u16 {
        //Chip-8 is designed to be a big endian system
        //Each opcode is two bytes long
        let higher_byte = self.memory[index] as u16;
        let lower_byte = self.memory[(index + 1) as usize] as u16;

        (higher_byte << 8) | lower_byte
    }

    pub fn read_byte(&self, index: usize) -> u8 {
        self.memory[index]
    }

    pub fn write_byte(&mut self, index: usize, value: u8) {
        self.memory[index] = value;
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        let start = START_ADDRESS as usize;
        let end = start + rom.len();
        self.memory[start..end].copy_from_slice(rom);
    }
}
