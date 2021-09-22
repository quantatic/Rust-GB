#[derive(Debug)]
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

#[derive(Clone, Copy, Debug)]
pub struct Instruction {
    pub instruction_type: InstructionType,
    pub cycles: u16,
    opcode: u8,
}

#[derive(Clone, Copy, Debug)]
pub enum InstructionType {
    AddByte {
        source: AddressingModeByte,
        destination: AddressingModeByte,
    },
    AddWord {
        source: AddressingModeWord,
        destination: AddressingModeWord,
    },
    Adc {
        source: AddressingModeByte,
        destination: AddressingModeByte,
    },
    And {
        source: AddressingModeByte,
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
    Halt,
    IncByte {
        target: AddressingModeByte,
    },
    IncWord {
        target: AddressingModeWord,
    },
    Jp {
        address: u16,
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
    Ret {
        taken_penalty: u16,
        condition: BranchConditionType,
    },
    Sbc {
        source: AddressingModeByte,
        destination: AddressingModeByte,
    },
    Sub {
        source: AddressingModeByte,
        destination: AddressingModeByte,
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
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            af: 0,
            bc: 0,
            de: 0,
            hl: 0,
            sp: 0,
            pc: 0,
            interrupt_master_enable: false,
            memory: [0; 0x10000],
        }
    }
}

impl Cpu {
    fn read_byte_address(&self, address: u16) -> u8 {
        self.memory[usize::from(address)]
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
        }
    }

    fn write_byte_address(&mut self, value: u8, address: u16) {
        self.memory[usize::from(address)] = value;
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
                    opcode,
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
                    opcode,
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
                    opcode,
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
                    opcode,
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
                    opcode,
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
                    opcode,
                }
            }
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
                fn get_addressing_mode(val: u8) -> AddressingModeByte {
                    match val {
                        0b111 => AddressingModeByte::Accumulator,
                        0b000 => AddressingModeByte::B,
                        0b001 => AddressingModeByte::C,
                        0b101 => AddressingModeByte::D,
                        0b011 => AddressingModeByte::E,
                        0b100 => AddressingModeByte::H,
                        0b110 => AddressingModeByte::HlIndirect,
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
                    opcode,
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
                    opcode,
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
                    opcode,
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
                        0b111 => AddressingModeByte::Accumulator,
                        0b000 => AddressingModeByte::B,
                        0b001 => AddressingModeByte::C,
                        0b101 => AddressingModeByte::D,
                        0b011 => AddressingModeByte::E,
                        0b100 => AddressingModeByte::H,
                        0b110 => AddressingModeByte::HlIndirect,
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
                    opcode,
                }
            }
            0x76 => Instruction {
                instruction_type: InstructionType::Halt,
                cycles: 4,
                opcode,
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
                    0b010 => InstructionType::Sub {
                        source,
                        destination: AddressingModeByte::Accumulator,
                    },
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

                Instruction {
                    instruction_type,
                    cycles,
                    opcode,
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

                Instruction {
                    instruction_type: InstructionType::Ret {
                        taken_penalty: 12,
                        condition,
                    },
                    cycles: 8,
                    opcode,
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
                    opcode,
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
                        address,
                        taken_penalty: 4,
                        condition,
                    },
                    cycles: 12,
                    opcode,
                }
            }
            0xC3 => {
                let address = self.read_word_address(self.pc + 1);
                self.pc += 3;

                Instruction {
                    instruction_type: InstructionType::Jp {
                        address,
                        taken_penalty: 0,
                        condition: BranchConditionType::Unconditional,
                    },
                    cycles: 16,
                    opcode,
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
                    opcode,
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
                    opcode,
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
                    opcode,
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
                    0b010 => InstructionType::Sub {
                        source,
                        destination: AddressingModeByte::Accumulator,
                    },
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
                    opcode,
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
                    opcode,
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
                    opcode,
                }
            }
            0xF3 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Di,
                    cycles: 4,
                    opcode,
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
            InstructionType::AddWord {
                source,
                destination,
            } => self.execute_add_word(source, destination),
            InstructionType::Adc {
                source,
                destination,
            } => self.execute_adc(source, destination),
            InstructionType::And { source } => self.execute_and(source),
            InstructionType::Call {
                address,
                taken_penalty,
                condition,
            } => self.execute_call(address, condition),
            InstructionType::DecByte { target } => self.execute_dec_byte(target),
            InstructionType::DecWord { target } => self.execute_dec_word(target),
            InstructionType::Di => self.interrupt_master_enable = false,
            InstructionType::IncByte { target } => self.execute_inc_byte(target),
            InstructionType::IncWord { target } => self.execute_inc_word(target),
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
            InstructionType::Jp {
                address,
                taken_penalty,
                condition,
            } => self.execute_jp(address, condition),
            InstructionType::Jr {
                offset,
                taken_penalty,
                condition,
            } => self.execute_jr(offset, condition),
            InstructionType::Nop => {}
            InstructionType::Pop { target } => self.execute_pop(target),
            InstructionType::Push { source } => self.execute_push(source),
            InstructionType::Ret {
                taken_penalty,
                condition,
            } => self.execute_ret(condition),
            _ => unreachable!("don't know how to execute:\n{:#x?}", instruction),
        };
    }

    fn execute_add_byte(&mut self, source: AddressingModeByte, destination: AddressingModeByte) {
        let source_value = self.read_byte(source);
        let destination_value = self.read_byte(destination);
        let result = source_value.wrapping_add(destination_value);
        self.write_byte(result, destination);
    }

    fn execute_add_word(&mut self, source: AddressingModeWord, destination: AddressingModeWord) {
        let source_value = self.read_word(source);
        let destination_value = self.read_word(destination);
        let result = source_value.wrapping_add(destination_value);
        self.write_word(result, destination);
    }

    fn execute_adc(&mut self, source: AddressingModeByte, destination: AddressingModeByte) {
        let source_value = self.read_byte(source);
        let destination_value = self.read_byte(destination);
        let result = source_value
            .wrapping_add(destination_value)
            .wrapping_add(self.get_carry_flag() as u8);
        self.write_byte(result, destination);
    }

    fn execute_and(&mut self, source: AddressingModeByte) {
        let source_value = self.read_byte(source) as u8;
        let destination_value = self.read_byte(AddressingModeByte::Accumulator) & source_value;
        self.write_byte(destination_value, AddressingModeByte::Accumulator);
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
        self.set_sign_flag(false);
        self.set_half_carry_flag((old_value & 0b00001000 != 0) && (new_value & 0b00001000 == 0));
    }

    fn execute_inc_word(&mut self, target: AddressingModeWord) {
        let old_value = self.read_word(target);
        let new_value = old_value + 1;
        self.write_word(new_value, target);
    }

    fn execute_dec_byte(&mut self, target: AddressingModeByte) {
        let old_value = self.read_byte(target);
        let new_value = old_value.wrapping_sub(1);
        self.write_byte(new_value, target);

        self.set_zero_flag(new_value == 0);
        self.set_sign_flag(true);
        self.set_half_carry_flag(!((old_value & 0b00001000 == 0) && (new_value & 0b00001000 != 0)));
    }

    fn execute_dec_word(&mut self, target: AddressingModeWord) {
        let old_value = self.read_word(target);
        let new_value = old_value + 1;
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

    fn execute_jp(&mut self, address: u16, condition: BranchConditionType) {
        if self.should_branch(condition) {
            self.pc = address;
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

    fn execute_ret(&mut self, condition: BranchConditionType) {
        if self.should_branch(condition) {
            let return_address = self.read_word_address(self.sp);
            self.sp += 2;
            self.pc = return_address;
        }
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
    const SIGN_FLAG_MASK: u16 = 0b00000000_0100_0000;
    const HALF_CARRY_FLAG_MASK: u16 = 0b00000000_0010_0000;
    const CARRY_FLAG_MASK: u16 = 0b00000000_0001_0000;

    fn get_zero_flag(&self) -> bool {
        (self.af & Self::ZERO_FLAG_MASK) != 0
    }

    fn get_sign_flag(&self) -> bool {
        (self.af & Self::SIGN_FLAG_MASK) != 0
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

    fn set_sign_flag(&mut self, set: bool) {
        if set {
            self.af |= Self::SIGN_FLAG_MASK;
        } else {
            self.af &= !Self::SIGN_FLAG_MASK;
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
