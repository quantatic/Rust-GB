use std::fmt::Debug;

pub struct Cpu {
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub sp: u16,
    pub pc: u16,
    pub interrupt_master_enable: bool,
    pub memory: [u8; 0x10000],
}

impl Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cpu")
            .field("af", &self.af)
            .field("bc", &self.bc)
            .field("de", &self.de)
            .field("hl", &self.hl)
            .field("sp", &self.sp)
            .field("pc", &self.pc)
            .field("interrupt_master_enable", &self.interrupt_master_enable)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Instruction {
    pub instruction_type: InstructionType,
    pub cycles: u16,
}

#[derive(Clone, Copy, Debug)]
pub enum InstructionType {
    AddByte {
        source: AddressingModeByte,
        destination: AddressingModeByte,
    },
    AddHl {
        source: AddressingModeWord,
    },
    AddSp {
        value: i8,
    },
    Adc {
        source: AddressingModeByte,
        destination: AddressingModeByte,
    },
    And {
        source: AddressingModeByte,
    },
    Bit {
        target: AddressingModeByte,
        bit: u8,
    },
    Call {
        address: u16,
        taken_penalty: u16,
        condition: BranchConditionType,
    },
    Cp {
        source: AddressingModeByte,
    },
    DecByte {
        target: AddressingModeByte,
    },
    DecWord {
        target: AddressingModeWord,
    },
    Di,
    Ei,
    Halt,
    IncByte {
        target: AddressingModeByte,
    },
    IncWord {
        target: AddressingModeWord,
    },
    Jp {
        target: AddressingModeWord,
        taken_penalty: u16,
        condition: BranchConditionType,
    },
    Jr {
        offset: i8,
        taken_penalty: u16,
        condition: BranchConditionType,
    },
    LdByte {
        source: AddressingModeByte,
        destination: AddressingModeByte,
    },
    LdWord {
        source: AddressingModeWord,
        destination: AddressingModeWord,
    },
    Ldh {
        source: AddressingModeByte,
        destination: AddressingModeByte,
    },
    Ldhl {
        source: AddressingModeWord,
        offset: i8,
    },
    Nop,
    Or {
        source: AddressingModeByte,
    },
    Pop {
        target: AddressingModeWord,
    },
    Push {
        source: AddressingModeWord,
    },
    Res {
        target: AddressingModeByte,
        bit: u8,
    },
    Ret {
        taken_penalty: u16,
        condition: BranchConditionType,
    },
    Reti,
    Rl {
        target: AddressingModeByte,
    },
    Rla,
    Rlc {
        target: AddressingModeByte,
    },
    Rlca,
    Rr {
        target: AddressingModeByte,
    },
    Rra,
    Rrc {
        target: AddressingModeByte,
    },
    Rrca,
    Rst {
        offset: u16,
    },
    Sbc {
        source: AddressingModeByte,
        destination: AddressingModeByte,
    },
    Sla {
        target: AddressingModeByte,
    },
    Set {
        target: AddressingModeByte,
        bit: u8,
    },
    Sra {
        target: AddressingModeByte,
    },
    Srl {
        target: AddressingModeByte,
    },
    Sub {
        source: AddressingModeByte,
    },
    Swap {
        target: AddressingModeByte,
    },
    Xor {
        source: AddressingModeByte,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum BranchConditionType {
    NotZero,
    NotCarry,
    Zero,
    Carry,
    Unconditional,
}

impl BranchConditionType {
    fn is_conditional(self) -> bool {
        match self {
            BranchConditionType::NotZero => true,
            BranchConditionType::NotCarry => true,
            BranchConditionType::Zero => true,
            BranchConditionType::Carry => true,
            BranchConditionType::Unconditional => true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AddressingModeByte {
    Accumulator,
    B,
    C,
    D,
    E,
    H,
    L,
    BcIndirect,
    DeIndirect,
    HlIndirect,
    HlIndirectIncrement,
    HlIndirectDecrement,
    Literal(u8),
    LiteralIndirect(u16),
}

impl AddressingModeByte {
    fn is_indirect(self) -> bool {
        match self {
            AddressingModeByte::Accumulator => false,
            AddressingModeByte::B => false,
            AddressingModeByte::C => false,
            AddressingModeByte::D => false,
            AddressingModeByte::E => false,
            AddressingModeByte::H => false,
            AddressingModeByte::L => false,
            AddressingModeByte::BcIndirect => true,
            AddressingModeByte::DeIndirect => true,
            AddressingModeByte::HlIndirect => true,
            AddressingModeByte::HlIndirectIncrement => true,
            AddressingModeByte::HlIndirectDecrement => true,
            AddressingModeByte::Literal(_) => false,
            AddressingModeByte::LiteralIndirect(_) => true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AddressingModeWord {
    Af,
    Bc,
    De,
    Hl,
    Sp,
    Literal(u16),
    LiteralIndirect(u16),
}

impl Default for Cpu {
    fn default() -> Self {
        let mut memory = [0; 0x10000];
        memory[0xFF05] = 0x00;
        memory[0xFF05] = 0x00;
        memory[0xFF06] = 0x00;
        memory[0xFF07] = 0x00;
        memory[0xFF10] = 0x80;
        memory[0xFF11] = 0xBF;
        memory[0xFF12] = 0xF3;
        memory[0xFF14] = 0xBF;
        memory[0xFF16] = 0x3F;
        memory[0xFF17] = 0x00;
        memory[0xFF19] = 0xBF;
        memory[0xFF1A] = 0x7F;
        memory[0xFF1B] = 0xFF;
        memory[0xFF1C] = 0x9F;
        memory[0xFF1E] = 0xBF;
        memory[0xFF20] = 0xFF;
        memory[0xFF21] = 0x00;
        memory[0xFF22] = 0x00;
        memory[0xFF23] = 0xBF;
        memory[0xFF24] = 0x77;
        memory[0xFF25] = 0xF3;
        memory[0xFF26] = 0xF1;
        memory[0xFF40] = 0x91;
        memory[0xFF42] = 0x00;
        memory[0xFF43] = 0x00;
        memory[0xFF45] = 0x00;
        memory[0xFF47] = 0xFC;
        memory[0xFF48] = 0xFF;
        memory[0xFF49] = 0xFF;
        memory[0xFF4A] = 0x00;
        memory[0xFF4B] = 0x00;
        memory[0xFFFF] = 0x00;

        Self {
            af: 0x01B0,
            bc: 0x0013,
            de: 0x00D8,
            hl: 0x014D,
            sp: 0xFFFE,
            pc: 0x100,
            interrupt_master_enable: false,
            memory,
        }
    }
}

impl Cpu {
    fn read_byte_address(&self, address: u16) -> u8 {
        let result = self.memory[usize::from(address)];
        // println!("memory[{:#X}] -> {:#X}", address, result);
        result
    }

    fn read_word_address(&self, address: u16) -> u16 {
        let low = self.read_byte_address(address);
        let high = self.read_byte_address(address + 1);
        u16::from(low) | (u16::from(high) << 8)
    }

    fn read_byte(&mut self, location: AddressingModeByte) -> u8 {
        match location {
            AddressingModeByte::Accumulator => (self.af >> 8) as u8,
            AddressingModeByte::B => (self.bc >> 8) as u8,
            AddressingModeByte::C => self.bc as u8,
            AddressingModeByte::D => (self.de >> 8) as u8,
            AddressingModeByte::E => self.de as u8,
            AddressingModeByte::H => (self.hl >> 8) as u8,
            AddressingModeByte::L => self.hl as u8,
            AddressingModeByte::BcIndirect => self.read_byte_address(self.bc),
            AddressingModeByte::DeIndirect => self.read_byte_address(self.de),
            AddressingModeByte::HlIndirect => self.read_byte_address(self.hl),
            AddressingModeByte::HlIndirectIncrement => {
                let result = self.read_byte_address(self.hl);
                self.hl = self.hl.wrapping_add(1);
                result
            }
            AddressingModeByte::HlIndirectDecrement => {
                let result = self.read_byte_address(self.hl);
                self.hl = self.hl.wrapping_sub(1);
                result
            }
            AddressingModeByte::Literal(val) => val,
            AddressingModeByte::LiteralIndirect(address) => self.read_byte_address(address),
        }
    }

    fn read_word(&mut self, location: AddressingModeWord) -> u16 {
        match location {
            AddressingModeWord::Af => self.af,
            AddressingModeWord::Bc => self.bc,
            AddressingModeWord::De => self.de,
            AddressingModeWord::Hl => self.hl,
            AddressingModeWord::Sp => self.sp,
            AddressingModeWord::Literal(val) => val,
            AddressingModeWord::LiteralIndirect(address) => self.read_word_address(address),
        }
    }

    fn write_byte_address(&mut self, value: u8, address: u16) {
        if address == 0xFF01 {
            print!("{}", char::from(value));
        }

        self.memory[usize::from(address)] = value;
        // println!("{:#X} -> memory[{:#X}]", value, address);
    }

    fn write_word_address(&mut self, value: u16, address: u16) {
        let low = value & 0x00FF;
        let high = value >> 8;
        self.write_byte_address(low as u8, address);
        self.write_byte_address(high as u8, address + 1);
    }

    fn write_byte(&mut self, val: u8, location: AddressingModeByte) {
        match location {
            AddressingModeByte::Accumulator => {
                self.af &= !0xFF00;
                self.af |= u16::from(val) << 8;
            }
            AddressingModeByte::B => {
                self.bc &= !0xFF00;
                self.bc |= u16::from(val) << 8;
            }
            AddressingModeByte::C => {
                self.bc &= !0x00FF;
                self.bc |= u16::from(val)
            }
            AddressingModeByte::D => {
                self.de &= !0xFF00;
                self.de |= u16::from(val) << 8
            }
            AddressingModeByte::E => {
                self.de &= !0x00FF;
                self.de |= u16::from(val)
            }
            AddressingModeByte::H => {
                self.hl &= !0xFF00;
                self.hl |= u16::from(val) << 8
            }
            AddressingModeByte::L => {
                self.hl &= !0x00FF;
                self.hl |= u16::from(val)
            }
            AddressingModeByte::BcIndirect => self.write_byte_address(val, self.bc),
            AddressingModeByte::DeIndirect => self.write_byte_address(val, self.de),
            AddressingModeByte::HlIndirect => self.write_byte_address(val, self.hl),
            AddressingModeByte::HlIndirectIncrement => {
                self.write_byte_address(val, self.hl);
                self.hl = self.hl.wrapping_add(1)
            }
            AddressingModeByte::HlIndirectDecrement => {
                self.write_byte_address(val, self.hl);
                self.hl = self.hl.wrapping_sub(1)
            }
            AddressingModeByte::Literal(_) => unreachable!(),
            AddressingModeByte::LiteralIndirect(address) => self.write_byte_address(val, address),
        }
    }

    fn write_word(&mut self, val: u16, location: AddressingModeWord) {
        match location {
            AddressingModeWord::Af => self.af = val,
            AddressingModeWord::Bc => self.bc = val,
            AddressingModeWord::De => self.de = val,
            AddressingModeWord::Hl => self.hl = val,
            AddressingModeWord::Sp => self.sp = val,
            AddressingModeWord::Literal(_) => unreachable!(),
            AddressingModeWord::LiteralIndirect(address) => self.write_word_address(val, address),
        }
    }

    pub fn decode(&mut self) -> Instruction {
        let opcode = self.read_byte_address(self.pc);
        match opcode {
            0x00 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Nop,
                    cycles: 4,
                }
            }
            0x01 | 0x11 | 0x21 | 0x31 => {
                let source = AddressingModeWord::Literal(self.read_word_address(self.pc + 1));
                let destination = match (opcode & 0b00110000) >> 4 {
                    0b00 => AddressingModeWord::Bc,
                    0b01 => AddressingModeWord::De,
                    0b10 => AddressingModeWord::Hl,
                    0b11 => AddressingModeWord::Sp,
                    _ => unreachable!(),
                };

                self.pc += 3;
                Instruction {
                    instruction_type: InstructionType::LdWord {
                        source,
                        destination,
                    },
                    cycles: 12,
                }
            }
            0x02 | 0x12 | 0x22 | 0x32 => {
                let source = AddressingModeByte::Accumulator;
                let destination = match (opcode & 0b00110000) >> 4 {
                    0b00 => AddressingModeByte::BcIndirect,
                    0b01 => AddressingModeByte::DeIndirect,
                    0b10 => AddressingModeByte::HlIndirectIncrement,
                    0b11 => AddressingModeByte::HlIndirectDecrement,
                    _ => unreachable!(),
                };

                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::LdByte {
                        source,
                        destination,
                    },
                    cycles: 8,
                }
            }
            0x03 | 0x0B | 0x13 | 0x1B | 0x23 | 0x2B | 0x33 | 0x3B => {
                fn get_addressing_mode(val: u8) -> AddressingModeWord {
                    match val {
                        0b00 => AddressingModeWord::Bc,
                        0b01 => AddressingModeWord::De,
                        0b10 => AddressingModeWord::Hl,
                        0b11 => AddressingModeWord::Sp,
                        _ => unreachable!(),
                    }
                }

                let target = get_addressing_mode((opcode & 0b00110000) >> 4);
                self.pc += 1;
                let instruction_type = match (opcode & 0b00001100) >> 2 {
                    0b00 => InstructionType::IncWord { target },
                    0b10 => InstructionType::DecWord { target },
                    _ => unreachable!(),
                };
                Instruction {
                    instruction_type,
                    cycles: 8,
                }
            }
            0x04 | 0x05 | 0x0C | 0x0D | 0x14 | 0x15 | 0x1C | 0x1D | 0x24 | 0x25 | 0x2C | 0x2D
            | 0x34 | 0x35 | 0x3C | 0x3D => {
                fn get_addressing_mode(val: u8) -> AddressingModeByte {
                    match val {
                        0b000 => AddressingModeByte::B,
                        0b001 => AddressingModeByte::C,
                        0b010 => AddressingModeByte::D,
                        0b011 => AddressingModeByte::E,
                        0b100 => AddressingModeByte::H,
                        0b101 => AddressingModeByte::L,
                        0b110 => AddressingModeByte::HlIndirect,
                        0b111 => AddressingModeByte::Accumulator,
                        _ => unreachable!(),
                    }
                }

                let target = get_addressing_mode((opcode & 0b00111000) >> 3);
                let cycles = if target.is_indirect() { 12 } else { 4 };

                self.pc += 1;
                let instruction_type = match opcode & 0b00000111 {
                    0b100 => InstructionType::IncByte { target },
                    0b101 => InstructionType::DecByte { target },
                    _ => unreachable!(),
                };

                Instruction {
                    instruction_type,
                    cycles,
                }
            }
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
                fn get_addressing_mode(val: u8) -> AddressingModeByte {
                    match val {
                        0b000 => AddressingModeByte::B,
                        0b001 => AddressingModeByte::C,
                        0b010 => AddressingModeByte::D,
                        0b011 => AddressingModeByte::E,
                        0b100 => AddressingModeByte::H,
                        0b101 => AddressingModeByte::L,
                        0b110 => AddressingModeByte::HlIndirect,
                        0b111 => AddressingModeByte::Accumulator,
                        _ => unreachable!(),
                    }
                }
                let r = get_addressing_mode((opcode & 0b00111000) >> 3);
                let n = self.read_byte_address(self.pc + 1);
                let cycles = if r.is_indirect() { 12 } else { 8 };

                self.pc += 2;
                Instruction {
                    instruction_type: InstructionType::LdByte {
                        source: AddressingModeByte::Literal(n),
                        destination: r,
                    },
                    cycles,
                }
            }
            0x07 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Rlca,
                    cycles: 4,
                }
            }
            0x08 => {
                let address = self.read_word_address(self.pc + 1);

                self.pc += 3;
                Instruction {
                    instruction_type: InstructionType::LdWord {
                        source: AddressingModeWord::Sp,
                        destination: AddressingModeWord::LiteralIndirect(address),
                    },
                    cycles: 20,
                }
            }
            0x09 | 0x19 | 0x29 | 0x39 => {
                let source = match opcode {
                    0x09 => AddressingModeWord::Bc,
                    0x19 => AddressingModeWord::De,
                    0x29 => AddressingModeWord::Hl,
                    0x39 => AddressingModeWord::Sp,
                    _ => unreachable!(),
                };

                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::AddHl { source },
                    cycles: 8,
                }
            }
            0x0A | 0x1A | 0x2A | 0x3A => {
                fn get_addressing_mode(val: u8) -> AddressingModeByte {
                    match val {
                        0b00 => AddressingModeByte::BcIndirect,
                        0b01 => AddressingModeByte::DeIndirect,
                        0b10 => AddressingModeByte::HlIndirectIncrement,
                        0b11 => AddressingModeByte::HlIndirectDecrement,
                        _ => unreachable!(),
                    }
                }

                let source = get_addressing_mode((opcode & 0b00110000) >> 4);
                let destination = AddressingModeByte::Accumulator;

                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::LdByte {
                        source,
                        destination,
                    },
                    cycles: 8,
                }
            }
            0x0F => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Rrca,
                    cycles: 4,
                }
            }
            0x17 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Rla,
                    cycles: 4,
                }
            }
            0x18 => {
                let offset = self.read_byte_address(self.pc + 1) as i8;
                self.pc += 2;
                Instruction {
                    instruction_type: InstructionType::Jr {
                        offset,
                        taken_penalty: 0,
                        condition: BranchConditionType::Unconditional,
                    },
                    cycles: 12,
                }
            }
            0x1F => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Rra,
                    cycles: 4,
                }
            }
            0x20 | 0x28 | 0x30 | 0x38 => {
                fn get_branch_condition_type(val: u8) -> BranchConditionType {
                    match val {
                        0b100 => BranchConditionType::NotZero,
                        0b101 => BranchConditionType::Zero,
                        0b110 => BranchConditionType::NotCarry,
                        0b111 => BranchConditionType::Carry,
                        _ => unreachable!(),
                    }
                }

                let condition = get_branch_condition_type((opcode & 0b00111000) >> 3);

                // Numbers are stored as 2's complement, so we can simply cast to i8 for desired offset.
                let offset = self.read_byte_address(self.pc + 1) as i8;

                self.pc += 2;

                Instruction {
                    instruction_type: InstructionType::Jr {
                        offset,
                        taken_penalty: 4,
                        condition,
                    },
                    cycles: 8,
                }
            }
            0x40 | 0x41 | 0x42 | 0x43 | 0x44 | 0x45 | 0x46 | 0x47 | 0x48 | 0x49 | 0x4A | 0x4B
            | 0x4C | 0x4D | 0x4E | 0x4F | 0x50 | 0x51 | 0x52 | 0x53 | 0x54 | 0x55 | 0x56 | 0x57
            | 0x58 | 0x59 | 0x5A | 0x5B | 0x5C | 0x5D | 0x5E | 0x5F | 0x60 | 0x61 | 0x62 | 0x63
            | 0x64 | 0x65 | 0x66 | 0x67 | 0x68 | 0x69 | 0x6A | 0x6B | 0x6C | 0x6D | 0x6E | 0x6F
            | 0x70 | 0x71 | 0x72 | 0x73 | 0x74 | 0x75 | 0x77 | 0x78 | 0x79 | 0x7A | 0x7B | 0x7C
            | 0x7D | 0x7E | 0x7F => {
                fn get_addressing_mode(val: u8) -> AddressingModeByte {
                    match val {
                        0b000 => AddressingModeByte::B,
                        0b001 => AddressingModeByte::C,
                        0b010 => AddressingModeByte::D,
                        0b011 => AddressingModeByte::E,
                        0b100 => AddressingModeByte::H,
                        0b101 => AddressingModeByte::L,
                        0b110 => AddressingModeByte::HlIndirect,
                        0b111 => AddressingModeByte::Accumulator,
                        _ => unreachable!(),
                    }
                }

                let source = get_addressing_mode(opcode & 0b00000111);
                let destination = get_addressing_mode((opcode & 0b00111000) >> 3);
                let cycles = if source.is_indirect() || destination.is_indirect() {
                    8
                } else {
                    4
                };

                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::LdByte {
                        source,
                        destination,
                    },
                    cycles,
                }
            }
            0x76 => Instruction {
                instruction_type: InstructionType::Halt,
                cycles: 4,
            },
            0x80 | 0x81 | 0x82 | 0x83 | 0x84 | 0x85 | 0x86 | 0x87 | 0x88 | 0x89 | 0x8A | 0x8B
            | 0x8C | 0x8D | 0x8E | 0x8F | 0x90 | 0x91 | 0x92 | 0x93 | 0x94 | 0x95 | 0x96 | 0x97
            | 0x98 | 0x99 | 0x9A | 0x9B | 0x9C | 0x9D | 0x9E | 0x9F | 0xA0 | 0xA1 | 0xA2 | 0xA3
            | 0xA4 | 0xA5 | 0xA6 | 0xA7 | 0xA8 | 0xA9 | 0xAA | 0xAB | 0xAC | 0xAD | 0xAE | 0xAF
            | 0xB0 | 0xB1 | 0xB2 | 0xB3 | 0xB4 | 0xB5 | 0xB6 | 0xB7 | 0xB8 | 0xB9 | 0xBA | 0xBB
            | 0xBC | 0xBD | 0xBE | 0xBF => {
                let source = match opcode & 0b00000111 {
                    0b000 => AddressingModeByte::B,
                    0b001 => AddressingModeByte::C,
                    0b010 => AddressingModeByte::D,
                    0b011 => AddressingModeByte::E,
                    0b100 => AddressingModeByte::H,
                    0b101 => AddressingModeByte::L,
                    0b110 => AddressingModeByte::HlIndirect,
                    0b111 => AddressingModeByte::Accumulator,
                    _ => unreachable!(),
                };

                let cycles = if source.is_indirect() { 8 } else { 4 };

                let instruction_type = match (opcode & 0b00111000) >> 3 {
                    0b000 => InstructionType::AddByte {
                        source,
                        destination: AddressingModeByte::Accumulator,
                    },
                    0b001 => InstructionType::Adc {
                        source,
                        destination: AddressingModeByte::Accumulator,
                    },
                    0b010 => InstructionType::Sub { source },
                    0b011 => InstructionType::Sbc {
                        source,
                        destination: AddressingModeByte::Accumulator,
                    },
                    0b100 => InstructionType::And { source },
                    0b101 => InstructionType::Xor { source },
                    0b110 => InstructionType::Or { source },
                    0b111 => InstructionType::Cp { source },
                    _ => unreachable!(),
                };

                self.pc += 1;
                Instruction {
                    instruction_type,
                    cycles,
                }
            }
            0xC0 | 0xC8 | 0xD0 | 0xD8 => {
                let condition = match (opcode & 0b00011000) >> 3 {
                    0b00 => BranchConditionType::NotZero,
                    0b01 => BranchConditionType::Zero,
                    0b10 => BranchConditionType::NotCarry,
                    0b11 => BranchConditionType::Carry,
                    _ => unreachable!(),
                };

                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Ret {
                        taken_penalty: 12,
                        condition,
                    },
                    cycles: 8,
                }
            }
            0xC1 | 0xD1 | 0xE1 | 0xF1 => {
                let target = match opcode {
                    0xC1 => AddressingModeWord::Bc,
                    0xD1 => AddressingModeWord::De,
                    0xE1 => AddressingModeWord::Hl,
                    0xF1 => AddressingModeWord::Af,
                    _ => unreachable!(),
                };

                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Pop { target },
                    cycles: 12,
                }
            }
            0xC2 | 0xCA | 0xD2 | 0xDA => {
                fn get_branch_condition_type(val: u8) -> BranchConditionType {
                    match val {
                        0b000 => BranchConditionType::NotZero,
                        0b001 => BranchConditionType::Zero,
                        0b010 => BranchConditionType::NotCarry,
                        0b011 => BranchConditionType::Carry,
                        _ => unreachable!(),
                    }
                }

                let condition = get_branch_condition_type((opcode & 0b00111000) >> 3);
                let address = self.read_word_address(self.pc + 1);

                self.pc += 3;
                Instruction {
                    instruction_type: InstructionType::Jp {
                        target: AddressingModeWord::Literal(address),
                        taken_penalty: 4,
                        condition,
                    },
                    cycles: 12,
                }
            }
            0xC3 => {
                let address = self.read_word_address(self.pc + 1);
                self.pc += 3;

                Instruction {
                    instruction_type: InstructionType::Jp {
                        target: AddressingModeWord::Literal(address),
                        taken_penalty: 0,
                        condition: BranchConditionType::Unconditional,
                    },
                    cycles: 16,
                }
            }
            0xC4 | 0xCC | 0xD4 | 0xDC => {
                let address = self.read_word_address(self.pc + 1);
                let condition = match opcode {
                    0xC4 => BranchConditionType::NotZero,
                    0xCC => BranchConditionType::Zero,
                    0xD4 => BranchConditionType::NotCarry,
                    0xDC => BranchConditionType::Carry,
                    _ => unreachable!(),
                };

                self.pc += 3;
                Instruction {
                    instruction_type: InstructionType::Call {
                        address,
                        condition,
                        taken_penalty: 12,
                    },
                    cycles: 12,
                }
            }
            0xC5 | 0xD5 | 0xE5 | 0xF5 => {
                let source = match opcode {
                    0xC5 => AddressingModeWord::Bc,
                    0xD5 => AddressingModeWord::De,
                    0xE5 => AddressingModeWord::Hl,
                    0xF5 => AddressingModeWord::Af,
                    _ => unreachable!(),
                };

                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Push { source },
                    cycles: 16,
                }
            }
            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
                let offset = opcode & 0b00111000;

                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Rst {
                        offset: u16::from(offset),
                    },
                    cycles: 16,
                }
            }
            0xC9 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Ret {
                        taken_penalty: 0,
                        condition: BranchConditionType::Unconditional,
                    },
                    cycles: 16,
                }
            }
            0xCB => {
                let cb_postfix = self.read_byte_address(self.pc + 1);
                let target = match cb_postfix & 0b00000111 {
                    0b000 => AddressingModeByte::B,
                    0b001 => AddressingModeByte::C,
                    0b010 => AddressingModeByte::D,
                    0b011 => AddressingModeByte::E,
                    0b100 => AddressingModeByte::H,
                    0b101 => AddressingModeByte::L,
                    0b110 => AddressingModeByte::HlIndirect,
                    0b111 => AddressingModeByte::Accumulator,
                    _ => unreachable!(),
                };

                let instruction_type = match (cb_postfix & 0b11111000) >> 3 {
                    0b00000 => InstructionType::Rlc { target },
                    0b00001 => InstructionType::Rrc { target },
                    0b00010 => InstructionType::Rl { target },
                    0b00011 => InstructionType::Rr { target },
                    0b00100 => InstructionType::Sla { target },
                    0b00101 => InstructionType::Sra { target },
                    0b00110 => InstructionType::Swap { target },
                    0b00111 => InstructionType::Srl { target },
                    0b01000..=0b01111 => {
                        let bit = (cb_postfix & 0b01110000) >> 4;
                        InstructionType::Bit { target, bit }
                    }
                    0b10000..=0b10111 => {
                        let bit = (cb_postfix & 0b01110000) >> 4;
                        InstructionType::Res { target, bit }
                    }
                    0b11000..=0b11111 => {
                        let bit = (cb_postfix & 0b01110000) >> 4;
                        InstructionType::Set { target, bit }
                    }
                    _ => unreachable!(),
                };

                let cycles = if target.is_indirect() { 16 } else { 8 };

                self.pc += 2;
                Instruction {
                    instruction_type,
                    cycles,
                }
            }
            0xCD => {
                let address = self.read_word_address(self.pc + 1);

                self.pc += 3;
                Instruction {
                    instruction_type: InstructionType::Call {
                        address,
                        taken_penalty: 0,
                        condition: BranchConditionType::Unconditional,
                    },
                    cycles: 24,
                }
            }
            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => {
                let source = AddressingModeByte::Literal(self.read_byte_address(self.pc + 1));

                let instruction_type = match (opcode & 0b00111000) >> 3 {
                    0b000 => InstructionType::AddByte {
                        source,
                        destination: AddressingModeByte::Accumulator,
                    },
                    0b001 => InstructionType::Adc {
                        source,
                        destination: AddressingModeByte::Accumulator,
                    },
                    0b010 => InstructionType::Sub { source },
                    0b011 => InstructionType::Sbc {
                        source,
                        destination: AddressingModeByte::Accumulator,
                    },
                    0b100 => InstructionType::And { source },
                    0b101 => InstructionType::Xor { source },
                    0b110 => InstructionType::Or { source },
                    0b111 => InstructionType::Cp { source },
                    _ => unreachable!(),
                };

                self.pc += 2;
                Instruction {
                    instruction_type,
                    cycles: 8,
                }
            }
            0xD9 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Reti,
                    cycles: 16,
                }
            }
            0xE0 | 0xF0 => {
                let offset = self.read_byte_address(self.pc + 1);
                let address = 0xFF00 + u16::from(offset);
                let (source, destination) = match opcode {
                    0xE0 => (
                        AddressingModeByte::Accumulator,
                        AddressingModeByte::LiteralIndirect(address),
                    ),
                    0xF0 => (
                        AddressingModeByte::LiteralIndirect(address),
                        AddressingModeByte::Accumulator,
                    ),
                    _ => unreachable!(),
                };

                self.pc += 2;
                Instruction {
                    instruction_type: InstructionType::Ldh {
                        source,
                        destination,
                    },
                    cycles: 12,
                }
            }
            0xE8 => {
                let source_value = self.read_byte_address(self.pc + 1);

                self.pc += 2;
                Instruction {
                    instruction_type: InstructionType::AddSp {
                        value: source_value as i8,
                    },
                    cycles: 16,
                }
            }
            0xE9 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Jp {
                        target: AddressingModeWord::Hl,
                        taken_penalty: 0,
                        condition: BranchConditionType::Unconditional,
                    },
                    cycles: 4,
                }
            }
            0xEA | 0xFA => {
                let address = self.read_word_address(self.pc + 1);

                let (source, destination) = match opcode {
                    0xEA => (
                        AddressingModeByte::Accumulator,
                        AddressingModeByte::LiteralIndirect(address),
                    ),
                    0xFA => (
                        AddressingModeByte::LiteralIndirect(address),
                        AddressingModeByte::Accumulator,
                    ),
                    _ => unreachable!(),
                };

                self.pc += 3;
                Instruction {
                    instruction_type: InstructionType::LdByte {
                        source,
                        destination,
                    },
                    cycles: 16,
                }
            }
            0xF3 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Di,
                    cycles: 4,
                }
            }
            0xF8 => {
                let offset = self.read_byte_address(self.pc + 1);

                self.pc += 2;
                Instruction {
                    instruction_type: InstructionType::Ldhl {
                        source: AddressingModeWord::Sp,
                        offset: offset as i8,
                    },
                    cycles: 12,
                }
            }
            0xF9 => {
                self.pc += 1;

                Instruction {
                    instruction_type: InstructionType::LdWord {
                        source: AddressingModeWord::Hl,
                        destination: AddressingModeWord::Sp,
                    },
                    cycles: 8,
                }
            }
            0xFB => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Ei,
                    cycles: 4,
                }
            }
            _ => unreachable!("unknown opcode {:#02X}", opcode),
        }
    }

    pub fn execute(&mut self, instruction: Instruction) {
        match instruction.instruction_type {
            InstructionType::AddByte {
                source,
                destination,
            } => self.execute_add_byte(source, destination),
            InstructionType::AddHl { source } => self.execute_add_hl(source),
            InstructionType::AddSp { value } => self.execute_add_sp(value),
            InstructionType::Adc {
                source,
                destination,
            } => self.execute_adc(source, destination),
            InstructionType::And { source } => self.execute_and(source),
            InstructionType::Bit { target, bit } => self.execute_bit(target, bit),
            InstructionType::Call {
                address,
                taken_penalty,
                condition,
            } => self.execute_call(address, condition),
            InstructionType::Cp { source } => self.execute_cp(source),
            InstructionType::DecByte { target } => self.execute_dec_byte(target),
            InstructionType::DecWord { target } => self.execute_dec_word(target),
            InstructionType::Di => self.interrupt_master_enable = false,
            InstructionType::Ei => self.interrupt_master_enable = true,
            InstructionType::IncByte { target } => self.execute_inc_byte(target),
            InstructionType::IncWord { target } => self.execute_inc_word(target),
            InstructionType::Jp {
                target,
                taken_penalty,
                condition,
            } => self.execute_jp(target, condition),
            InstructionType::Jr {
                offset,
                taken_penalty,
                condition,
            } => self.execute_jr(offset, condition),
            InstructionType::LdByte {
                source,
                destination,
            } => self.execute_ld_byte(source, destination),
            InstructionType::LdWord {
                source,
                destination,
            } => self.execute_ld_word(source, destination),
            InstructionType::Ldh {
                source,
                destination,
            } => self.execute_ldh(source, destination),
            InstructionType::Ldhl { source, offset } => self.execute_ldhl(source, offset),
            InstructionType::Nop => {}
            InstructionType::Or { source } => self.execute_or(source),
            InstructionType::Pop { target } => self.execute_pop(target),
            InstructionType::Push { source } => self.execute_push(source),
            InstructionType::Res { target, bit } => self.execute_res(target, bit),
            InstructionType::Ret {
                taken_penalty,
                condition,
            } => self.execute_ret(condition),
            InstructionType::Reti => self.execute_reti(),
            InstructionType::Rl { target } => self.execute_rl(target),
            InstructionType::Rla => self.execute_rla(),
            InstructionType::Rlc { target } => self.execute_rlc(target),
            InstructionType::Rlca => self.execute_rlca(),
            InstructionType::Rr { target } => self.execute_rr(target),
            InstructionType::Rra => self.execute_rra(),
            InstructionType::Rrc { target } => self.execute_rrc(target),
            InstructionType::Rrca => self.execute_rrca(),
            InstructionType::Rst { offset } => self.execute_rst(offset),
            InstructionType::Sbc {
                source,
                destination,
            } => self.execute_sbc(source, destination),
            InstructionType::Set { target, bit } => self.execute_set(target, bit),
            InstructionType::Sla { target } => self.execute_sla(target),
            InstructionType::Sra { target } => self.execute_sra(target),
            InstructionType::Srl { target } => self.execute_srl(target),
            InstructionType::Sub { source } => self.execute_sub(source),
            InstructionType::Swap { target } => self.execute_swap(target),
            InstructionType::Xor { source } => self.execute_xor(source),
            _ => unreachable!("don't know how to execute:\n{:#x?}", instruction),
        };
    }

    fn execute_add_byte(&mut self, source: AddressingModeByte, destination: AddressingModeByte) {
        let source_value = self.read_byte(source);
        let destination_value = self.read_byte(destination);
        let (result, carry_out) = destination_value.overflowing_add(source_value);
        let half_carry =
            (((source_value & 0b0000_1111) + (destination_value & 0b0000_1111)) & 0b0001_0000) != 0;
        self.write_byte(result, destination);

        self.set_zero_flag(result == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(half_carry);
        self.set_carry_flag(carry_out);
    }

    fn execute_add_hl(&mut self, source: AddressingModeWord) {
        let source_value = self.read_word(source);
        let destination_value = self.read_word(AddressingModeWord::Hl);
        let (result, carry_out) = destination_value.overflowing_add(source_value);
        self.write_word(result, AddressingModeWord::Hl);

        self.set_subtract_flag(false);
        self.set_half_carry_flag(
            (((source_value & 0b0000_1111_1111_1111)
                + (destination_value & 0b0000_1111_1111_1111))
                & 0b0001_0000_0000_0000)
                != 0,
        );
        self.set_carry_flag(carry_out);
    }

    fn execute_add_sp(&mut self, value: i8) {
        let destination_value = self.read_word(AddressingModeWord::Sp);
        let result = destination_value.wrapping_add(i16::from(value) as u16);
        self.write_word(result, AddressingModeWord::Sp);

        // sp += value uses value as a signed value (and negative value correctly
        // affects the entire sp, including carry in from upper byte).
        //
        // Flags are only set from the addition of value to the lower byte of sp.
        // This means that half-carry flag is set if carry from bit 3 -> 4, and
        // carry flag is set if carry out from bit 7. High byte of sp is ignored
        // for both half-carry and carry flags.
        self.set_zero_flag(false);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(
            ((((value as u8) & 0b0000_1111) + ((destination_value as u8) & 0b0000_1111))
                & 0b0001_0000)
                != 0,
        );
        let (_, carry_out) = (destination_value as u8).overflowing_add(value as u8);
        self.set_carry_flag(carry_out);
    }

    fn execute_adc(&mut self, source: AddressingModeByte, destination: AddressingModeByte) {
        let source_value = self.read_byte(source);
        let destination_value = self.read_byte(destination);
        let (result, half_carry, carry) = if self.get_carry_flag() {
            let (intermediate_result, carry_one) = source_value.overflowing_add(destination_value);
            let (result, carry_two) = intermediate_result.overflowing_add(1);
            let half_carry =
                (((source_value & 0b0000_1111) + (destination_value & 0b0000_1111) + 1)
                    & 0b0001_0000)
                    != 0;

            (result, half_carry, carry_one | carry_two)
        } else {
            let (result, carry) = source_value.overflowing_add(destination_value);
            let half_carry = (((source_value & 0b0000_1111) + (destination_value & 0b0000_1111))
                & 0b0001_0000)
                != 0;

            (result, half_carry, carry)
        };

        self.write_byte(result, destination);

        self.set_zero_flag(result == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(half_carry);
        self.set_carry_flag(carry);
    }

    fn execute_and(&mut self, source: AddressingModeByte) {
        let source_value = self.read_byte(source) as u8;
        let destination_value = self.read_byte(AddressingModeByte::Accumulator) & source_value;
        self.write_byte(destination_value, AddressingModeByte::Accumulator);

        self.set_zero_flag(destination_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(true);
        self.set_carry_flag(false);
    }

    fn execute_bit(&mut self, target: AddressingModeByte, bit: u8) {
        let source_value = self.read_byte(target);

        self.set_zero_flag((source_value & (1 << bit)) != 0)
    }

    fn execute_call(&mut self, address: u16, condition: BranchConditionType) {
        if self.should_branch(condition) {
            self.sp -= 2;
            self.write_word_address(self.pc, self.sp);
            self.pc = address;
        }
    }

    fn execute_inc_byte(&mut self, target: AddressingModeByte) {
        let old_value = self.read_byte(target);
        let new_value = old_value.wrapping_add(1);
        self.write_byte(new_value, target);

        self.set_zero_flag(new_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag((old_value & 0b0001_0000) != (new_value & 0b0001_0000));
    }

    fn execute_inc_word(&mut self, target: AddressingModeWord) {
        let old_value = self.read_word(target);
        let new_value = old_value.wrapping_add(1);
        self.write_word(new_value, target);
    }

    fn execute_cp(&mut self, source: AddressingModeByte) {
        let source_value = self.read_byte(source);
        let accumulator_value = self.read_byte(AddressingModeByte::Accumulator);

        self.set_zero_flag(source_value == accumulator_value);
        self.set_subtract_flag(true);
        self.set_half_carry_flag((accumulator_value & 0b0000_1111) < (source_value & 0b0000_1111));
        self.set_carry_flag(accumulator_value < source_value);
    }

    fn execute_dec_byte(&mut self, target: AddressingModeByte) {
        let old_value = self.read_byte(target);
        let new_value = old_value.wrapping_sub(1);
        self.write_byte(new_value, target);

        self.set_zero_flag(new_value == 0);
        self.set_subtract_flag(true);
        self.set_half_carry_flag((old_value & 0b0001_0000) != (new_value & 0b0001_0000));
    }

    fn execute_dec_word(&mut self, target: AddressingModeWord) {
        let old_value = self.read_word(target);
        let new_value = old_value.wrapping_sub(1);
        self.write_word(new_value, target);
    }

    fn execute_ld_byte(&mut self, source: AddressingModeByte, destination: AddressingModeByte) {
        let value = self.read_byte(source);
        self.write_byte(value, destination);
    }

    fn execute_ld_word(&mut self, source: AddressingModeWord, destination: AddressingModeWord) {
        let value = self.read_word(source);
        self.write_word(value, destination);
    }

    fn execute_ldh(&mut self, source: AddressingModeByte, destination: AddressingModeByte) {
        let value = self.read_byte(source);
        self.write_byte(value, destination);
    }

    fn execute_ldhl(&mut self, source: AddressingModeWord, offset: i8) {
        let source_value = self.read_word(source);
        let result_value = source_value.wrapping_add(i16::from(offset) as u16);
        self.write_word(result_value, AddressingModeWord::Hl);

        // sp + offset uses offset as a signed value (and negative value correctly
        // affects the entire sp, including carry in from upper byte).
        //
        // Flags are only set from the addition of offset to the lower byte of sp.
        // This means that half-carry flag is set if carry from bit 3 -> 4, and
        // carry flag is set if carry out from bit 7. High byte of sp is ignored
        // for both half-carry and carry flags.
        self.set_zero_flag(false);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(
            ((((source_value as u8) & 0b0000_1111) + ((offset as u8) & 0b0000_1111)) & 0b0001_0000)
                != 0,
        );
        let (_, carry_out) = (source_value as u8).overflowing_add(offset as u8);
        self.set_carry_flag(carry_out);
    }

    fn execute_jp(&mut self, target: AddressingModeWord, condition: BranchConditionType) {
        if self.should_branch(condition) {
            self.pc = self.read_word(target);
        }
    }

    fn execute_jr(&mut self, offset: i8, condition: BranchConditionType) {
        if self.should_branch(condition) {
            // Signed numbers are stored as 2's complement. Wrapping add after
            // casting to unsigned has same effect as wrapping add of signed to
            // unsigned.
            self.pc = self.pc.wrapping_add(offset as u16);
        }
    }

    fn execute_or(&mut self, source: AddressingModeByte) {
        let source_value = self.read_byte(source);
        let destination_value = self.read_byte(AddressingModeByte::Accumulator);
        let result_value = source_value | destination_value;
        self.write_byte(result_value, AddressingModeByte::Accumulator);

        self.set_zero_flag(result_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag(false);
    }

    fn execute_pop(&mut self, target: AddressingModeWord) {
        let value = self.read_word_address(self.sp);
        self.sp += 2;
        self.write_word(value, target);
    }

    fn execute_push(&mut self, source: AddressingModeWord) {
        let value = self.read_word(source);
        self.sp -= 2;
        self.write_word_address(value, self.sp);
    }

    fn execute_res(&mut self, target: AddressingModeByte, bit: u8) {
        let source_value = self.read_byte(target);
        let result_value = source_value & !(1 << bit);
        self.write_byte(result_value, target);
    }

    fn execute_ret(&mut self, condition: BranchConditionType) {
        if self.should_branch(condition) {
            let return_address = self.read_word_address(self.sp);
            self.sp += 2;
            self.pc = return_address;
        }
    }

    fn execute_reti(&mut self) {
        let return_address = self.read_word_address(self.sp);
        self.sp += 2;
        self.pc = return_address;
    }

    fn execute_rl(&mut self, target: AddressingModeByte) {
        let old_value = self.read_byte(target);
        let new_value = (old_value << 1) | (self.get_carry_flag() as u8);
        self.write_byte(new_value, target);

        self.set_zero_flag(new_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_value & 0b1000_0000) != 0);
    }

    fn execute_rla(&mut self) {
        let old_accumulator = self.read_byte(AddressingModeByte::Accumulator);
        let new_accumulator = (old_accumulator << 1) | (self.get_carry_flag() as u8);
        self.write_byte(new_accumulator, AddressingModeByte::Accumulator);

        self.set_zero_flag(new_accumulator == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_accumulator & 0b1000_0000) != 0);
    }

    fn execute_rlc(&mut self, target: AddressingModeByte) {
        let old_value = self.read_byte(target);
        let new_value = old_value.rotate_left(1);
        self.write_byte(new_value, target);

        self.set_zero_flag(new_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_value & 0b1000_0000) != 0);
    }

    fn execute_rlca(&mut self) {
        let old_accumulator = self.read_byte(AddressingModeByte::Accumulator);
        let new_accumulator = old_accumulator.rotate_left(1);
        self.write_byte(new_accumulator, AddressingModeByte::Accumulator);

        self.set_zero_flag(new_accumulator == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_accumulator & 0b1000_0000) != 0);
    }

    fn execute_rr(&mut self, target: AddressingModeByte) {
        let old_value = self.read_byte(target);
        let new_value = (old_value >> 1) | (self.get_carry_flag() as u8).rotate_right(1);
        self.write_byte(new_value, target);

        self.set_zero_flag(new_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_value & 0b0000_0001) != 0);
    }

    fn execute_rra(&mut self) {
        let old_accumulator = self.read_byte(AddressingModeByte::Accumulator);
        let new_accumulator =
            (old_accumulator >> 1) | (self.get_carry_flag() as u8).rotate_right(1);
        self.write_byte(new_accumulator, AddressingModeByte::Accumulator);

        self.set_zero_flag(new_accumulator == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_accumulator & 0b0000_0001) != 0);
    }

    fn execute_rrc(&mut self, target: AddressingModeByte) {
        let old_value = self.read_byte(target);
        let new_value = old_value.rotate_right(1);
        self.write_byte(new_value, target);

        self.set_zero_flag(new_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_value & 0b0000_0001) != 0);
    }

    fn execute_rrca(&mut self) {
        let old_accumulator = self.read_byte(AddressingModeByte::Accumulator);
        let new_accumulator = old_accumulator.rotate_right(1);
        self.write_byte(new_accumulator, AddressingModeByte::Accumulator);

        self.set_zero_flag(new_accumulator == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_accumulator & 0b0000_0001) != 0);
    }

    fn execute_rst(&mut self, offset: u16) {
        self.sp -= 2;
        self.write_word_address(self.pc, self.sp);
        self.pc = offset;

        // TODO: re-enable interrupts
    }

    // Some gameboy documentation has carry/half-carry documentation backwards for this op.
    // Carry and half-carry flags are set when there is a borrow-in to bit 7 for carry flag,
    // or borrow-in to bit 3 for half-carry flag, respectively.
    fn execute_sbc(&mut self, source: AddressingModeByte, destination: AddressingModeByte) {
        let source_value = self.read_byte(source);
        let destination_value = self.read_byte(destination);

        let (result, half_carry, carry) = if self.get_carry_flag() {
            let (intermediate_result, borrow_one) = destination_value.overflowing_sub(source_value);
            let (result, borrow_two) = intermediate_result.overflowing_sub(1);
            let half_borrow =
                (destination_value & 0b0000_1111) < ((source_value & 0b0000_1111) + 1);

            (result, half_borrow, borrow_one | borrow_two)
        } else {
            let (result, borrow) = destination_value.overflowing_sub(source_value);
            let half_borrow = (destination_value & 0b0000_1111) < (source_value & 0b0000_1111);

            (result, half_borrow, borrow)
        };

        self.write_byte(result, destination);

        self.set_zero_flag(result == 0);
        self.set_subtract_flag(true);
        self.set_half_carry_flag(half_carry);
        self.set_carry_flag(carry);
    }

    fn execute_set(&mut self, target: AddressingModeByte, bit: u8) {
        let old_value = self.read_byte(target);
        let result_value = old_value | (1 << bit);
        self.write_byte(result_value, target);
    }

    fn execute_sla(&mut self, target: AddressingModeByte) {
        let old_value = self.read_byte(target);
        let result_value = old_value << 1;
        self.write_byte(result_value, target);

        self.set_zero_flag(result_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_value & 0b1000_0000) != 0);
    }

    fn execute_sra(&mut self, target: AddressingModeByte) {
        let old_value = self.read_byte(target);
        // Signed right shift performs sign extension.
        let result_value = ((old_value as i8) >> 1) as u8;
        self.write_byte(result_value, target);

        self.set_zero_flag(result_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_value & 0b0000_0001) != 0);
    }

    fn execute_srl(&mut self, target: AddressingModeByte) {
        let old_value = self.read_byte(target);
        // Signed right shift performs sign extension.
        let result_value = old_value >> 1;
        self.write_byte(result_value, target);

        self.set_zero_flag(result_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_value & 0b0000_0001) != 0);
    }

    // Some gameboy documentation has carry/half-carry documentation backwards for this op.
    // Carry and half-carry flags are set when there is a borrow-in to bit 7 for carry flag,
    // or borrow-in to bit 3 for half-carry flag, respectively.
    fn execute_sub(&mut self, source: AddressingModeByte) {
        let source_value = self.read_byte(source);
        let destination_value = self.read_byte(AddressingModeByte::Accumulator);
        let (result_value, carry_in) = destination_value.overflowing_sub(source_value);
        let half_carry_in = (destination_value & 0b0000_1111) < (source_value & 0b0000_1111);

        self.write_byte(result_value, AddressingModeByte::Accumulator);

        self.set_zero_flag(result_value == 0);
        self.set_subtract_flag(true);
        self.set_half_carry_flag(half_carry_in);
        self.set_carry_flag(carry_in);
    }
    fn execute_swap(&mut self, target: AddressingModeByte) {
        let source_value = self.read_byte(target);
        // Original low nibble will be shifted out when shifting right, and likewise,
        // original high nibble will be shifted out when shifting left.
        let result_value = (source_value >> 4) | (source_value << 4);
        self.write_byte(result_value, target);

        self.set_zero_flag(result_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag(false);
    }

    fn execute_xor(&mut self, source: AddressingModeByte) {
        let source_value = self.read_byte(source);
        let result_value = self.read_byte(AddressingModeByte::Accumulator) ^ source_value;
        self.write_byte(result_value, AddressingModeByte::Accumulator);

        self.set_zero_flag(result_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag(false);
    }

    fn should_branch(&self, condition: BranchConditionType) -> bool {
        match condition {
            BranchConditionType::NotZero => !self.get_zero_flag(),
            BranchConditionType::NotCarry => !self.get_carry_flag(),
            BranchConditionType::Zero => self.get_zero_flag(),
            BranchConditionType::Carry => self.get_carry_flag(),
            BranchConditionType::Unconditional => true,
        }
    }
}

impl Cpu {
    const ZERO_FLAG_MASK: u16 = 0b00000000_1000_0000;
    const SUBTRACT_FLAG_MASK: u16 = 0b00000000_0100_0000;
    const HALF_CARRY_FLAG_MASK: u16 = 0b00000000_0010_0000;
    const CARRY_FLAG_MASK: u16 = 0b00000000_0001_0000;

    fn get_zero_flag(&self) -> bool {
        (self.af & Self::ZERO_FLAG_MASK) != 0
    }

    fn get_subtract_flag(&self) -> bool {
        (self.af & Self::SUBTRACT_FLAG_MASK) != 0
    }

    fn get_half_carry_flag(&self) -> bool {
        (self.af & Self::HALF_CARRY_FLAG_MASK) != 0
    }

    fn get_carry_flag(&self) -> bool {
        (self.af & Self::CARRY_FLAG_MASK) != 0
    }

    fn set_zero_flag(&mut self, set: bool) {
        if set {
            self.af |= Self::ZERO_FLAG_MASK;
        } else {
            self.af &= !Self::ZERO_FLAG_MASK;
        }
    }

    fn set_subtract_flag(&mut self, set: bool) {
        if set {
            self.af |= Self::SUBTRACT_FLAG_MASK;
        } else {
            self.af &= !Self::SUBTRACT_FLAG_MASK;
        }
    }

    fn set_half_carry_flag(&mut self, set: bool) {
        if set {
            self.af |= Self::HALF_CARRY_FLAG_MASK;
        } else {
            self.af &= !Self::HALF_CARRY_FLAG_MASK;
        }
    }

    fn set_carry_flag(&mut self, set: bool) {
        if set {
            self.af |= Self::CARRY_FLAG_MASK;
        } else {
            self.af &= !Self::CARRY_FLAG_MASK;
        }
    }
}
