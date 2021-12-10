use std::fmt::{Debug, Display};

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
    halted: bool,
    stopped: bool,
    m_cycles_completed: u8,
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
            .field("halted", &self.halted)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Instruction {
    pub instruction_type: InstructionType,
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
        target: AddressingModeWord,
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
        condition: BranchConditionType,
    },
    JpHl,
    Jr {
        unsigned_offset: AddressingModeByte,
        condition: BranchConditionType,
    },
    LdByte {
        source: AddressingModeByte,
        destination: AddressingModeByte,
    },
    LdSp {
        source: AddressingModeWord,
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

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.instruction_type {
            InstructionType::Adc {
                destination,
                source,
            } => write!(f, "adc {}, {}", destination, source),
            InstructionType::AddByte {
                destination,
                source,
            } => write!(f, "add {}, {}", destination, source),
            InstructionType::AddHl { source } => write!(f, "add hl, {}", source),
            InstructionType::AddSp { value } => write!(f, "add sp, ${:02x}", value),
            InstructionType::And { source } => write!(f, "and a, {}", source),
            InstructionType::Bit { bit, target } => write!(f, "bit {}, {}", bit, target),
            InstructionType::Call {
                target: address,
                condition,
                ..
            } => match condition {
                BranchConditionType::Unconditional => write!(f, "call {}", address),
                _ => write!(f, "call {}, {}", condition, address),
            },
            InstructionType::Ccf => f.write_str("ccf"),
            InstructionType::Cp { source } => write!(f, "cp a, {}", source),
            InstructionType::Cpl => f.write_str("cpl"),
            InstructionType::Daa => f.write_str("daa"),
            InstructionType::DecByte { target } => write!(f, "dec {}", target),
            InstructionType::DecWord { target } => write!(f, "dec {}", target),
            InstructionType::Di => f.write_str("di"),
            InstructionType::Ei => f.write_str("ei"),
            InstructionType::Halt => f.write_str("halt"),
            InstructionType::IncByte { target } => write!(f, "inc {}", target),
            InstructionType::IncWord { target } => write!(f, "inc {}", target),
            InstructionType::Jp {
                condition, target, ..
            } => match condition {
                BranchConditionType::Unconditional => write!(f, "jp {}", target),
                _ => write!(f, "jp {}, {}", condition, target),
            },
            InstructionType::JpHl => f.write_str("jp hl"),
            InstructionType::Jr {
                condition,
                unsigned_offset,
            } => match condition {
                BranchConditionType::Unconditional => write!(f, "jr {:+}", unsigned_offset),
                _ => write!(f, "jr {}, {:+}", condition, unsigned_offset),
            },
            InstructionType::LdByte {
                destination,
                source,
            } => write!(f, "ld {}, {}", destination, source),
            InstructionType::LdSp { source } => write!(f, "ld sp, {}", source),
            InstructionType::LdWord {
                source,
                destination,
            } => write!(f, "ld {}, {}", destination, source),
            InstructionType::Ldhl { offset, source } => {
                write!(f, "ld hl, {}{:+02}", source, offset)
            }
            InstructionType::Nop => f.write_str("nop"),
            InstructionType::Or { source } => write!(f, "or a, {}", source),
            InstructionType::Pop { target } => write!(f, "pop {}", target),
            InstructionType::Push { source } => write!(f, "push {}", source),
            InstructionType::Res { bit, target } => write!(f, "res {}, {}", bit, target),
            InstructionType::Ret { condition, .. } => match condition {
                BranchConditionType::Unconditional => f.write_str("ret"),
                _ => write!(f, "ret {}", condition),
            },
            InstructionType::Reti => f.write_str("reti"),
            InstructionType::Rl { target } => write!(f, "rl {}", target),
            InstructionType::Rla => f.write_str("rla"),
            InstructionType::Rlc { target } => write!(f, "rlc {}", target),
            InstructionType::Rlca => f.write_str("rlca"),
            InstructionType::Rr { target } => write!(f, "rr {}", target),
            InstructionType::Rra => f.write_str("rra"),
            InstructionType::Rrc { target } => write!(f, "rrc {}", target),
            InstructionType::Rrca => f.write_str("rrca"),
            InstructionType::Rst { offset } => write!(f, "rst ${:02x}", offset),
            InstructionType::Sbc {
                destination,
                source,
            } => write!(f, "sbc {}, {}", destination, source),
            InstructionType::Scf => f.write_str("scf"),
            InstructionType::Set { bit, target } => write!(f, "{}, {}", bit, target),
            InstructionType::Sla { target } => write!(f, "sla {}", target),
            InstructionType::Sra { target } => write!(f, "sra {}", target),
            InstructionType::Srl { target } => write!(f, "srl {}", target),
            InstructionType::Stop => f.write_str("stop"),
            InstructionType::Sub { source } => write!(f, "sub a, {} ", source),
            InstructionType::Swap { target } => write!(f, "swap {}", target),
            InstructionType::Xor { source } => write!(f, "xor {}", source),
        }
    }
}

impl Display for AddressingModeByte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressingModeByte::Accumulator => f.write_str("a"),
            AddressingModeByte::B => f.write_str("b"),
            AddressingModeByte::BcIndirect => f.write_str("[bc]"),
            AddressingModeByte::C => f.write_str("c"),
            AddressingModeByte::CIndirect => f.write_str("[c + ff00]"),
            AddressingModeByte::D => f.write_str("d"),
            AddressingModeByte::DeIndirect => f.write_str("[de]"),
            AddressingModeByte::E => f.write_str("e"),
            AddressingModeByte::H => f.write_str("h"),
            AddressingModeByte::HlIndirect => f.write_str("[hl]"),
            AddressingModeByte::HlIndirectDecrement => f.write_str("[hl-]"),
            AddressingModeByte::HlIndirectIncrement => f.write_str("[hl+]"),
            AddressingModeByte::L => f.write_str("l"),
            AddressingModeByte::Literal(val) => write!(f, "${:02x}", val),
            AddressingModeByte::LiteralIndirect(val) => write!(f, "[${:04x}]", val),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BranchConditionType {
    NotZero,
    NotCarry,
    Zero,
    Carry,
    Unconditional,
}

impl Display for BranchConditionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BranchConditionType::NotZero => f.write_str("nz"),
            BranchConditionType::NotCarry => f.write_str("nc"),
            BranchConditionType::Zero => f.write_str("z"),
            BranchConditionType::Carry => f.write_str("c"),
            BranchConditionType::Unconditional => f.write_str("unconditional"),
        }
    }
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

impl Display for AddressingModeWord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressingModeWord::Af => f.write_str("af"),
            AddressingModeWord::Bc => f.write_str("bc"),
            AddressingModeWord::De => f.write_str("de"),
            AddressingModeWord::Hl => f.write_str("hl"),
            AddressingModeWord::Sp => f.write_str("sp"),
            AddressingModeWord::Literal(val) => write!(f, "${:04x}", val),
            AddressingModeWord::LiteralIndirect(val) => write!(f, "[${:04x}]", val),
        }
    }
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
            halted: false,
            stopped: false,
            m_cycles_completed: 0,
        }
    }
}

impl Cpu {
    // Runs a single fetch/decode/execute cycle. Returns the number of t-cycles elapsed during this execution.
    pub fn fetch_decode_execute(&mut self) -> u8 {
        if self.stopped {
            self.delay_m_cycle();
        } else if self.halted {
            self.delay_m_cycle();

            // If currently halted, check to see if ongoing halt is finished.
            if self.bus.halt_finished() {
                self.halted = false;
            }
        } else if let Some(interrupt_type) = self.bus.poll_interrupt() {
            self.handle_interrupt(interrupt_type);
        } else {
            // let start_pc = self.pc;
            // let info_string = format!("af: 0x{:04x} bc: 0x{:04x}, de: 0x{:04x}, hl: 0x{:04x}, IME: {} IE: 0b{:08b} IF: 0b{:08b} dot: {} timer counter: {} tick counter: 0b{:016b}", self.af, self.bc, self.de, self.hl, self.bus.interrupt_master_enable, self.bus.interrupt_enable, self.bus.interrupt_flag, self.bus.ppu.dot, self.bus.timer.timer_counter, self.bus.timer.tick_counter);
            let decoded = self.decode();
            // println!("{:04x}: {} {}", start_pc, decoded, info_string);
            self.execute(decoded);
        }

        let m_cycles_completed = self.m_cycles_completed;
        self.m_cycles_completed = 0;
        match self.bus.get_current_speed() {
            SpeedMode::Normal => m_cycles_completed * 4,
            SpeedMode::Double => m_cycles_completed * 2,
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
                self.delay_m_cycle(); // dereference (FF00 + C)
                self.bus.read_byte_address(address)
            }
            AddressingModeByte::BcIndirect => {
                self.delay_m_cycle(); // dereference bc
                self.bus.read_byte_address(self.bc)
            }
            AddressingModeByte::DeIndirect => {
                self.delay_m_cycle(); // dereference de
                self.bus.read_byte_address(self.de)
            }
            AddressingModeByte::HlIndirect => {
                self.delay_m_cycle(); // dereference hl
                self.bus.read_byte_address(self.hl)
            }
            AddressingModeByte::HlIndirectIncrement => {
                self.delay_m_cycle(); // dereference hl
                let result = self.bus.read_byte_address(self.hl);
                self.hl = self.hl.wrapping_add(1);
                result
            }
            AddressingModeByte::HlIndirectDecrement => {
                self.delay_m_cycle(); // derefence hl
                let result = self.bus.read_byte_address(self.hl);
                self.hl = self.hl.wrapping_sub(1);
                result
            }
            AddressingModeByte::Literal(val) => val,
            AddressingModeByte::LiteralIndirect(address) => {
                self.delay_m_cycle(); // dereference address
                self.bus.read_byte_address(address)
            }
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
            AddressingModeWord::LiteralIndirect(address) => {
                self.delay_m_cycle(); // read data LSB
                let lsb_byte = self.bus.read_byte_address(address);
                self.delay_m_cycle(); // read data MSB
                let msb_byte = self.bus.read_byte_address(address + 1);

                u16::from_be_bytes([msb_byte, lsb_byte])
            }
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
                self.delay_m_cycle(); // dereference (FF00 + C)
                let address = 0xFF00 + u16::from(self.read_byte(AddressingModeByte::C));
                self.bus.write_byte_address(val, address);
            }
            AddressingModeByte::BcIndirect => {
                self.delay_m_cycle(); // dereference bc
                self.bus.write_byte_address(val, self.bc);
            }
            AddressingModeByte::DeIndirect => {
                self.delay_m_cycle(); // dereference de
                self.bus.write_byte_address(val, self.de);
            }
            AddressingModeByte::HlIndirect => {
                self.delay_m_cycle(); // dereference hl
                self.bus.write_byte_address(val, self.hl);
            }
            AddressingModeByte::HlIndirectIncrement => {
                self.delay_m_cycle(); // dereference hl
                self.bus.write_byte_address(val, self.hl);
                self.hl = self.hl.wrapping_add(1)
            }
            AddressingModeByte::HlIndirectDecrement => {
                self.delay_m_cycle(); // dereference hl
                self.bus.write_byte_address(val, self.hl);
                self.hl = self.hl.wrapping_sub(1)
            }
            AddressingModeByte::Literal(_) => unreachable!(),
            AddressingModeByte::LiteralIndirect(address) => {
                self.delay_m_cycle(); // write data byte
                self.bus.write_byte_address(val, address);
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
                let [msb_data, lsb_data] = val.to_be_bytes();
                self.delay_m_cycle(); // write data LSB
                self.bus.write_byte_address(lsb_data, address);
                self.delay_m_cycle(); // write data MSB
                self.bus.write_byte_address(msb_data, address + 1);
            }
        }
    }

    fn delay_m_cycle(&mut self) {
        self.bus.step_m_cycle();
        self.m_cycles_completed += 1;
    }

    fn decode(&mut self) -> Instruction {
        let opcode = self.read_byte(AddressingModeByte::LiteralIndirect(self.pc));
        // print!("opcode: {:02x} ", opcode);

        match opcode {
            0x00 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Nop,
                }
            }
            0x01 | 0x11 | 0x21 | 0x31 => {
                let source_value = self.read_word(AddressingModeWord::LiteralIndirect(self.pc + 1));
                let source = AddressingModeWord::Literal(source_value);
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
                Instruction { instruction_type }
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

                self.pc += 1;
                let instruction_type = match opcode & 0b00000111 {
                    0b100 => InstructionType::IncByte { target },
                    0b101 => InstructionType::DecByte { target },
                    _ => unreachable!(),
                };

                Instruction { instruction_type }
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
                let n = self.read_byte(AddressingModeByte::LiteralIndirect(self.pc + 1));

                self.pc += 2;
                Instruction {
                    instruction_type: InstructionType::LdByte {
                        source: AddressingModeByte::Literal(n),
                        destination: r,
                    },
                }
            }
            0x07 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Rlca,
                }
            }
            0x08 => {
                let destination_address =
                    self.read_word(AddressingModeWord::LiteralIndirect(self.pc + 1));
                let destination = AddressingModeWord::LiteralIndirect(destination_address);

                self.pc += 3;
                Instruction {
                    instruction_type: InstructionType::LdWord {
                        source: AddressingModeWord::Sp,
                        destination,
                    },
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
                }
            }
            0x0F => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Rrca,
                }
            }
            0x10 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Stop,
                }
            }
            0x17 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Rla,
                }
            }
            0x18 => {
                let unsigned_offset_value =
                    self.read_byte(AddressingModeByte::LiteralIndirect(self.pc + 1));
                let unsigned_offset = AddressingModeByte::Literal(unsigned_offset_value);

                self.pc += 2;
                Instruction {
                    instruction_type: InstructionType::Jr {
                        unsigned_offset,
                        condition: BranchConditionType::Unconditional,
                    },
                }
            }
            0x1F => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Rra,
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

                let unsigned_offset_value =
                    self.read_byte(AddressingModeByte::LiteralIndirect(self.pc + 1));
                let unsigned_offset = AddressingModeByte::Literal(unsigned_offset_value);

                self.pc += 2;

                Instruction {
                    instruction_type: InstructionType::Jr {
                        unsigned_offset,
                        condition,
                    },
                }
            }
            0x27 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Daa,
                }
            }
            0x2F => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Cpl,
                }
            }
            0x37 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Scf,
                }
            }
            0x3F => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Ccf,
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

                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::LdByte {
                        source,
                        destination,
                    },
                }
            }
            0x76 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Halt,
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
                Instruction { instruction_type }
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
                    instruction_type: InstructionType::Ret { condition },
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
                let target_address =
                    self.read_word(AddressingModeWord::LiteralIndirect(self.pc + 1));
                let target = AddressingModeWord::Literal(target_address);

                self.pc += 3;
                Instruction {
                    instruction_type: InstructionType::Jp { target, condition },
                }
            }
            0xC3 => {
                let target_address =
                    self.read_word(AddressingModeWord::LiteralIndirect(self.pc + 1));
                let target = AddressingModeWord::Literal(target_address);
                self.pc += 3;

                Instruction {
                    instruction_type: InstructionType::Jp {
                        target,
                        condition: BranchConditionType::Unconditional,
                    },
                }
            }
            0xC4 | 0xCC | 0xD4 | 0xDC => {
                let target_address =
                    self.read_word(AddressingModeWord::LiteralIndirect(self.pc + 1));
                let target = AddressingModeWord::Literal(target_address);

                let condition = match opcode {
                    0xC4 => BranchConditionType::NotZero,
                    0xCC => BranchConditionType::Zero,
                    0xD4 => BranchConditionType::NotCarry,
                    0xDC => BranchConditionType::Carry,
                    _ => unreachable!(),
                };

                self.pc += 3;
                Instruction {
                    instruction_type: InstructionType::Call { target, condition },
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
                }
            }
            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
                let offset = opcode & 0b00111000;

                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Rst {
                        offset: u16::from(offset),
                    },
                }
            }
            0xC9 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Ret {
                        condition: BranchConditionType::Unconditional,
                    },
                }
            }
            0xCB => {
                let cb_postfix = self.read_byte(AddressingModeByte::LiteralIndirect(self.pc + 1));

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

                self.pc += 2;

                Instruction { instruction_type }
            }
            0xCD => {
                let target_address =
                    self.read_word(AddressingModeWord::LiteralIndirect(self.pc + 1));
                let target = AddressingModeWord::Literal(target_address);

                self.pc += 3;
                Instruction {
                    instruction_type: InstructionType::Call {
                        target,
                        condition: BranchConditionType::Unconditional,
                    },
                }
            }
            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => {
                let source_value = self.read_byte(AddressingModeByte::LiteralIndirect(self.pc + 1));
                let source = AddressingModeByte::Literal(source_value);

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
                Instruction { instruction_type }
            }
            0xD9 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Reti,
                }
            }
            0xE0 | 0xF0 => {
                let offset = self.read_byte(AddressingModeByte::LiteralIndirect(self.pc + 1));
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
                }
            }
            0xE8 => {
                let source_value = self.read_byte(AddressingModeByte::LiteralIndirect(self.pc + 1));

                self.pc += 2;
                Instruction {
                    instruction_type: InstructionType::AddSp {
                        value: source_value as i8,
                    },
                }
            }
            0xE9 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::JpHl,
                }
            }
            0xEA | 0xFA => {
                let address = self.read_word(AddressingModeWord::LiteralIndirect(self.pc + 1));

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
                }
            }
            0xF3 => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Di,
                }
            }
            0xF8 => {
                let offset = self.read_byte(AddressingModeByte::LiteralIndirect(self.pc + 1));

                self.pc += 2;
                Instruction {
                    instruction_type: InstructionType::Ldhl {
                        source: AddressingModeWord::Sp,
                        offset: offset as i8,
                    },
                }
            }
            0xF9 => {
                self.pc += 1;

                Instruction {
                    instruction_type: InstructionType::LdSp {
                        source: AddressingModeWord::Hl,
                    },
                }
            }
            0xFB => {
                self.pc += 1;
                Instruction {
                    instruction_type: InstructionType::Ei,
                }
            }
            _ => unreachable!("unknown opcode 0x{:02X}, PC: 0x{:02X}", opcode, self.pc),
        }
    }

    fn execute(&mut self, instruction: Instruction) {
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
                target: address,
                condition,
            } => self.execute_call(address, condition),
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
            InstructionType::Jp { target, condition } => self.execute_jp(target, condition),
            InstructionType::JpHl => self.execute_jp_hl(),
            InstructionType::Jr {
                unsigned_offset,
                condition,
            } => self.execute_jr(unsigned_offset, condition),
            InstructionType::LdByte {
                source,
                destination,
            } => self.execute_ld_byte(source, destination),
            InstructionType::LdSp { source } => self.execute_ld_sp(source),
            InstructionType::LdWord {
                source,
                destination,
            } => self.execute_ld_word(source, destination),
            InstructionType::Ldhl { source, offset } => self.execute_ldhl(source, offset),
            InstructionType::Nop => {}
            InstructionType::Or { source } => self.execute_or(source),
            InstructionType::Pop { target } => self.execute_pop(target),
            InstructionType::Push { source } => self.execute_push(source),
            InstructionType::Res { target, bit } => self.execute_res(target, bit),
            InstructionType::Ret { condition } => self.execute_ret(condition),
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
        }
    }

    fn handle_interrupt(&mut self, interrupt_type: InterruptType) {
        self.delay_m_cycle(); // interrupt wait state (CPU likely executing NOPs)
        self.delay_m_cycle(); // interrupt wait state (CPU likely executing NOPs)

        let [pc_msb, pc_lsb] = self.pc.to_be_bytes();
        self.sp -= 1;
        self.write_byte(pc_msb, AddressingModeByte::LiteralIndirect(self.sp));
        self.sp -= 1;
        self.write_byte(pc_lsb, AddressingModeByte::LiteralIndirect(self.sp));

        self.delay_m_cycle(); // set PC?
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

        // Takes extra cycle for the add to propogate to upper byte
        self.delay_m_cycle();
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

        // Takes extra cycle for add to propgate to upper byte.
        self.delay_m_cycle();
        let result = destination_value.wrapping_add(i16::from(value) as u16);

        // internal (likely delay when writing to sp).
        self.delay_m_cycle();
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

        self.set_zero_flag((source_value & (1 << bit)) == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(true);
    }

    fn execute_call(&mut self, address: AddressingModeWord, condition: BranchConditionType) {
        let call_address = self.read_word(address);

        if self.should_branch(condition) {
            self.delay_m_cycle(); // internal

            let [pc_msb, pc_lsb] = self.pc.to_be_bytes();
            self.sp -= 1;
            self.write_byte(pc_msb, AddressingModeByte::LiteralIndirect(self.sp));
            self.sp -= 1;
            self.write_byte(pc_lsb, AddressingModeByte::LiteralIndirect(self.sp));

            self.pc = call_address;
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

        // Takes extra cycle for add to propogate to upper byte.
        self.delay_m_cycle();
        let new_value = old_value.wrapping_add(1);
        self.write_word(new_value, target);
    }

    fn execute_ccf(&mut self) {
        let old_carry_flag = self.get_carry_flag();
        let new_carry_flag = !old_carry_flag;

        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag(new_carry_flag);
    }

    fn execute_cp(&mut self, source: AddressingModeByte) {
        let source_value = self.read_byte(source);
        let accumulator_value = self.read_byte(AddressingModeByte::Accumulator);

        self.set_zero_flag(source_value == accumulator_value);
        self.set_subtract_flag(true);
        self.set_half_carry_flag((accumulator_value & 0b0000_1111) < (source_value & 0b0000_1111));
        self.set_carry_flag(accumulator_value < source_value);
    }

    fn execute_cpl(&mut self) {
        let source_value = self.read_byte(AddressingModeByte::Accumulator);
        self.write_byte(!source_value, AddressingModeByte::Accumulator);

        self.set_subtract_flag(true);
        self.set_half_carry_flag(true);
    }

    fn execute_daa(&mut self) {
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

        // Takes extra cycle for dec to propogate to upper byte.
        self.delay_m_cycle();
        let new_value = old_value.wrapping_sub(1);
        self.write_word(new_value, target);
    }

    fn execute_di(&mut self) {
        self.bus.set_interrupt_master_enable(false);
    }

    fn execute_ei(&mut self) {
        self.bus.set_interrupt_master_enable(true);
    }

    fn execute_halt(&mut self) {
        self.halted = true;
    }

    fn execute_ld_byte(&mut self, source: AddressingModeByte, destination: AddressingModeByte) {
        let value = self.read_byte(source);
        self.write_byte(value, destination);
    }

    fn execute_ld_sp(&mut self, source: AddressingModeWord) {
        let value = self.read_word(source);

        // internal (likely delay when writing to sp).
        self.delay_m_cycle();
        self.write_word(value, AddressingModeWord::Sp);
    }

    fn execute_ld_word(&mut self, source: AddressingModeWord, destination: AddressingModeWord) {
        let value = self.read_word(source);
        self.write_word(value, destination);
    }

    fn execute_ldhl(&mut self, source: AddressingModeWord, offset: i8) {
        let source_value = self.read_word(source);

        // internal (likely waiting for add to propogate to upper byte).
        self.delay_m_cycle();
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
        let new_address = self.read_word(target);

        if self.should_branch(condition) {
            self.delay_m_cycle();
            self.pc = new_address;
        }
    }

    fn execute_jp_hl(&mut self) {
        let new_address = self.read_word(AddressingModeWord::Hl);
        self.pc = new_address;
    }

    fn execute_jr(&mut self, unsigned_offset: AddressingModeByte, condition: BranchConditionType) {
        let signed_offset = self.read_byte(unsigned_offset) as i8;
        if self.should_branch(condition) {
            // Signed numbers are stored as 2's complement. Wrapping add after
            // casting to unsigned has same effect as wrapping add of signed to
            // unsigned.
            self.delay_m_cycle(); // internal modify PC
            self.pc = self.pc.wrapping_add(signed_offset as u16);
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
        let value_lsb = self.read_byte(AddressingModeByte::LiteralIndirect(self.sp));
        self.sp += 1;
        let value_msb = self.read_byte(AddressingModeByte::LiteralIndirect(self.sp));
        self.sp += 1;

        let value = u16::from_be_bytes([value_msb, value_lsb]);
        self.write_word(value, target);
    }

    fn execute_push(&mut self, source: AddressingModeWord) {
        let value = self.read_word(source);
        let [value_msb, value_lsb] = value.to_be_bytes();

        self.delay_m_cycle(); // internal

        self.sp -= 1;
        self.write_byte(value_msb, AddressingModeByte::LiteralIndirect(self.sp));
        self.sp -= 1;
        self.write_byte(value_lsb, AddressingModeByte::LiteralIndirect(self.sp));
    }

    fn execute_res(&mut self, target: AddressingModeByte, bit: u8) {
        let source_value = self.read_byte(target);
        let result_value = source_value & !(1 << bit);
        self.write_byte(result_value, target);
    }

    fn execute_ret(&mut self, condition: BranchConditionType) {
        if matches!(condition, BranchConditionType::Unconditional) {
            let lsb_return_address = self.read_byte(AddressingModeByte::LiteralIndirect(self.sp));
            self.sp += 1;
            let msb_return_address = self.read_byte(AddressingModeByte::LiteralIndirect(self.sp));
            self.sp += 1;

            let return_address = u16::from_be_bytes([msb_return_address, lsb_return_address]);

            self.delay_m_cycle(); // set PC?
            self.pc = return_address;
        } else {
            self.delay_m_cycle(); // branch decision?
            if self.should_branch(condition) {
                let lsb_return_address =
                    self.read_byte(AddressingModeByte::LiteralIndirect(self.sp));
                self.sp += 1;
                let msb_return_address =
                    self.read_byte(AddressingModeByte::LiteralIndirect(self.sp));
                self.sp += 1;

                let return_address = u16::from_be_bytes([msb_return_address, lsb_return_address]);

                self.delay_m_cycle(); // set PC?
                self.pc = return_address;
            }
        }
    }

    fn execute_reti(&mut self) {
        let lsb_return_address = self.read_byte(AddressingModeByte::LiteralIndirect(self.sp));
        self.sp += 1;
        let msb_return_address = self.read_byte(AddressingModeByte::LiteralIndirect(self.sp));
        self.sp += 1;

        let return_address = u16::from_be_bytes([msb_return_address, lsb_return_address]);

        self.delay_m_cycle(); // set PC?
        self.pc = return_address;

        self.bus.set_interrupt_master_enable(true);
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

        // The manual states that the zero flag is set when the result is zero, but
        // other documentation states that the zero flag is unconditionally reset.
        //
        // The zero flag being unconditionally reset passes blargg's cpu tests
        // (whereas conditionally setting the zero flag fails).
        self.set_zero_flag(false);
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

        // The manual states that the zero flag is set when the result is zero, but
        // other documentation states that the zero flag is unconditionally reset.
        //
        // The zero flag being unconditionally reset passes blargg's cpu tests
        // (whereas conditionally setting the zero flag fails).
        self.set_zero_flag(false);
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

        // The manual states that the zero flag is set when the result is zero, but
        // other documentation states that the zero flag is unconditionally reset.
        //
        // The zero flag being unconditionally reset passes blargg's cpu tests
        // (whereas conditionally setting the zero flag fails).
        self.set_zero_flag(false);
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

        // The manual states that the zero flag is set when the result is zero, but
        // other documentation states that the zero flag is unconditionally reset.
        //
        // The zero flag being unconditionally reset passes blargg's cpu tests
        // (whereas conditionally setting the zero flag fails).
        self.set_zero_flag(false);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag((old_accumulator & 0b0000_0001) != 0);
    }

    fn execute_rst(&mut self, offset: u16) {
        self.delay_m_cycle(); // internal

        let [pc_msb, pc_lsb] = self.pc.to_be_bytes();
        self.sp -= 1;
        self.write_byte(pc_msb, AddressingModeByte::LiteralIndirect(self.sp));
        self.sp -= 1;
        self.write_byte(pc_lsb, AddressingModeByte::LiteralIndirect(self.sp));

        self.pc = offset;
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

    fn execute_scf(&mut self) {
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag(true);
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

    fn execute_stop(&mut self) {
        // If the bus does not handle this stop (by performing a speed switch),
        // we need to stop until the next user input is received.
        self.stopped = !self.bus.maybe_handle_stop();
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
