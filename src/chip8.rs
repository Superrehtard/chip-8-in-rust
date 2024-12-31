use crate::cpu::{Cpu, PROGRAM_START};
use crate::memory::Memory;

pub struct Chip8 {
    memory: Memory,
    cpu: Cpu,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Self {
            memory: Memory::new(),
            cpu: Cpu::new(),
        }
    }

    pub fn load_program(&mut self, program: &Vec<u8>) {
        self.memory.load_program(program, PROGRAM_START as usize);
    }

    pub fn decode_and_execute(&mut self) -> bool {
        self.cpu.decode_and_execute(&mut self.memory)
    }
}
