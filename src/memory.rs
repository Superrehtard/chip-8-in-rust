pub struct Memory {
    data: [u8; 4096],
}

impl Memory {
    pub fn new() -> Memory {
        let mut memory = Memory { data: [0; 4096] };

        let sprites: [[u8; 5]; 16] = [
            [0xF0, 0x90, 0x90, 0x90, 0xF0], // 0
            [0x20, 0x60, 0x20, 0x20, 0x70], // 1
            [0xF0, 0x10, 0xF0, 0x80, 0xF0], // 2
            [0xF0, 0x10, 0xF0, 0x10, 0xF0], // 3
            [0x90, 0x90, 0xF0, 0x10, 0x10], // 4
            [0xF0, 0x80, 0xF0, 0x10, 0xF0], // 5
            [0xF0, 0x80, 0xF0, 0x90, 0xF0], // 6
            [0xF0, 0x10, 0x20, 0x40, 0x40], // 7
            [0xF0, 0x90, 0xF0, 0x90, 0xF0], // 8
            [0xF0, 0x90, 0xF0, 0x10, 0xF0], // 9
            [0xF0, 0x90, 0xF0, 0x90, 0x90], // A
            [0xE0, 0x90, 0xE0, 0x90, 0xE0], // B
            [0xF0, 0x80, 0x80, 0x80, 0xF0], // C
            [0xE0, 0x90, 0x90, 0x90, 0xE0], // D
            [0xF0, 0x80, 0xF0, 0x80, 0xF0], // E
            [0xF0, 0x80, 0xF0, 0x80, 0x80], // F
        ];

        for (i, &sprite) in sprites.iter().enumerate() {
            memory.load_program(&sprite, i * 5);
        }

        memory
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Load a program into memory starting at specified address
    /// self.data[start_address..start_address + program.len()].copy_from_slice(program);
    pub fn load_program(&mut self, program: &[u8], start_address: usize) {
        for (i, &byte) in program.iter().enumerate() {
            self.data[start_address + i] = byte;
        }
    }

    pub fn read_byte(&self, address: usize) -> u8 {
        self.data[address]
    }

    /// Read two bytes (a word) from memory
    pub fn fetch_opcode(&self, address: usize) -> u16 {
        // Combine two consecutive bytes to form a 16-bit opcode
        let high_byte = self.data[address] as u16;
        let low_byte = self.data[address + 1] as u16;
        (high_byte << 8) | low_byte
    }

    /// Write a byte to a memory address
    pub fn write_byte(&mut self, address: usize, value: u8) {
        self.data[address] = value;
    }

    /// Reset memory
    pub fn reset(&mut self) {
        self.data = [0; 4096];
    }
}
