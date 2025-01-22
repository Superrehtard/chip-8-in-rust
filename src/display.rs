struct Display {

}

impl Display {
    fn new() -> Display {
        Display {}
    }

    /// couple of questions
    /// why are we reading bytes from memory at i_register?
    /// why are we using the i_register as the starting address?
    /// what is the purpose of the vx and vy registers?
    /// what is the logic to get display_x and display_y?
    /// why are we doing self.display[y][x] ?
    /// and why are we checking if the display[y][x] == 1?
    /// what is the purpose of the xor operation?
    pub fn draw_sprite(&mut self, vx: u8, vy: u8, height: u8, memory: &Memory) -> bool {
        let x = vx as usize;
        let y = vy as usize;
        let mut collision = false;

        for byte in 0..height {
            let sprite = memory.read_byte(self.i_register as usize + byte as usize);
            for bit in 0..8 {
                let pixel = (sprite >> (7 - bit)) & 1;
                if pixel == 1 {
                    let display_x = (x + bit) % 64;
                    let display_y = (y + byte as usize) % 32;
                    if self.display[display_y][display_x] == 1 {
                        collision = true;
                    }
                    self.display[display_y][display_x] ^= 1;
                    print!("{} ", self.display[display_y][display_x]);
                }
            }
            println!();
        }

        collision
    }

    fn render(&self, _pixels: &Vec<u8>) {
        // render the pixels
    }
}