use std::fmt::Debug;

use crate::{
    bus::{Bus, InterruptType, SpeedMode},
    cartridge::Cartridge,
    joypad::Button,
};

#[derive(Clone)]
pub struct Cpu {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    sp: u16,
    pc: u16,
    pub bus: Bus,
    cycles_delay: u8,
    halted: bool,
    stopped: bool,
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
            .field("cycles_delay", &self.cycles_delay)
            .field("halted", &self.halted)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Instruction {
    pub instruction_type: InstructionType,
    pub cycles: u8,
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
        taken_penalty: u8,
        condition: BranchConditionType,
    },
    Ccf,
    Cp {
        source: AddressingModeByte,
    },
    Cpl,
    Daa,
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
        taken_penalty: u8,
        condition: BranchConditionType,
    },
    Jr {
        offset: i8,
        taken_penalty: u8,
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
        taken_penalty: u8,
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
    Scf,
    Set {
        target: AddressingModeByte,
        bit: u8,
    },
    Sla {
        target: AddressingModeByte,
    },
    Sra {
        target: AddressingModeByte,
    },
    Srl {
        target: AddressingModeByte,
    },
    Stop,
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

#[derive(Clone, Copy, Debug)]
pub enum RegisterByte {
    Accumulator,
    B,
    C,
    D,
    E,
    H,
    L,
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
    CIndirect,
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
            AddressingModeByte::CIndirect => true,
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

impl Cpu {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            af: 0x0000,
            bc: 0x0000,
            de: 0x0000,
            hl: 0x0000,
            sp: 0x0000,
            pc: 0x0000,
            bus: Bus::new(cartridge),
            cycles_delay: 0,
            halted: false,
            stopped: false,
        }
    }
}

impl Cpu {
    pub fn step(&mut self) {
        if self.stopped {
            return;
        }

        let cycles_to_execute = match self.bus.get_current_speed() {
            SpeedMode::Normal => 1,
            SpeedMode::Double => 2,
        };

        self.bus.step();

        for _ in 0..cycles_to_execute {
            if self.cycles_delay == 0 {
                // If currently halted, check to see if ongoing halt is finished. If not, bail.
                if self.halted {
                    if self.bus.halt_finished() {
                        self.halted = false;
                    } else {
                        return;
                    }
                }

                if let Some(interrupt_type) = self.bus.poll_interrupt() {
                    self.handle_interrupt(interrupt_type);
                    self.cycles_delay = 20;
                } else {
                    let decoded = self.decode();
                    self.cycles_delay = self.execute(decoded);
                }
            }

            self.cycles_delay -= 1;
        }
    }

    pub fn set_button_pressed(&mut self, button: Button, pressed: bool) {
        if pressed {
            self.stopped = false;
        }

        match button {
            Button::A => self.bus.joypad.set_a_pressed(pressed),
            Button::B => self.bus.joypad.set_b_pressed(pressed),
            Button::Start => self.bus.joypad.set_start_pressed(pressed),
            Button::Select => self.bus.joypad.set_select_pressed(pressed),
            Button::Up => self.bus.joypad.set_up_pressed(pressed),
            Button::Down => self.bus.joypad.set_down_pressed(pressed),
            Button::Left => self.bus.joypad.set_left_pressed(pressed),
            Button::Right => self.bus.joypad.set_right_pressed(pressed),
        }
    }

    #[cfg(test)]
    pub fn read_register(&self, register: RegisterByte) -> u8 {
        match register {
            RegisterByte::Accumulator => (self.af >> 8) as u8,
            RegisterByte::B => (self.bc >> 8) as u8,
            RegisterByte::C => self.bc as u8,
            RegisterByte::D => (self.de >> 8) as u8,
            RegisterByte::E => self.de as u8,
            RegisterByte::H => (self.hl >> 8) as u8,
            RegisterByte::L => self.hl as u8,
        }
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
            AddressingModeByte::CIndirect => {
                let address = 0xFF00 + u16::from(self.read_byte(AddressingModeByte::C));
                self.bus.read_byte_address(address)
            }
            AddressingModeByte::BcIndirect => self.bus.read_byte_address(self.bc),
            AddressingModeByte::DeIndirect => self.bus.read_byte_address(self.de),
            AddressingModeByte::HlIndirect => self.bus.read_byte_address(self.hl),
            AddressingModeByte::HlIndirectIncrement => {
                let result = self.bus.read_byte_address(self.hl);
                self.hl = self.hl.wrapping_add(1);
                result
            }
            AddressingModeByte::HlIndirectDecrement => {
                let result = self.bus.read_byte_address(self.hl);
                self.hl = self.hl.wrapping_sub(1);
                result
            }
            AddressingModeByte::Literal(val) => val,
            AddressingModeByte::LiteralIndirect(address) => self.bus.read_byte_address(address),
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
            AddressingModeWord::LiteralIndirect(address) => self.bus.read_word_address(address),
        }
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
            AddressingModeByte::CIndirect => {
                let address = 0xFF00 + u16::from(self.read_byte(AddressingModeByte::C));
                self.bus.write_byte_address(val, address);
            }
            AddressingModeByte::BcIndirect => {
                self.bus.write_byte_address(val, self.bc);
            }
            AddressingModeByte::DeIndirect => {
                self.bus.write_byte_address(val, self.de);
            }
            AddressingModeByte::HlIndirect => {
                self.bus.write_byte_address(val, self.hl);
            }
            AddressingModeByte::HlIndirectIncrement => {
                self.bus.write_byte_address(val, self.hl);
                self.hl = self.hl.wrapping_add(1)
            }
            AddressingModeByte::HlIndirectDecrement => {
                self.bus.write_byte_address(val, self.hl);
                self.hl = self.hl.wrapping_sub(1)
            }
            AddressingModeByte::Literal(_) => unreachable!(),
            AddressingModeByte::LiteralIndirect(address) => {
                self.bus.write_byte_address(val, address)
            }
        }
    }

    // Bottom byte of Af (bottom byte of flags register) is always masked as 0,
    // even if we try to write a non-zero value to it.
    fn write_word(&mut self, val: u16, location: AddressingModeWord) {
        match location {
            AddressingModeWord::Af => self.af = val & 0xFFF0,
            AddressingModeWord::Bc => self.bc = val,
            AddressingModeWord::De => self.de = val,
            AddressingModeWord::Hl => self.hl = val,
            AddressingModeWord::Sp => self.sp = val,
            AddressingModeWord::Literal(_) => unreachable!(),
            AddressingModeWord::LiteralIndirect(address) => {
                self.bus.write_word_address(val, address)
            }
        }
    }

    fn decode(&mut self) -> Instruction {
        let opcode = self.bus.read_byte_address(self.pc);
        match opcode {
            0x00 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Nop,
                    cycles: 4,
                }
            }
            0x01 | 0x11 | 0x21 | 0x31 => {
                let source = AddressingModeWord::Literal(self.bus.read_word_address(self.pc + 1));
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
                let n = self.bus.read_byte_address(self.pc + 1);
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
                let address = self.bus.read_word_address(self.pc + 1);

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
            0x10 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Stop,
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
                let offset = self.bus.read_byte_address(self.pc + 1) as i8;
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
                let offset = self.bus.read_byte_address(self.pc + 1) as i8;

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
            0x27 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Daa,
                    cycles: 4,
                }
            }
            0x2F => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Cpl,
                    cycles: 4,
                }
            }
            0x37 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Scf,
                    cycles: 4,
                }
            }
            0x3F => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Ccf,
                    cycles: 4,
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
            0x76 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Halt,
                    cycles: 4,
                }
            }
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
                let address = self.bus.read_word_address(self.pc + 1);

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
                let address = self.bus.read_word_address(self.pc + 1);
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
                let address = self.bus.read_word_address(self.pc + 1);
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
                let cb_postfix = self.bus.read_byte_address(self.pc + 1);
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
                        let bit = (cb_postfix & 0b00111000) >> 3;
                        InstructionType::Bit { target, bit }
                    }
                    0b10000..=0b10111 => {
                        let bit = (cb_postfix & 0b00111000) >> 3;
                        InstructionType::Res { target, bit }
                    }
                    0b11000..=0b11111 => {
                        let bit = (cb_postfix & 0b00111000) >> 3;
                        InstructionType::Set { target, bit }
                    }
                    _ => unreachable!(),
                };

                let cycles = if target.is_indirect() {
                    if matches!(instruction_type, InstructionType::Bit { .. }) {
                        12
                    } else {
                        16
                    }
                } else {
                    8
                };

                self.pc += 2;
                Instruction {
                    instruction_type,
                    cycles,
                }
            }
            0xCD => {
                let address = self.bus.read_word_address(self.pc + 1);

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
                let source = AddressingModeByte::Literal(self.bus.read_byte_address(self.pc + 1));

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
                let offset = self.bus.read_byte_address(self.pc + 1);
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
                    instruction_type: InstructionType::LdByte {
                        source,
                        destination,
                    },
                    cycles: 12,
                }
            }
            0xE2 | 0xF2 => {
                let (source, destination) = match opcode {
                    0xE2 => (
                        AddressingModeByte::Accumulator,
                        AddressingModeByte::CIndirect,
                    ),
                    0xF2 => (
                        AddressingModeByte::CIndirect,
                        AddressingModeByte::Accumulator,
                    ),
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
            0xE8 => {
                let source_value = self.bus.read_byte_address(self.pc + 1);

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
                let address = self.bus.read_word_address(self.pc + 1);

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
                let offset = self.bus.read_byte_address(self.pc + 1);

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
            _ => unreachable!("unknown opcode 0x{:02X}, PC: 0x{:02X}", opcode, self.pc),
        }
    }

    fn execute(&mut self, instruction: Instruction) -> u8 {
        let branch_penalty = match instruction.instruction_type {
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
            } => self.execute_call(address, condition, taken_penalty),
            InstructionType::Ccf => self.execute_ccf(),
            InstructionType::Cp { source } => self.execute_cp(source),
            InstructionType::Cpl => self.execute_cpl(),
            InstructionType::Daa => self.execute_daa(),
            InstructionType::DecByte { target } => self.execute_dec_byte(target),
            InstructionType::DecWord { target } => self.execute_dec_word(target),
            InstructionType::Di => self.execute_di(),
            InstructionType::Ei => self.execute_ei(),
            InstructionType::Halt => self.execute_halt(),
            InstructionType::IncByte { target } => self.execute_inc_byte(target),
            InstructionType::IncWord { target } => self.execute_inc_word(target),
            InstructionType::Jp {
                target,
                taken_penalty,
                condition,
            } => self.execute_jp(target, condition, taken_penalty),
            InstructionType::Jr {
                offset,
                taken_penalty,
                condition,
            } => self.execute_jr(offset, condition, taken_penalty),
            InstructionType::LdByte {
                source,
                destination,
            } => self.execute_ld_byte(source, destination),
            InstructionType::LdWord {
                source,
                destination,
            } => self.execute_ld_word(source, destination),
            InstructionType::Ldhl { source, offset } => self.execute_ldhl(source, offset),
            InstructionType::Nop => 0,
            InstructionType::Or { source } => self.execute_or(source),
            InstructionType::Pop { target } => self.execute_pop(target),
            InstructionType::Push { source } => self.execute_push(source),
            InstructionType::Res { target, bit } => self.execute_res(target, bit),
            InstructionType::Ret {
                taken_penalty,
                condition,
            } => self.execute_ret(condition, taken_penalty),
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
            InstructionType::Scf => self.execute_scf(),
            InstructionType::Set { target, bit } => self.execute_set(target, bit),
            InstructionType::Sla { target } => self.execute_sla(target),
            InstructionType::Sra { target } => self.execute_sra(target),
            InstructionType::Srl { target } => self.execute_srl(target),
            InstructionType::Stop => self.execute_stop(),
            InstructionType::Sub { source } => self.execute_sub(source),
            InstructionType::Swap { target } => self.execute_swap(target),
            InstructionType::Xor { source } => self.execute_xor(source),
        };

        instruction.cycles + branch_penalty
    }

    fn handle_interrupt(&mut self, interrupt_type: InterruptType) {
        self.sp -= 2;
        self.bus.write_word_address(self.pc, self.sp);
        self.pc = match interrupt_type {
            InterruptType::VBlank => 0x40,
            InterruptType::LcdStat => 0x48,
            InterruptType::Timer => 0x50,
            InterruptType::Serial => 0x58,
            InterruptType::Joypad => 0x60,
        };
    }
}

impl Cpu {
    fn execute_add_byte(
        &mut self,
        source: AddressingModeByte,
        destination: AddressingModeByte,
    ) -> u8 {
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

        0
    }

    fn execute_add_hl(&mut self, source: AddressingModeWord) -> u8 {
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

        0
    }

    fn execute_add_sp(&mut self, value: i8) -> u8 {
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

        0
    }

    fn execute_adc(&mut self, source: AddressingModeByte, destination: AddressingModeByte) -> u8 {
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

        0
    }

    fn execute_and(&mut self, source: AddressingModeByte) -> u8 {
        let source_value = self.read_byte(source) as u8;
        let destination_value = self.read_byte(AddressingModeByte::Accumulator) & source_value;
        self.write_byte(destination_value, AddressingModeByte::Accumulator);

        self.set_zero_flag(destination_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(true);
        self.set_carry_flag(false);

        0
    }

    fn execute_bit(&mut self, target: AddressingModeByte, bit: u8) -> u8 {
        let source_value = self.read_byte(target);

        self.set_zero_flag((source_value & (1 << bit)) == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(true);

        0
    }

    fn execute_call(
        &mut self,
        address: u16,
        condition: BranchConditionType,
        branch_penalty: u8,
    ) -> u8 {
        if self.should_branch(condition) {
            self.sp -= 2;
            self.bus.write_word_address(self.pc, self.sp);
            self.pc = address;

            branch_penalty
        } else {
            0
        }
    }

    fn execute_inc_byte(&mut self, target: AddressingModeByte) -> u8 {
        let old_value = self.read_byte(target);
        let new_value = old_value.wrapping_add(1);
        self.write_byte(new_value, target);

        self.set_zero_flag(new_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag((old_value & 0b0001_0000) != (new_value & 0b0001_0000));

        0
    }

    fn execute_inc_word(&mut self, target: AddressingModeWord) -> u8 {
        let old_value = self.read_word(target);
        let new_value = old_value.wrapping_add(1);
        self.write_word(new_value, target);

        0
    }

    fn execute_ccf(&mut self) -> u8 {
        let old_carry_flag = self.get_carry_flag();
        let new_carry_flag = !old_carry_flag;

        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag(new_carry_flag);

        0
    }

    fn execute_cp(&mut self, source: AddressingModeByte) -> u8 {
        let source_value = self.read_byte(source);
        let accumulator_value = self.read_byte(AddressingModeByte::Accumulator);

        self.set_zero_flag(source_value == accumulator_value);
        self.set_subtract_flag(true);
        self.set_half_carry_flag((accumulator_value & 0b0000_1111) < (source_value & 0b0000_1111));
        self.set_carry_flag(accumulator_value < source_value);

        0
    }

    fn execute_cpl(&mut self) -> u8 {
        let source_value = self.read_byte(AddressingModeByte::Accumulator);
        self.write_byte(!source_value, AddressingModeByte::Accumulator);

        self.set_subtract_flag(true);
        self.set_half_carry_flag(true);

        0
    }

    fn execute_daa(&mut self) -> u8 {
        let source_value = self.read_byte(AddressingModeByte::Accumulator);
        let mut result_value = source_value;

        if self.get_subtract_flag() {
            if self.get_carry_flag() {
                result_value = result_value.wrapping_sub(0x60);
            }

            if self.get_half_carry_flag() {
                result_value = result_value.wrapping_sub(0x06);
            }
        } else {
            if self.get_carry_flag() || (source_value > 0x99) {
                result_value = result_value.wrapping_add(0x60);
                self.set_carry_flag(true);
            }

            if self.get_half_carry_flag() || ((source_value & 0x0F) > 0x09) {
                result_value = result_value.wrapping_add(0x06);
            }
        }

        self.write_byte(result_value, AddressingModeByte::Accumulator);

        self.set_zero_flag(result_value == 0);
        self.set_half_carry_flag(false);
        // carry flag already (possibly) set above

        0
    }

    fn execute_dec_byte(&mut self, target: AddressingModeByte) -> u8 {
        let old_value = self.read_byte(target);
        let new_value = old_value.wrapping_sub(1);
        self.write_byte(new_value, target);

        self.set_zero_flag(new_value == 0);
        self.set_subtract_flag(true);
        self.set_half_carry_flag((old_value & 0b0001_0000) != (new_value & 0b0001_0000));

        0
    }

    fn execute_dec_word(&mut self, target: AddressingModeWord) -> u8 {
        let old_value = self.read_word(target);
        let new_value = old_value.wrapping_sub(1);
        self.write_word(new_value, target);

        0
    }

    fn execute_di(&mut self) -> u8 {
        self.bus.set_interrupt_master_enable(false);

        0
    }

    fn execute_ei(&mut self) -> u8 {
        self.bus.set_interrupt_master_enable(true);

        0
    }

    fn execute_halt(&mut self) -> u8 {
        self.halted = true;

        0
    }

    fn execute_ld_byte(
        &mut self,
        source: AddressingModeByte,
        destination: AddressingModeByte,
    ) -> u8 {
        let value = self.read_byte(source);
        self.write_byte(value, destination);

        0
    }

    fn execute_ld_word(
        &mut self,
        source: AddressingModeWord,
        destination: AddressingModeWord,
    ) -> u8 {
        let value = self.read_word(source);
        self.write_word(value, destination);

        0
    }

    fn execute_ldhl(&mut self, source: AddressingModeWord, offset: i8) -> u8 {
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

        0
    }

    fn execute_jp(
        &mut self,
        target: AddressingModeWord,
        condition: BranchConditionType,
        branch_penalty: u8,
    ) -> u8 {
        if self.should_branch(condition) {
            self.pc = self.read_word(target);

            branch_penalty
        } else {
            0
        }
    }

    fn execute_jr(&mut self, offset: i8, condition: BranchConditionType, branch_penalty: u8) -> u8 {
        if self.should_branch(condition) {
            // Signed numbers are stored as 2's complement. Wrapping add after
            // casting to unsigned has same effect as wrapping add of signed to
            // unsigned.
            self.pc = self.pc.wrapping_add(offset as u16);

            branch_penalty
        } else {
            0
        }
    }

    fn execute_or(&mut self, source: AddressingModeByte) -> u8 {
        let source_value = self.read_byte(source);
        let destination_value = self.read_byte(AddressingModeByte::Accumulator);
        let result_value = source_value | destination_value;
        self.write_byte(result_value, AddressingModeByte::Accumulator);

        self.set_zero_flag(result_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag(false);

        0
    }

    fn execute_pop(&mut self, target: AddressingModeWord) -> u8 {
        let value = self.bus.read_word_address(self.sp);
        self.sp += 2;
        self.write_word(value, target);

        0
    }

    fn execute_push(&mut self, source: AddressingModeWord) -> u8 {
        let value = self.read_word(source);
        self.sp -= 2;
        self.bus.write_word_address(value, self.sp);

        0
    }

    fn execute_res(&mut self, target: AddressingModeByte, bit: u8) -> u8 {
        let source_value = self.read_byte(target);
        let result_value = source_value & !(1 << bit);
        self.write_byte(result_value, target);

        0
    }

    fn execute_ret(&mut self, condition: BranchConditionType, branch_penalty: u8) -> u8 {
        if self.should_branch(condition) {
            let return_address = self.bus.read_word_address(self.sp);
            self.sp += 2;
            self.pc = return_address;

            branch_penalty
        } else {
            0
        }
    }

    fn execute_reti(&mut self) -> u8 {
        let return_address = self.bus.read_word_address(self.sp);
        self.sp += 2;
        self.pc = return_address;
        self.bus.set_interrupt_master_enable(true);

        0
    }

    fn execute_rl(&mut self, target: AddressingModeByte) -> u8 {
        let old_value = self.read_byte(target);
        let new_value = (old_value << 1) | (self.get_carry_flag() as u8);
        self.write_byte(new_value, target);

        self.set_zero_flag(new_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_value & 0b1000_0000) != 0);

        0
    }

    fn execute_rla(&mut self) -> u8 {
        let old_accumulator = self.read_byte(AddressingModeByte::Accumulator);
        let new_accumulator = (old_accumulator << 1) | (self.get_carry_flag() as u8);
        self.write_byte(new_accumulator, AddressingModeByte::Accumulator);

        // The manual states that the zero flag is set when the result is zero, but
        // other documentation states that the zero flag is unconditionally reset.
        //
        // The zero flag being unconditionally reset passes blargg's cpu tests
        // (whereas conditionally setting the zero flag fails).
        self.set_zero_flag(false);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_accumulator & 0b1000_0000) != 0);

        0
    }

    fn execute_rlc(&mut self, target: AddressingModeByte) -> u8 {
        let old_value = self.read_byte(target);
        let new_value = old_value.rotate_left(1);
        self.write_byte(new_value, target);

        self.set_zero_flag(new_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_value & 0b1000_0000) != 0);

        0
    }

    fn execute_rlca(&mut self) -> u8 {
        let old_accumulator = self.read_byte(AddressingModeByte::Accumulator);
        let new_accumulator = old_accumulator.rotate_left(1);
        self.write_byte(new_accumulator, AddressingModeByte::Accumulator);

        // The manual states that the zero flag is set when the result is zero, but
        // other documentation states that the zero flag is unconditionally reset.
        //
        // The zero flag being unconditionally reset passes blargg's cpu tests
        // (whereas conditionally setting the zero flag fails).
        self.set_zero_flag(false);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_accumulator & 0b1000_0000) != 0);

        0
    }

    fn execute_rr(&mut self, target: AddressingModeByte) -> u8 {
        let old_value = self.read_byte(target);
        let new_value = (old_value >> 1) | (self.get_carry_flag() as u8).rotate_right(1);
        self.write_byte(new_value, target);

        self.set_zero_flag(new_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_value & 0b0000_0001) != 0);

        0
    }

    fn execute_rra(&mut self) -> u8 {
        let old_accumulator = self.read_byte(AddressingModeByte::Accumulator);
        let new_accumulator =
            (old_accumulator >> 1) | (self.get_carry_flag() as u8).rotate_right(1);
        self.write_byte(new_accumulator, AddressingModeByte::Accumulator);

        // The manual states that the zero flag is set when the result is zero, but
        // other documentation states that the zero flag is unconditionally reset.
        //
        // The zero flag being unconditionally reset passes blargg's cpu tests
        // (whereas conditionally setting the zero flag fails).
        self.set_zero_flag(false);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_accumulator & 0b0000_0001) != 0);

        0
    }

    fn execute_rrc(&mut self, target: AddressingModeByte) -> u8 {
        let old_value = self.read_byte(target);
        let new_value = old_value.rotate_right(1);
        self.write_byte(new_value, target);

        self.set_zero_flag(new_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_value & 0b0000_0001) != 0);

        0
    }

    fn execute_rrca(&mut self) -> u8 {
        let old_accumulator = self.read_byte(AddressingModeByte::Accumulator);
        let new_accumulator = old_accumulator.rotate_right(1);
        self.write_byte(new_accumulator, AddressingModeByte::Accumulator);

        // The manual states that the zero flag is set when the result is zero, but
        // other documentation states that the zero flag is unconditionally reset.
        //
        // The zero flag being unconditionally reset passes blargg's cpu tests
        // (whereas conditionally setting the zero flag fails).
        self.set_zero_flag(false);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_accumulator & 0b0000_0001) != 0);

        0
    }

    fn execute_rst(&mut self, offset: u16) -> u8 {
        self.sp -= 2;
        self.bus.write_word_address(self.pc, self.sp);
        self.pc = offset;

        0
    }

    // Some gameboy documentation has carry/half-carry documentation backwards for this op.
    // Carry and half-carry flags are set when there is a borrow-in to bit 7 for carry flag,
    // or borrow-in to bit 3 for half-carry flag, respectively.
    fn execute_sbc(&mut self, source: AddressingModeByte, destination: AddressingModeByte) -> u8 {
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

        0
    }

    fn execute_scf(&mut self) -> u8 {
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag(true);

        0
    }

    fn execute_set(&mut self, target: AddressingModeByte, bit: u8) -> u8 {
        let old_value = self.read_byte(target);
        let result_value = old_value | (1 << bit);
        self.write_byte(result_value, target);

        0
    }

    fn execute_sla(&mut self, target: AddressingModeByte) -> u8 {
        let old_value = self.read_byte(target);
        let result_value = old_value << 1;
        self.write_byte(result_value, target);

        self.set_zero_flag(result_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_value & 0b1000_0000) != 0);

        0
    }

    fn execute_sra(&mut self, target: AddressingModeByte) -> u8 {
        let old_value = self.read_byte(target);
        // Signed right shift performs sign extension.
        let result_value = ((old_value as i8) >> 1) as u8;
        self.write_byte(result_value, target);

        self.set_zero_flag(result_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_value & 0b0000_0001) != 0);

        0
    }

    fn execute_srl(&mut self, target: AddressingModeByte) -> u8 {
        let old_value = self.read_byte(target);
        // Signed right shift performs sign extension.
        let result_value = old_value >> 1;
        self.write_byte(result_value, target);

        self.set_zero_flag(result_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_value & 0b0000_0001) != 0);

        0
    }

    fn execute_stop(&mut self) -> u8 {
        // If the bus does not handle this stop (by performing a speed switch),
        // we need to stop until the next user input is received.
        self.stopped = !self.bus.maybe_handle_stop();

        0
    }

    // Some gameboy documentation has carry/half-carry documentation backwards for this op.
    // Carry and half-carry flags are set when there is a borrow-in to bit 7 for carry flag,
    // or borrow-in to bit 3 for half-carry flag, respectively.
    fn execute_sub(&mut self, source: AddressingModeByte) -> u8 {
        let source_value = self.read_byte(source);
        let destination_value = self.read_byte(AddressingModeByte::Accumulator);
        let (result_value, carry_in) = destination_value.overflowing_sub(source_value);
        let half_carry_in = (destination_value & 0b0000_1111) < (source_value & 0b0000_1111);

        self.write_byte(result_value, AddressingModeByte::Accumulator);

        self.set_zero_flag(result_value == 0);
        self.set_subtract_flag(true);
        self.set_half_carry_flag(half_carry_in);
        self.set_carry_flag(carry_in);

        0
    }
    fn execute_swap(&mut self, target: AddressingModeByte) -> u8 {
        let source_value = self.read_byte(target);
        // Original low nibble will be shifted out when shifting right, and likewise,
        // original high nibble will be shifted out when shifting left.
        let result_value = (source_value >> 4) | (source_value << 4);
        self.write_byte(result_value, target);

        self.set_zero_flag(result_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag(false);

        0
    }

    fn execute_xor(&mut self, source: AddressingModeByte) -> u8 {
        let source_value = self.read_byte(source);
        let result_value = self.read_byte(AddressingModeByte::Accumulator) ^ source_value;
        self.write_byte(result_value, AddressingModeByte::Accumulator);

        self.set_zero_flag(result_value == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag(false);

        0
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
