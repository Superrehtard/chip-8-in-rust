use crate::memory::Memory;

pub const PROGRAM_START: u16 = 0x200;

pub struct Cpu {
    registers: [u8; 16],
    return_stack: Vec<u16>,
    pc: u16,
    i_register: u16,
    display: [[u8; 64]; 32],
    keys: [bool; 16],
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: [0; 16],
            return_stack: Vec::new(),
            pc: PROGRAM_START,
            i_register: 0,
            display: [[0; 64]; 32],
            keys: [false; 16],
        }
    }

    // Read the program counter
    pub fn read_pc(&self) -> u16 {
        self.pc
    }

    // Write to the program counter
    pub fn write_pc(&mut self, value: u16) {
        self.pc = value;
    }

    pub fn write_register(&mut self, register: usize, value: u8) {
        self.registers[register] = value;
    }

    pub fn read_register(&self, register: usize) -> u8 {
        self.registers[register]
    }

    pub fn clear_display(&mut self) {
        for row in self.display.iter_mut() {
            row.fill(0);
        }
    }

    pub fn set_key(&mut self, key: usize, pressed: bool) {
        self.keys[key] = pressed;
    }

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

    pub fn decode_and_execute(&mut self, memory: &mut Memory) -> bool {
        // Ensure the program counter is within the bounds of memory
        if (self.pc + 1) as usize >= memory.len() {
            println!("Program counter out of bounds: {}", self.pc);
            return false;
        }

        // Fetch the opcode from memory
        let opcode = memory.fetch_opcode(self.pc as usize);
        if opcode != 0 {
            println!("Fetched opcode: {:04X} at {}", opcode, self.pc);
        }
        // Decode and execute the opcode
        match opcode & 0xF000 {
            0x0000 => {
                match opcode & 0x00FF {
                    0x00E0 => {
                        // 00E0: Clear the display
                        self.clear_display();
                        self.pc += 2;
                    }
                    0x00EE => {
                        // 00EE: Return from subroutine
                        // if let Some(return_address) = stack.pop() {
                        //     println!("Return from subroutine to 0x{:03X}", return_address);
                        //     *pc = return_address;
                        // } else {
                        //     println!("Stack is empty. Cannot return from subroutine.");
                        // }
                        println!("Return from subroutine (ignored for now)");
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
                self.pc = address; // Update the program counter
            }
            0x2000 => {
                // Opcode 2NNN: call subroutine at NNN
                let address = opcode & 0x0FFF;
                println!("Call subroutine at 0x{:03X}", address);
                // push pc + 2 to stack, then set pc = address
                self.return_stack.push(self.pc + 2);
                println!("Pushed 0x{:03X} to stack", self.pc);

                // set pc to the subroutine address
                self.pc = address;
            }
            0x3000 => {
                // Opcode 3XNN: skip if register X == NN
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = opcode & 0x00FF;
                println!("Skip next instruction if V[{:X}] == 0x{:02X}", x, nn);

                if self.registers[x] == nn as u8 {
                    self.pc += 4; // Skip next instruction
                } else {
                    self.pc += 2; // Move to next instruction
                }
            }
            0x4000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                println!("Skip next instruction if V[{:X}] != 0x{:02X}", x, nn);

                if self.registers[x] != nn {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x5000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                println!("Skip next instruction if V[{:X}] == V[{:X}]", x, y);

                if self.registers[x] == self.registers[y] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x6000 => {
                // Opcode 6XNN: set register X to NN
                let x = (opcode & 0x0F00) >> 8;
                let nn = (opcode & 0x00FF) as u8;
                println!("Set V[{:X}] = 0x{:02X}", x, nn);
                // Set register V[x] to NN
                self.registers[x as usize] = nn as u8; // Store NN in register V[x]
                self.pc += 2;
            }
            0x7000 => {
                // 7XNN: Add NN to register X
                let x = (opcode & 0x0F00) >> 8;
                let nn = opcode & 0x00FF;
                println!("Add 0x{:02X} to V[{:X}]", nn, x);
                self.registers[x as usize] = self.registers[x as usize].wrapping_add(nn as u8);
                self.pc += 2; // Advance PC after executing
            }
            0x8000 => {
                match opcode & 0x000F {
                    0x0000 => {
                        // 8XY0: Set Vx = Vy
                        let x = (opcode & 0x0F00) >> 8;
                        let y = (opcode & 0x00F0) >> 4;
                        println!("Set V[{:X}] = V[{:X}]", x, y);
                        self.registers[x as usize] = self.registers[y as usize];
                    }
                    0x0001 => {
                        // 8XY1: Set Vx = Vx OR Vy
                        let x = (opcode & 0x0F00) >> 8;
                        let y = (opcode & 0x00F0) >> 4;
                        println!("Set V[{:X}] = V[{:X}] OR V[{:X}]", x, x, y);
                        self.registers[x as usize] |= self.registers[y as usize];
                    }
                    0x0002 => {
                        // 8XY2: Set Vx = Vx AND Vy
                        let x = (opcode & 0x0F00) >> 8;
                        let y = (opcode & 0x00F0) >> 4;
                        println!("Set V[{:X}] = V[{:X}] AND V[{:X}]", x, x, y);
                        self.registers[x as usize] &= self.registers[y as usize];
                    }
                    0x0003 => {
                        // 8XY3: Set Vx = Vx XOR Vy
                        let x = (opcode & 0x0F00) >> 8;
                        let y = (opcode & 0x00F0) >> 4;
                        println!("Set V[{:X}] = V[{:X}] XOR V[{:X}]", x, x, y);
                        self.registers[x as usize] ^= self.registers[y as usize];
                    }
                    0x0004 => {
                        // 8XY4: Add Vy to Vx, set VF to carry
                        let x = (opcode & 0x0F00) >> 8;
                        let y = (opcode & 0x00F0) >> 4;
                        println!("Add V[{:X}] to V[{:X}]", y, x);
                        let sum =
                            self.registers[x as usize] as u16 + self.registers[y as usize] as u16;
                        self.registers[0xF] = if sum > 0xFF { 1 } else { 0 };
                    }
                    0x0005 => {
                        // 8XY5: Subtract Vy from Vx, set VF to NOT borrow
                        let x = (opcode & 0x0F00) >> 8;
                        let y = (opcode & 0x00F0) >> 4;
                        println!("Subtract V[{:X}] from V[{:X}]", y, x);
                        self.registers[0xF] =
                            if self.registers[x as usize] > self.registers[y as usize] {
                                1
                            } else {
                                0
                            };
                        self.registers[x as usize] =
                            self.registers[x as usize].wrapping_sub(self.registers[y as usize]);
                    }
                    0x0006 => {
                        // 8XY6: Shift Vx right by 1, set VF to LSB of Vx before shift
                        let x = (opcode & 0x0F00) >> 8;
                        println!("Shift V[{:X}] right by 1", x);
                        self.registers[0xF] = self.registers[x as usize] & 0x1;
                        self.registers[x as usize] >>= 1;
                    }
                    0x0007 => {
                        // 8XY7: Set Vx = Vy - Vx, set VF to NOT borrow
                        let x = (opcode & 0x0F00) >> 8;
                        let y = (opcode & 0x00F0) >> 4;
                        println!("Set V[{:X}] = V[{:X}] - V[{:X}]", x, y, x);
                        self.registers[0xF] =
                            if self.registers[y as usize] > self.registers[x as usize] {
                                1
                            } else {
                                0
                            };
                        self.registers[x as usize] =
                            self.registers[y as usize].wrapping_sub(self.registers[x as usize]);
                    }
                    0x000E => {
                        // 8XYE: Shift Vx to the left by 1, set VF to MSB of Vx before shift
                        let x = (opcode & 0x0F00) >> 8;
                        println!("Shift V[{:X}] left by 1", x);
                        self.registers[0xF] = (self.registers[x as usize] & 0x80) >> 7;
                        self.registers[x as usize] <<= 1;
                    }
                    _ => {
                        println!("Unknown opcode: 0x{:04X}", opcode);
                        self.pc += 2; // Skip unknown opcodes
                    }
                }
                self.pc += 2;
            }
            0x9000 => {
                // 9XY0: Skip next instruction if Vx != Vy
                let x = (opcode & 0x0F00) >> 8;
                let y = (opcode & 0x00F0) >> 4;
                println!("Skip next instruction if V[{:X}] != V[{:X}]", x, y);
                if self.registers[x as usize] != self.registers[y as usize] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0xA000 => {
                // ANNN: Set I to address NNN
                let address = opcode & 0x0FFF;
                println!("Set I = 0x{:03X}", address);
                // Set the index register I to the address NNN
                self.i_register = address;
                self.pc += 2;
            }
            0xB000 => {
                // BNNN: Jump to address NNN + V0
                let address = opcode & 0x0FFF;
                println!("Jump to address 0x{:03X} + V[0]", address);
                self.pc = address + self.registers[0] as u16;
            }
            0xC000 => {
                // CXNN: Set Vx to a random number AND NN
                let x = (opcode & 0x0F00) >> 8;
                let nn = opcode & 0x00FF;
                println!("Set V[{:X}] = random() AND 0x{:02X}", x, nn);
                let random_byte: u8 = rand::random();
                self.registers[x as usize] = random_byte & nn as u8;
                self.pc += 2;
            }
            0xD000 => {
                // DXYN: Draw a sprite at coordinate (VX, VY) with width 8 and height N
                let x = self.registers[((opcode & 0x0F00) >> 8) as usize];
                let y = self.registers[((opcode & 0x00F0) >> 4) as usize];
                let height = (opcode & 0x000F) as u8;
                let collision = self.draw_sprite(x, y, height, memory);
                self.registers[0xF] = if collision { 1 } else { 0 };
                self.pc += 2;
            }
            0xE000 => {
                match opcode & 0x00FF {
                    0x009E => {
                        // EX9E: Skip next instruction if key with the value of Vx is pressed
                        let x = (opcode & 0x0F00) >> 8;
                        if self.keys[self.registers[x as usize] as usize] {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    }
                    0x00A1 => {
                        // EXA1: Skip next instruction if key with the value of Vx is not pressed
                        let x = (opcode & 0x0F00) >> 8;
                        if !self.keys[self.registers[x as usize] as usize] {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    }
                    _ => {
                        println!("Unknown opcode: 0x{:04X}", opcode);
                        self.pc += 2; // Skip unknown opcodes
                    }
                }
            }
            0xF000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;

                match opcode & 0x00FF {
                    0x0007 => {
                        // FX07: Set Vx = delay timer value
                        println!("Set V[{:X}] = delay timer (ignored for now)", x);
                        // self.registers[x] = *delay_timer;
                        self.pc += 2;
                    }
                    0x000A => {
                        // FX0A: Wait for a key press, store the value of the key in Vx
                        println!("Wait for key press, store in V[{:X}]", x);
                        // Ignoring this opcode for now
                        // for (i, &key) in keys.iter().enumerate() {
                        //     if key {
                        //         registers[x] = i as u8;
                        //         println!("Key pressed: V[{:X}] = 0x{:02X}", x, i);
                        //         *pc += 2;
                        //         return;
                        //     }
                        // }
                        // If no key is pressed, return without incrementing pc
                        println!("No key pressed. Waiting...");
                    }
                    0x0015 => {
                        // FX15: Set delay timer = Vx
                        println!("Set delay timer = V[{:X}] (ignored for now)", x);
                        // *delay_timer = registers[x];
                        self.pc += 2;
                    }
                    0x0018 => {
                        // FX18: Set sound timer = Vx
                        println!("Set sound timer = V[{:X}] <<Ignored>>", x);
                        // *sound_timer = registers[x];
                        self.pc += 2;
                    }
                    0x001E => {
                        // FX1E: Set I = I + Vx
                        println!("Set I = I + V[{:X}]", x);
                        self.i_register += self.registers[x] as u16;
                        self.pc += 2;
                    }
                    0x0029 => {
                        // FX29: Set I = location of sprite for digit Vx
                        let digit = self.registers[x] as u16 & 0xF; // Only the last 4 bits are used
                        self.i_register = digit * 5; // Each sprite is 5 bytes long (is it?)
                        println!(
                            "Set I = location of sprite for digit {}; I = 0x{:04X}",
                            digit, self.i_register
                        );
                        self.pc += 2;
                    }
                    0x0033 => {
                        // FX33: Store BCD representation of Vx in memory locations I, I+1, I+2
                        let value = self.registers[x];
                        let hundreds = value / 100;
                        let tens = (value / 10) % 10;
                        let ones = value % 10;
                        memory.write_byte(self.i_register as usize, hundreds);
                        memory.write_byte(self.i_register as usize + 1, tens);
                        memory.write_byte(self.i_register as usize + 2, ones);
                        println!(
                            "Stored BCD of V[{:X}] ({}): [{}, {}, {}] at I = 0x{:04X}",
                            x, value, hundreds, tens, ones, self.i_register
                        );
                        self.pc += 2;
                    }
                    0x0055 => {
                        // FX55: Store registers V0 through VX in memory starting at I
                        for reg in 0..=x {
                            memory
                                .write_byte((self.i_register as usize) + reg, self.registers[reg]);
                        }
                        println!(
                            "Stored registers V0 through V[{:X}] in memory starting at I = 0x{:04X}",
                            x,
                            self.i_register
                        );
                        self.pc += 2;
                    }
                    0x0065 => {
                        // FX65: Read registers V0 through VX from memory starting at I
                        for reg in 0..=x {
                            self.registers[reg] = memory.read_byte(self.i_register as usize + reg);
                        }
                        println!(
                            "Read registers V0 through V[{:X}] from memory starting at I = 0x{:04X}",
                            x,
                            self.i_register
                        );
                        self.pc += 2;
                    }
                    _ => {
                        println!("Unknown opcode: 0x{:04X}", opcode);
                        self.pc += 2; // Skip unknown opcodes
                    }
                }
            }
            _ => {
                println!("Unknown opcode: 0x{:04X}", opcode);
                self.pc += 2; // Skip unknown opcodes
            }
        }

        println!("Registers: {:?}", self.registers);
        println!("Program counter: 0x{:03X}", self.pc);
        println!("I register: 0x{:03X}", self.i_register);

        true
    }
}
