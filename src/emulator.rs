use crate::ram::Ram;
use crate::screen::Screen;
use crate::cpu::Cpu;

pub struct Emulator {
    ram: Ram,
    screen: Screen,
    cpu: Cpu,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            ram: Ram::new(),
            screen: Screen::new(),
            cpu: Cpu::new(),
        }
    }

    pub fn cycle(&mut self) {
        self.cpu.cycle(&mut self.ram, &mut self.screen);
    }

    pub fn tick_timers(&mut self) {
        self.cpu.tick_timers();
    }

    pub fn set_key_press(&mut self, index: usize, pressed: bool) {
        self.cpu.set_key_press(index, pressed)
    }

    pub fn get_display(&self) -> &[bool] {
        self.screen.get_pixels()
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.ram.load_rom(rom);
    }
}
