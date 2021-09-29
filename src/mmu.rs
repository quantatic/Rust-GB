use crate::timer::Timer;

#[derive(Clone, Copy, Debug)]
pub enum InterruptType {
    VBlank,
    LcdStat,
    Timer,
    Serial,
    Joypad,
}

pub struct Mmu {
    interrupt_enable: u8,
    interrupt_flag: u8,
    interrupt_master_enable: bool,
    pub memory: [u8; 0x10000],
    low_ram: [u8; 0x2000],
    high_ram: [u8; 0x7F],
    video_ram: [u8; 0x2000],
    pub timer: Timer,
}

impl Default for Mmu {
    fn default() -> Self {
        let mut memory = [0; 0x10000];
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
            interrupt_enable: Default::default(),
            interrupt_flag: Default::default(),
            interrupt_master_enable: Default::default(),
            memory,
            low_ram: [0; 8192],
            high_ram: [0; 127],
            video_ram: [0; 8192],
            timer: Default::default(),
        }
    }
}

impl Mmu {
    pub fn read_byte_address(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.memory[usize::from(address)],
            0x4000..=0x7FFF => self.memory[usize::from(address)],
            0x8000..=0x9FFF => {
                let vram_offset = address - 0x8000;
                self.video_ram[usize::from(vram_offset)]
            }
            0xC000..=0xDFFF => {
                let ram_offset = address - 0xC000;
                self.low_ram[usize::from(ram_offset)]
            }
            0xFF0F => self.interrupt_flag,
            0xFF44 => {
                eprintln!("reading from unimplemented LY");
                0
            }
            0xFF80..=0xFFFE => {
                let ram_offset = address - 0xFF80;
                self.high_ram[usize::from(ram_offset)]
            }
            0xFFFF => self.interrupt_enable,
            _ => todo!("read from 0x{:02X}", address),
        }
    }

    pub fn read_word_address(&self, address: u16) -> u16 {
        let low = self.read_byte_address(address);
        let high = self.read_byte_address(address + 1);
        u16::from(low) | (u16::from(high) << 8)
    }

    pub fn write_byte_address(&mut self, value: u8, address: u16) {
        if address == 0xFF01 {
            print!("{}", char::from(value));
        }

        match address {
            0x0000..=0x3FFF => self.memory[usize::from(address)] = value,
            0x4000..=0x7FFF => self.memory[usize::from(address)] = value,
            0x8000..=0x9FFF => {
                let vram_offset = address - 0x8000;
                self.video_ram[usize::from(vram_offset)] = value;
            }
            0xC000..=0xDFFF => {
                let ram_offset = address - 0xC000;
                self.low_ram[usize::from(ram_offset)] = value;
            }
            0xFF01 => eprintln!("writing 0x{:02X} to unimplemented SB", value),
            0xFF02 => eprintln!("writing 0x{:02X} to unimplemented SC", value),
            0xFF04 => self.timer.set_divider_register(value),
            0xFF05 => self.timer.set_timer_counter(value),
            0xFF06 => self.timer.set_timer_modulo(value),
            0xFF07 => self.timer.set_timer_control(value),
            0xFF0F => self.interrupt_flag = value & 0b0001_1111,
            0xFF24 => eprintln!("writing 0x{:02X} to unimplemented NR50", value),
            0xFF25 => eprintln!("writing 0x{:02X} to unimplemented NR51", value),
            0xFF26 => eprintln!("writing 0x{:02X} to unimplemented NR52", value),
            0xFF40 => eprintln!("writing 0x{:02X} to unimplemented LCDC", value),
            0xFF42 => eprintln!("writing 0x{:02X} to unimplemented SCY", value),
            0xFF43 => eprintln!("writing 0x{:02X} to unimplemented SCX", value),
            0xFF44 => eprintln!("writing 0x{:02X} to unimplemented LY", value),
            0xFF47 => eprintln!("writing 0x{:02X} to unimplemented BGP", value),
            0xFF80..=0xFFFE => {
                let ram_offset = address - 0xFF80;
                self.high_ram[usize::from(ram_offset)] = value;
            }
            0xFFFF => self.interrupt_enable = value & 0b0001_1111,
            _ => todo!("write to 0x{:02X}", address),
        }
    }

    pub fn write_word_address(&mut self, value: u16, address: u16) {
        let low = value & 0x00FF;
        let high = value >> 8;
        self.write_byte_address(low as u8, address);
        self.write_byte_address(high as u8, address + 1);
    }
}

impl Mmu {
    const VBLANK_INTERRUPT_MASK: u8 = 0b0000_0001;
    const LCD_STAT_INTERRUPT_MASK: u8 = 0b000_00010;
    const TIMER_INTERRUPT_MASK: u8 = 0b0000_0100;
    const SERIAL_INTERRUPT_MASK: u8 = 0b0000_1000;
    const JOYPAD_INTERRUPT_MASK: u8 = 0b0001_0000;

    pub fn set_interrupt_master_enable(&mut self, set: bool) {
        self.interrupt_master_enable = set;
    }

    // Checks to see if an interrupt can be handled. An interrupt can
    // be handled if:
    //  - The interrupt master enable flag is set.
    //  - The corresponding interrupt enable bit is set.
    //  - The corresponding interrupt flag bit is set.
    //
    // If all 3 of these conditions are met, the function returns
    // the highest-priority interrupt to be handled (the interrupt
    // corresponding to the lowest bit in the interrupt flag) is unset
    // from the interrupt flag, and is returned.
    //
    // The returned interrupt is expected to be handled immedietly.
    pub fn poll_interrupt(&mut self) -> Option<InterruptType> {
        if self.timer.poll_interrupt() {
            self.interrupt_flag |= Self::TIMER_INTERRUPT_MASK;
        }

        if !self.interrupt_master_enable {
            return None;
        }

        for bit_idx in 0..=4 {
            let mask = 1 << bit_idx;
            if ((self.interrupt_enable & mask) != 0) && ((self.interrupt_flag & mask) != 0) {
                self.interrupt_flag &= !mask;
                self.interrupt_master_enable = false;
                return match bit_idx {
                    0 => Some(InterruptType::VBlank),
                    1 => Some(InterruptType::LcdStat),
                    2 => Some(InterruptType::Timer),
                    3 => Some(InterruptType::Serial),
                    4 => Some(InterruptType::Joypad),
                    _ => unreachable!(),
                };
            }
        }

        None
    }

    // Checks to see if an ongoing HALT instruction should finish. This is the
    // case when (IE & IF) != 0, meaning there is a pending interrupt.
    //
    // Note that this pending interrupt does not require IME to be set to end
    // an ongoing HALT. When IME is not set, the pending interrupt is not
    // actually handled, but instead execution resumes with the instruction
    // after the HALT instruction.
    //
    // TODO: There is a HALT bug when the instruction before the halt is an EI.
    pub fn halt_finished(&mut self) -> bool {
        if self.timer.poll_interrupt() {
            self.interrupt_flag |= Self::TIMER_INTERRUPT_MASK;
        }

        (self.interrupt_enable & self.interrupt_flag) != 0
    }
}
