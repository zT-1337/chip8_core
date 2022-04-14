pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

pub struct Screen {
    pixels: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
}

impl Screen {
    pub fn new() -> Self {
        Self {
            pixels: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
        }
    }

    pub fn clear_screen(&mut self) {
        self.pixels = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
    }

    pub fn get_pixel(&self, index: usize) -> bool {
        self.pixels[index]
    }

    pub fn xor_pixel(&mut self, index: usize, value: bool) {
        self.pixels[index] ^= value;
    }

    pub fn get_pixels(&self) -> &[bool] {
        &self.pixels
    }
}
