fn main() {
    // Simulate 4KB of memory as an array of 4096 bytes
    let mut memory: [u8; 4096] = [0; 4096];
    let mut registers = [0u8; 16];
    let mut display = [[0u8; 64]; 32]; // 32x64 monochrome display

    // Timers
    let mut delay_timer: u8 = 0;
    let mut sound_timer: u8 = 0;

    let mut keys = [false; 16]; // Array to represent the state of the 16 keys
    let mut i_register: u16 = 0; // Index register

    let mut stack: Vec<usize> = Vec::new(); // Stack to store return addresses
                                            // Define where the program should start
    let program_start: usize = 0x200;

    // Example: A small "program" to load into memory
    let program: [u8; 8] = [
        0x12, 0x02, // An example Chip-8 opcode (not real)
        0x22, 0x04, // Another example opcode
        0x61, 0x23, // Another imaginary opcode for chip-8
        0x71, 0x34, // Yet another imaginary opcode
    ];

    keys[5] = true; // Simulate key press

    // Load the program into memory starting at 0x200
    for (i, &byte) in program.iter().enumerate() {
        memory[program_start + i] = byte;
    }

    // Fetch the first opcode from memory
    let mut pc: usize = program_start; // Program counter starts at 0x200

    while pc < memory.len() {
        let opcode = fetch_opcode(&memory, pc);
        println!("Fetched opcode at 0x{:04X}: 0x{:04X}", pc, opcode);
        decode_and_execute(
            opcode,
            &mut pc,
            &mut registers,
            &mut stack,
            &mut memory,
            &mut display,
            &keys,
            &mut delay_timer,
            &mut sound_timer,
            &mut i_register,
        );

        // Stop execution if we jump past the program or hit invalid memory
        if pc >= program_start + program.len() {
            println!("Execution stopped: PC out of bounds.");
            break;
        }
    }
    println!("Final Registers: {:?}", registers);
}

fn fetch_opcode(memory: &[u8; 4096], pc: usize) -> u16 {
    // Combine two consecutive bytes to form a 16-bit opcode
    let high_byte = memory[pc] as u16;
    let low_byte = memory[pc + 1] as u16;
    (high_byte << 8) | low_byte
}

fn clear_display(display: &mut [[u8; 64]; 32]) {
    // clear the display
    println!("Clearing the display.");
    for row in display.iter_mut() {
        row.fill(0);
    }
}

fn decode_and_execute(
    opcode: u16,
    pc: &mut usize,
    registers: &mut [u8; 16],
    stack: &mut Vec<usize>,
    memory: &mut [u8; 4096],
    display: &mut [[u8; 64]; 32],
    keys: &[bool; 16], // Array to represent the state of the 16 keys
    delay_timer: &mut u8,
    sound_timer: &mut u8,
    i_register: &mut u16,
) {
    match opcode & 0xF000 {
        0x0000 => {
            match opcode & 0x00FF {
                0x00E0 => {
                    clear_display(display);
                }
                0x00EE => {
                    // 00EE: Return from subroutine
                    if let Some(return_address) = stack.pop() {
                        println!("Return from subroutine to 0x{:03X}", return_address);
                        *pc = return_address;
                    } else {
                        println!("Stack is empty. Cannot return from subroutine.");
                    }
                }
                _ => {
                    let address = opcode & 0x0FFF;
                    println!(
                        "Call RCA 1802 program at 0x{:03X} (ignored for now)",
                        address
                    );
                    // Ignoring this opcode for now
                }
            }
        }
        0x1000 => {
            // 1NNN: Jump to address NNN
            let address = opcode & 0x0FFF;
            println!("Jump to address 0x{:03X}", address);
            // Set pc to address
            *pc = address as usize; // Update the program counter
        }
        0x2000 => {
            // Opcode 2NNN: call subroutine at NNN
            let address = opcode & 0x0FFF;
            println!("Call subroutine at 0x{:03X}", address);
            // push current pc to stack, then set pc = address
            stack.push(*pc);

            // set pc to the subroutine address
            *pc = address as usize;
        }
        0x3000 => {
            // Opcode 3XNN: skip if register X == NN
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let nn = opcode & 0x00FF;
            println!("Skip next instruction if V[{:X}] == 0x{:02X}", x, nn);

            if registers[x] == nn as u8 {
                *pc += 4; // Skip next instruction
            } else {
                *pc += 2; // Move to next instruction
            }
        }
        0x4000 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let nn = (opcode & 0x00FF) as u8;
            println!("Skip next instruction if V[{:X}] != 0x{:02X}", x, nn);

            if registers[x] != nn {
                *pc += 4;
            } else {
                *pc += 2;
            }
        }
        0x5000 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let y = ((opcode & 0x00F0) >> 4) as usize;
            println!("Skip next instruction if V[{:X}] == V[{:X}]", x, y);

            if registers[x] == registers[y] {
                *pc += 4;
            } else {
                *pc += 2;
            }
        }
        0x6000 => {
            // Opcode 6XNN: set register X to NN
            let x = (opcode & 0x0F00) >> 8;
            let nn = (opcode & 0x00FF) as u8;
            println!("Set V[{:X}] = 0x{:02X}", x, nn);
            // Set register V[x] to NN
            registers[x as usize] = nn as u8; // Store NN in register V[x]
            *pc += 2;
        }
        0x7000 => {
            // 7XNN: Add NN to register X
            let x = (opcode & 0x0F00) >> 8;
            let nn = opcode & 0x00FF;
            println!("Add 0x{:02X} to V[{:X}]", nn, x);
            registers[x as usize] = registers[x as usize].wrapping_add(nn as u8);
            *pc += 2; // Advance PC after executing
        }
        0x8000 => {
            match opcode & 0x000F {
                0x0000 => {
                    // 8XY0: Set Vx = Vy
                    let x = (opcode & 0x0F00) >> 8;
                    let y = (opcode & 0x00F0) >> 4;
                    println!("Set V[{:X}] = V[{:X}]", x, y);
                    registers[x as usize] = registers[y as usize];
                }
                0x0001 => {
                    // 8XY1: Set Vx = Vx OR Vy
                    let x = (opcode & 0x0F00) >> 8;
                    let y = (opcode & 0x00F0) >> 4;
                    println!("Set V[{:X}] = V[{:X}] OR V[{:X}]", x, x, y);
                    registers[x as usize] |= registers[y as usize];
                }
                0x0002 => {
                    // 8XY2: Set Vx = Vx AND Vy
                    let x = (opcode & 0x0F00) >> 8;
                    let y = (opcode & 0x00F0) >> 4;
                    println!("Set V[{:X}] = V[{:X}] AND V[{:X}]", x, x, y);
                    registers[x as usize] &= registers[y as usize];
                }
                0x0003 => {
                    // 8XY3: Set Vx = Vx XOR Vy
                    let x = (opcode & 0x0F00) >> 8;
                    let y = (opcode & 0x00F0) >> 4;
                    println!("Set V[{:X}] = V[{:X}] XOR V[{:X}]", x, x, y);
                    registers[x as usize] ^= registers[y as usize];
                }
                0x0004 => {
                    // 8XY4: Add Vy to Vx, set VF to carry
                    let x = (opcode & 0x0F00) >> 8;
                    let y = (opcode & 0x00F0) >> 4;
                    println!("Add V[{:X}] to V[{:X}]", y, x);
                    let sum = registers[x as usize] as u16 + registers[y as usize] as u16;
                    registers[0xF] = if sum > 0xFF { 1 } else { 0 };
                }
                0x0005 => {
                    // 8XY5: Subtract Vy from Vx, set VF to NOT borrow
                    let x = (opcode & 0x0F00) >> 8;
                    let y = (opcode & 0x00F0) >> 4;
                    println!("Subtract V[{:X}] from V[{:X}]", y, x);
                    registers[0xF] = if registers[x as usize] > registers[y as usize] {
                        1
                    } else {
                        0
                    };
                    registers[x as usize] =
                        registers[x as usize].wrapping_sub(registers[y as usize]);
                }
                0x0006 => {
                    // 8XY6: Shift Vx right by 1, set VF to LSB of Vx before shift
                    let x = (opcode & 0x0F00) >> 8;
                    println!("Shift V[{:X}] right by 1", x);
                    registers[0xF] = registers[x as usize] & 0x1;
                    registers[x as usize] >>= 1;
                }
                0x0007 => {
                    // 8XY7: Set Vx = Vy - Vx, set VF to NOT borrow
                    let x = (opcode & 0x0F00) >> 8;
                    let y = (opcode & 0x00F0) >> 4;
                    println!("Set V[{:X}] = V[{:X}] - V[{:X}]", x, y, x);
                    registers[0xF] = if registers[y as usize] > registers[x as usize] {
                        1
                    } else {
                        0
                    };
                    registers[x as usize] =
                        registers[y as usize].wrapping_sub(registers[x as usize]);
                }
                0x000E => {
                    // 8XYE: Shift Vx to the left by 1, set VF to MSB of Vx before shift
                    let x = (opcode & 0x0F00) >> 8;
                    println!("Shift V[{:X}] left by 1", x);
                    registers[0xF] = (registers[x as usize] & 0x80) >> 7;
                    registers[x as usize] <<= 1;
                }
                _ => {
                    println!("Unknown opcode: 0x{:04X}", opcode);
                    *pc += 2; // Skip unknown opcodes
                }
            }
            *pc += 2;
        }
        0x9000 => {
            // 9XY0: Skip next instruction if Vx != Vy
            let x = (opcode & 0x0F00) >> 8;
            let y = (opcode & 0x00F0) >> 4;
            println!("Skip next instruction if V[{:X}] != V[{:X}]", x, y);
            if registers[x as usize] != registers[y as usize] {
                *pc += 4;
            } else {
                *pc += 2;
            }
        }
        0xA000 => {
            // ANNN: Set I to address NNN
            let address = opcode & 0x0FFF;
            println!("Set I = 0x{:03X}", address);
            // Set the index register I to the address NNN
            registers[0xF] = 0; // Reset the carry flag
            *pc += 2;
        }
        0xB000 => {
            // BNNN: Jump to address NNN + V0
            let address = opcode & 0x0FFF;
            println!("Jump to address 0x{:03X} + V[0]", address);
            *pc = (address + registers[0] as u16) as usize;
        }
        0xC000 => {
            // CXNN: Set Vx to a random number AND NN
            let x = (opcode & 0x0F00) >> 8;
            let nn = opcode & 0x00FF;
            println!("Set V[{:X}] = random() AND 0x{:02X}", x, nn);
            let random_byte: u8 = rand::random();
            registers[x as usize] = random_byte & nn as u8;
            *pc += 2;
        }
        0xD000 => {
            // DXYN: Draw a sprite at coordinate (VX, VY) with width 8 and height N
            let x = (opcode & 0x0F00) >> 8;
            let y = (opcode & 0x00F0) >> 4;
            let height = opcode & 0x000F;
            println!(
                "Draw sprite at V[{:X}], V[{:X}] with height {}",
                x, y, height
            );
            let i = registers[x as usize] as usize;
            let j = registers[y as usize] as usize;
            let sprite =
                &memory[registers[0xF] as usize..registers[0xF] as usize + height as usize];
            for (row, &byte) in sprite.iter().enumerate() {
                for col in 0..8 {
                    let bit = (byte >> (7 - col)) & 0x1;
                    display[(i + row) % 32][(j + col) % 64] ^= bit;
                }
            }
            *pc += 2;
        }
        0xE000 => {
            match opcode & 0x00FF {
                0x009E => {
                    // EX9E: Skip next instruction if key with the value of Vx is pressed
                    let x = (opcode & 0x0F00) >> 8;
                    println!("Skip next instruction if key V[{:X}] is pressed", x);
                    let key = registers[x as usize] as usize;

                    if keys[key] {
                        *pc += 4; // skip next instruction
                    } else {
                        *pc += 2;
                    }
                }
                0x00A1 => {
                    // EXA1: Skip next instruction if key with the value of Vx is not pressed
                    let x = (opcode & 0x0F00) >> 8;
                    println!("Skip next instruction if key V[{:X}] is not pressed", x);
                    // Ignoring this opcode for now
                    let key = registers[x as usize] as usize;

                    if !keys[key] {
                        *pc += 4; // skip next instruction
                    } else {
                        *pc += 2;
                    }
                }
                _ => {
                    println!("Unknown opcode: 0x{:04X}", opcode);
                    *pc += 2; // Skip unknown opcodes
                }
            }
        }
        0xF000 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;

            match opcode & 0x00FF {
                0x0007 => {
                    // FX07: Set Vx = delay timer value
                    println!("Set V[{:X}] = delay timer", x);
                    registers[x] = *delay_timer;
                    *pc += 2;
                }
                0x000A => {
                    // FX0A: Wait for a key press, store the value of the key in Vx
                    println!("Wait for key press, store in V[{:X}]", x);
                    for (i, &key) in keys.iter().enumerate() {
                        if key {
                            registers[x] = i as u8;
                            println!("Key pressed: V[{:X}] = 0x{:02X}", x, i);
                            *pc += 2;
                            return;
                        }
                    }
                    // If no key is pressed, return without incrementing pc
                    println!("No key pressed. Waiting...");
                }
                0x0015 => {
                    // FX15: Set delay timer = Vx
                    println!("Set delay timer = V[{:X}]", x);
                    *delay_timer = registers[x];
                    *pc += 2;
                }
                0x0018 => {
                    // FX18: Set sound timer = Vx
                    println!("Set sound timer = V[{:X}]", x);
                    *sound_timer = registers[x];
                    *pc += 2;
                }
                0x001E => {
                    // FX1E: Set I = I + Vx
                    println!("Set I = I + V[{:X}]", x);
                    *i_register += registers[x] as u16;
                    *pc += 2;
                }
                0x0029 => {
                    // FX29: Set I = location of sprite for digit Vx
                    let digit = registers[x] as u16 & 0xF; // Only the last 4 bits are used
                    *i_register = digit * 5; // Each sprite is 5 bytes long (is it?)
                    println!(
                        "Set I = location of sprite for digit {}; I = 0x{:04X}",
                        digit, *i_register
                    );
                    *pc += 2;
                }
                0x0033 => {
                    // FX33: Store BCD representation of Vx in memory locations I, I+1, I+2
                    let value = registers[x];
                    let hundreds = value / 100;
                    let tens = (value / 10) % 10;
                    let ones = value % 10;
                    memory[*i_register as usize] = hundreds;
                    memory[*i_register as usize + 1] = tens;
                    memory[*i_register as usize + 2] = ones;
                    println!(
                        "Stored BCD of V[{:X}] ({}): [{}, {}, {}] at I = 0x{:04X}",
                        x,
                        value,
                        hundreds,
                        tens,
                        ones,
                        *i_register
                    );
                    *pc += 2;
                }
                0x0055 => {
                    // FX55: Store registers V0 through VX in memory starting at I
                    for reg in 0..=x {
                        memory[(*i_register as usize) + reg] = registers[reg];
                    }
                    println!(
                        "Stored registers V0 through V[{:X}] in memory starting at I = 0x{:04X}",
                        x,
                        *i_register
                    );
                    *pc += 2;
                }
                0x0065 => {
                    // FX65: Read registers V0 through VX from memory starting at I
                    for reg in 0..=x {
                        registers[reg] = memory[(*i_register as usize) + reg];
                    }
                    println!(
                        "Read registers V0 through V[{:X}] from memory starting at I = 0x{:04X}",
                        x,
                        *i_register
                    );
                    *pc += 2;
                }
                _ => {
                    println!("Unknown opcode: 0x{:04X}", opcode);
                    *pc += 2; // Skip unknown opcodes
                }
            }
        }
        _ => {
            println!("Unknown opcode: 0x{:04X}", opcode);
            *pc += 2; // Skip unknown opcodes
        }
    }
}
