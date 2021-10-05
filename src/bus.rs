use crate::{
    cartridge::{self, Cartridge},
    ppu::Ppu,
    serial::Serial,
    timer::Timer,
};

#[derive(Clone, Copy, Debug)]
pub enum InterruptType {
    VBlank,
    LcdStat,
    Timer,
    Serial,
    Joypad,
}

pub struct Bus {
    interrupt_enable: u8,
    interrupt_flag: u8,
    interrupt_master_enable: bool,
    low_ram: [u8; 0x2000],
    high_ram: [u8; 0x7F],
    cartridge_ram: [u8; 0x2000],
    cartridge: Cartridge,
    timer: Timer,
    pub serial: Serial,
    pub ppu: Ppu,
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        let mut result = Self {
            interrupt_enable: Default::default(),
            interrupt_flag: Default::default(),
            interrupt_master_enable: Default::default(),
            low_ram: [0; 0x2000],
            high_ram: [0; 0x7F],
            cartridge_ram: [0; 0x2000],
            timer: Default::default(),
            serial: Default::default(),
            ppu: Default::default(),
            cartridge,
        };

        result.write_byte_address(0x00, 0xFF05);
        result.write_byte_address(0x00, 0xFF06);
        result.write_byte_address(0x00, 0xFF07);
        result.write_byte_address(0x80, 0xFF10);
        result.write_byte_address(0xBF, 0xFF11);
        result.write_byte_address(0xF3, 0xFF12);
        result.write_byte_address(0xBF, 0xFF14);
        result.write_byte_address(0x3F, 0xFF16);
        result.write_byte_address(0x00, 0xFF17);
        result.write_byte_address(0xBF, 0xFF19);
        result.write_byte_address(0x7F, 0xFF1A);
        result.write_byte_address(0xFF, 0xFF1B);
        result.write_byte_address(0x9F, 0xFF1C);
        result.write_byte_address(0xBF, 0xFF1E);
        result.write_byte_address(0xFF, 0xFF20);
        result.write_byte_address(0x00, 0xFF21);
        result.write_byte_address(0x00, 0xFF22);
        result.write_byte_address(0xBF, 0xFF23);
        result.write_byte_address(0x77, 0xFF24);
        result.write_byte_address(0xF3, 0xFF25);
        result.write_byte_address(0xF1, 0xFF26);
        result.write_byte_address(0x91, 0xFF40);
        result.write_byte_address(0x00, 0xFF42);
        result.write_byte_address(0x00, 0xFF43);
        result.write_byte_address(0x00, 0xFF45);
        result.write_byte_address(0xFC, 0xFF47);
        result.write_byte_address(0xFF, 0xFF48);
        result.write_byte_address(0xFF, 0xFF49);
        result.write_byte_address(0x00, 0xFF4A);
        result.write_byte_address(0x00, 0xFF4B);
        result.write_byte_address(0x00, 0xFFFF);

        result
    }
}

impl Bus {
    pub fn step(&mut self) {
        if self.timer.poll_interrupt() {
            self.interrupt_flag |= Self::TIMER_INTERRUPT_MASK;
        }

        if self.ppu.poll_vblank_interrupt() {
            self.interrupt_flag |= Self::VBLANK_INTERRUPT_MASK;
        }

        if self.ppu.poll_stat_interrupt() {
            self.interrupt_flag |= Self::LCD_STAT_INTERRUPT_MASK;
        }

        self.timer.step();
        self.ppu.step();
    }

    pub fn read_byte_address(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => self.cartridge.read_rom(address),
            0x8000..=0x97FF => self.ppu.read_character_ram(address - 0x8000),
            0x9800..=0x9BFF => self.ppu.read_bg_map_data_1(address - 0x9800),
            0x9C00..=0x9FFF => self.ppu.read_bg_map_data_2(address - 0x9C00),
            0xA000..=0xBFFF => self.cartridge_ram[usize::from(address - 0xA000)],
            0xC000..=0xDFFF => self.low_ram[usize::from(address - 0xC000)],
            0xE000..=0xFDFF => self.read_byte_address(address - 0x2000), // echo ram
            0xFE00..=0xFE9F => self.ppu.read_object_attribute_memory(address - 0xFE00),
            0xFEA0..=0xFEFF => 0x00, // unusable memory, read returns garbage
            0xFF00 => {
                eprintln!("reading from unimplemented P1");
                0xFF
            }
            0xFF0F => self.interrupt_flag,
            0xFF40 => self.ppu.read_lcd_control(),
            0xFF41 => self.ppu.read_stat(),
            0xFF42 => self.ppu.read_scroll_y(),
            0xFF43 => self.ppu.read_scroll_x(),
            0xFF44 => self.ppu.read_lcd_y(),
            0xFF47 => self.ppu.read_bg_palette(),
            0xFF48 => self.ppu.read_obj_palette_1(),
            0xFF49 => self.ppu.read_obj_palette_2(),
            0xFF4A => self.ppu.read_window_y(),
            0xFF4B => self.ppu.read_window_x(),
            0xFF4D => {
                eprintln!("reading from unimplemented KEY1");
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
        u16::from_le_bytes([low, high])
    }

    pub fn write_byte_address(&mut self, value: u8, address: u16) {
        match address {
            0x0000..=0x7FFF => self.cartridge.write_rom(value, address),
            0x8000..=0x97FF => {
                self.ppu.write_character_ram(value, address - 0x8000);
            }
            0x9800..=0x9BFF => self.ppu.write_bg_map_data_1(value, address - 0x9800),
            0x9C00..=0x9FFF => self.ppu.write_bg_map_data_2(value, address - 0x9C00),
            0xA000..=0xBFFF => self.cartridge_ram[usize::from(address - 0xA000)] = value,
            0xC000..=0xDFFF => self.low_ram[usize::from(address - 0xC000)] = value,
            0xE000..=0xFDFF => self.write_byte_address(value, address - 0x2000), // echo ram
            0xFE00..=0xFE9F => self
                .ppu
                .write_object_attribute_memory(value, address - 0xFE00),
            0xFEA0..=0xFEFF => {} // unusable memory, write is no-op
            0xFF00 => eprintln!("writing 0x{:02X} to unimplemented P1", value),
            0xFF01 => self.serial.write_byte(value),
            0xFF02 => eprintln!("writing 0x{:02X} to unimplemented SC", value),
            0xFF04 => self.timer.set_divider_register(value),
            0xFF05 => self.timer.set_timer_counter(value),
            0xFF06 => self.timer.set_timer_modulo(value),
            0xFF07 => self.timer.set_timer_control(value),
            0xFF0F => self.interrupt_flag = value & 0b0001_1111,
            0xFF10 => eprintln!("writing 0x{:02X} to unimplemented NR10", value),
            0xFF11 => eprintln!("writing 0x{:02X} to unimplemented NR11", value),
            0xFF12 => eprintln!("writing 0x{:02X} to unimplemented NR12", value),
            0xFF13 => eprintln!("writing 0x{:02X} to unimplemented NR13", value),
            0xFF14 => eprintln!("writing 0x{:02X} to unimplemented NR14", value),
            0xFF16 => eprintln!("writing 0x{:02X} to unimplemented NR21", value),
            0xFF17 => eprintln!("writing 0x{:02X} to unimplemented NR22", value),
            0xFF18 => eprintln!("writing 0x{:02X} to unimplemented NR23", value),
            0xFF19 => eprintln!("writing 0x{:02X} to unimplemented NR24", value),
            0xFF1A => eprintln!("writing 0x{:02X} to unimplemented NR30", value),
            0xFF1B => eprintln!("writing 0x{:02X} to unimplemented NR31", value),
            0xFF1C => eprintln!("writing 0x{:02X} to unimplemented NR32", value),
            0xFF1D => eprintln!("writing 0x{:02X} to unimplemented NR33", value),
            0xFF1E => eprintln!("writing 0x{:02X} to unimplemented NR34", value),
            0xFF20 => eprintln!("writing 0x{:02X} to unimplemented NR41", value),
            0xFF21 => eprintln!("writing 0x{:02X} to unimplemented NR42", value),
            0xFF22 => eprintln!("writing 0x{:02X} to unimplemented NR43", value),
            0xFF23 => eprintln!("writing 0x{:02X} to unimplemented NR44", value),
            0xFF24 => eprintln!("writing 0x{:02X} to unimplemented NR50", value),
            0xFF25 => eprintln!("writing 0x{:02X} to unimplemented NR51", value),
            0xFF26 => eprintln!("writing 0x{:02X} to unimplemented NR52", value),
            0xFF30..=0xFF3F => eprintln!(
                "writing 0x{:02X} to WAVE_PATTERN_RAM[{:02X}]",
                value,
                address - 0xFF30
            ),
            0xFF40 => self.ppu.write_lcd_control(value),
            0xFF41 => self.ppu.write_stat(value),
            0xFF42 => self.ppu.write_scroll_y(value),
            0xFF43 => self.ppu.write_scroll_x(value),
            0xFF46 => {
                // DMA
                let start_address = u16::from(value) * 0x100;
                for offset in 0..0xA0 {
                    let data = self.read_byte_address(start_address + offset);
                    self.ppu.write_object_attribute_memory(data, offset);
                }
            }
            0xFF47 => self.ppu.write_bg_palette(value),
            0xFF48 => self.ppu.write_obj_palette_1(value),
            0xFF49 => self.ppu.write_obj_palette_2(value),
            0xFF4A => self.ppu.write_window_y(value),
            0xFF4B => self.ppu.write_window_x(value),
            0xFF4D => eprintln!("writing 0x{:02X} to unimplemented KEY1", value),
            0xFF80..=0xFFFE => {
                let ram_offset = address - 0xFF80;
                self.high_ram[usize::from(ram_offset)] = value;
            }
            0xFFFF => self.interrupt_enable = value & 0b0001_1111,
            _ => eprintln!("unknown write of 0x{:02X} to 0x{:02X}", value, address),
        }
    }

    pub fn write_word_address(&mut self, value: u16, address: u16) {
        let bytes = value.to_le_bytes();
        self.write_byte_address(bytes[0], address);
        self.write_byte_address(bytes[1], address + 1);
    }
}

impl Bus {
    const VBLANK_INTERRUPT_MASK: u8 = 0b0000_0001;
    const LCD_STAT_INTERRUPT_MASK: u8 = 0b010_0010;
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
        (self.interrupt_enable & self.interrupt_flag) != 0
    }
}
