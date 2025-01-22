use std::{fs::File, io::Read};
use std::time::{Duration, Instant};
use std::thread;

mod chip8;
mod cpu;
mod memory;
mod display;

fn main() {
    let mut file = File::open("data/INVADERS").unwrap();
    let mut data = Vec::<u8>::new();
    let _ = file.read_to_end(&mut data);

    let mut chip8 = chip8::Chip8::new();
    chip8.load_program(&data);

    let mut last_update = Instant::now();

    loop {
        if !chip8.decode_and_execute() {
            break;
        }

        // Simulate 60Hz update rate for timers and display
        if last_update.elapsed() >= Duration::from_millis(16) {
            // Update timers and display here
            last_update = Instant::now();
        }

        // Handle key inputs here
        // Example: chip8.cpu.set_key(0, true); // Set key 0 as pressed

        // Sleep to limit the emulation speed
        thread::sleep(Duration::from_millis(1));
    }
}

fn fetch_opcode(memory: &[u8; 4096], pc: usize) -> u16 {
    // Combine two consecutive bytes to form a 16-bit opcode
    let high_byte = memory[pc] as u16;
    let low_byte = memory[pc + 1] as u16;
    (high_byte << 8) | low_byte
}
