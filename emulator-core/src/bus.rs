use crate::{
    apu::Apu,
    cartridge::Cartridge,
    joypad::Joypad,
    ppu::{Ppu, PpuMode, PpuRenderStatus},
    serial::Serial,
    timer::Timer,
};

const BOOT_ROM: &[u8; 0x900] = include_bytes!("cgb_boot_rom.bin");

#[derive(Clone, Copy, Debug)]
pub enum InterruptType {
    VBlank,
    LcdStat,
    Timer,
    Serial,
    Joypad,
}

#[derive(Clone, Copy, Debug)]
pub enum SpeedMode {
    Normal,
    Double,
}

#[derive(Clone)]
pub struct Bus {
    pub interrupt_enable: u8,
    pub interrupt_flag: u8,
    pub interrupt_master_enable: bool,
    wram_banks: Box<[[u8; 0x1000]; 8]>,
    wram_bank_index: u8,
    high_ram: [u8; 0x7F],
    pub boot_rom_enabled: bool,
    dma_source: u16,
    dma_destination: u16,
    prepare_speed_switch: bool,
    current_speed: SpeedMode,
    hblank_dma_blocks_left: u8,
    hblank_dma_ongoing: bool,
    pub cartridge: Cartridge,
    pub timer: Timer,
    pub serial: Serial,
    pub ppu: Ppu,
    pub joypad: Joypad,
    pub apu: Apu,
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            interrupt_enable: 0,
            interrupt_flag: 0,
            interrupt_master_enable: false,
            wram_banks: Box::new([[0; 0x1000]; 8]),
            wram_bank_index: 1,
            high_ram: [0; 0x7F],
            boot_rom_enabled: true,
            dma_source: 0,
            dma_destination: 0,
            prepare_speed_switch: false,
            current_speed: SpeedMode::Normal,
            hblank_dma_blocks_left: 0,
            hblank_dma_ongoing: false,
            timer: Default::default(),
            serial: Default::default(),
            ppu: Default::default(),
            joypad: Default::default(),
            apu: Default::default(),
            cartridge,
        }
    }
}

impl Bus {
    pub fn step_m_cycle(&mut self) {
        for i in 0..4 {
            let double_speed_tick = matches!(self.current_speed, SpeedMode::Double) && i % 2 == 1;

            let old_ppu_mode = self.ppu.get_stat_mode();

            if !double_speed_tick {
                self.apu.step();
                self.ppu.step();
            }

            self.cartridge.step();
            self.timer.step();

            let new_ppu_mode = self.ppu.get_stat_mode();

            if !matches!(old_ppu_mode, PpuRenderStatus::HBlank)
                && matches!(new_ppu_mode, PpuRenderStatus::HBlank)
                && self.hblank_dma_blocks_left > 0
                && self.hblank_dma_ongoing
            {
                for _ in 0..Self::DMA_BLOCK_SIZE {
                    let data = self.read_byte_address(self.dma_source);
                    self.dma_source += 1;

                    self.write_byte_address(data, self.dma_destination);
                    self.dma_destination += 1;
                }
                self.hblank_dma_blocks_left -= 1;

                if self.hblank_dma_blocks_left == 0 {
                    self.hblank_dma_ongoing = false;
                }
            }

            self.update_interrupt_flag();
        }
    }

    pub fn read_byte_address(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x00FF => {
                if self.boot_rom_enabled {
                    BOOT_ROM[usize::from(address)]
                } else {
                    self.cartridge.read(address)
                }
            }
            0x0100..=0x01FF => self.cartridge.read(address),
            0x0200..=0x08FF => {
                if self.boot_rom_enabled {
                    BOOT_ROM[usize::from(address)]
                } else {
                    self.cartridge.read(address)
                }
            }
            0x900..=0x7FFF => self.cartridge.read(address),
            0x8000..=0x9FFF => self.ppu.read_vram(address - 0x8000),
            0xA000..=0xBFFF => self.cartridge.read(address),
            0xC000..=0xCFFF => self.wram_banks[0][usize::from(address - 0xC000)],
            0xD000..=0xDFFF => {
                self.wram_banks[usize::from(self.wram_bank_index)][usize::from(address - 0xD000)]
            }
            0xE000..=0xFDFF => self.read_byte_address(address - 0x2000), // echo ram
            0xFE00..=0xFE9F => self.ppu.read_object_attribute_memory(address - 0xFE00),
            0xFEA0..=0xFEFF => 0x00, // unusable memory, read returns garbage
            0xFF00 => self.joypad.read(),
            0xFF04 => self.timer.get_divider_register(),
            0xFF05 => self.timer.get_timer_counter(),
            0xFF06 => self.timer.get_timer_modulo(),
            0xFF07 => self.timer.get_timer_control(),
            0xFF0F => self.interrupt_flag,
            0xFF10 => self.apu.read_nr10(),
            0xFF11 => self.apu.read_nr11(),
            0xFF12 => self.apu.read_nr12(),
            0xFF13 => self.apu.read_nr13(),
            0xFF14 => self.apu.read_nr14(),
            0xFF15 => self.apu.read_nr20(),
            0xFF16 => self.apu.read_nr21(),
            0xFF17 => self.apu.read_nr22(),
            0xFF18 => self.apu.read_nr23(),
            0xFF19 => self.apu.read_nr24(),
            0xFF1A => self.apu.read_nr30(),
            0xFF1B => self.apu.read_nr31(),
            0xFF1C => self.apu.read_nr32(),
            0xFF1D => self.apu.read_nr33(),
            0xFF1E => self.apu.read_nr34(),
            0xFF1F => self.apu.read_nr40(),
            0xFF20 => self.apu.read_nr41(),
            0xFF21 => self.apu.read_nr42(),
            0xFF22 => self.apu.read_nr43(),
            0xFF23 => self.apu.read_nr44(),
            0xFF24 => self.apu.read_nr50(),
            0xFF25 => self.apu.read_nr51(),
            0xFF26 => self.apu.read_nr52(),
            0xFF27..=0xFF2F => 0xFF, // $FF27-$FF2F always read back as $FF
            0xFF30..=0xFF3F => self.apu.read_wave_pattern_ram(address - 0xFF30),
            0xFF40 => self.ppu.read_lcd_control(),
            0xFF41 => self.ppu.read_stat(),
            0xFF42 => self.ppu.read_scroll_y(),
            0xFF43 => self.ppu.read_scroll_x(),
            0xFF44 => self.ppu.read_lcd_y(),
            0xFF45 => self.ppu.read_lcd_y_compare(),
            0xFF47 => self.ppu.read_bg_palette(),
            0xFF48 => self.ppu.read_obj_palette_0(),
            0xFF49 => self.ppu.read_obj_palette_1(),
            0xFF4A => self.ppu.read_window_y(),
            0xFF4B => self.ppu.read_window_x(),
            0xFF4D => self.read_key_1(),
            0xFF4F => self.ppu.read_vram_bank(),
            0xFF51 => self.read_dma_source_high(),
            0xFF52 => self.read_dma_source_low(),
            0xFF53 => self.read_dma_destination_high(),
            0xFF54 => self.read_dma_destination_low(),
            0xFF55 => self.read_dma_start(),
            0xFF68 => self.ppu.read_background_palette_index(),
            0xFF69 => self.ppu.read_background_palette_data(),
            0xFF6A => self.ppu.read_obj_palette_index(),
            0xFF6B => self.ppu.read_obj_palette_data(),
            0xFF70 => self.wram_bank_index,
            0xFF80..=0xFFFE => self.high_ram[usize::from(address - 0xFF80)],
            0xFFFF => self.interrupt_enable,
            _ => {
                println!("read from 0x{:02X}", address);
                0
            }
        }
    }

    pub fn write_byte_address(&mut self, value: u8, address: u16) {
        match address {
            0x0000..=0x7FFF => {
                self.cartridge.write(value, address);
            }
            0x8000..=0x9FFF => self.ppu.write_vram(value, address - 0x8000),
            0xA000..=0xBFFF => self.cartridge.write(value, address),
            0xC000..=0xCFFF => self.wram_banks[0][usize::from(address - 0xC000)] = value,
            0xD000..=0xDFFF => {
                self.wram_banks[usize::from(self.wram_bank_index)][usize::from(address - 0xD000)] =
                    value
            }
            0xE000..=0xFDFF => self.write_byte_address(value, address - 0x2000), // echo ram
            0xFE00..=0xFE9F => self
                .ppu
                .write_object_attribute_memory(value, address - 0xFE00),
            0xFEA0..=0xFEFF => {} // unusable memory, write is no-op
            0xFF00 => self.joypad.write(value),
            0xFF01 => self.serial.write_byte(value),
            0xFF02 => {}
            0xFF04 => self.timer.set_divider_register(value),
            0xFF05 => self.timer.set_timer_counter(value),
            0xFF06 => self.timer.set_timer_modulo(value),
            0xFF07 => self.timer.set_timer_control(value),
            0xFF0F => {
                self.interrupt_flag = value & 0b0001_1111;
            }
            0xFF10 => self.apu.write_nr10(value),
            0xFF11 => self.apu.write_nr11(value),
            0xFF12 => self.apu.write_nr12(value),
            0xFF13 => self.apu.write_nr13(value),
            0xFF14 => self.apu.write_nr14(value),
            0xFF15 => self.apu.write_nr20(value),
            0xFF16 => self.apu.write_nr21(value),
            0xFF17 => self.apu.write_nr22(value),
            0xFF18 => self.apu.write_nr23(value),
            0xFF19 => self.apu.write_nr24(value),
            0xFF1A => self.apu.write_nr30(value),
            0xFF1B => self.apu.write_nr31(value),
            0xFF1C => self.apu.write_nr32(value),
            0xFF1D => self.apu.write_nr33(value),
            0xFF1E => self.apu.write_nr34(value),
            0xFF1F => self.apu.write_nr40(value),
            0xFF20 => self.apu.write_nr41(value),
            0xFF21 => self.apu.write_nr42(value),
            0xFF22 => self.apu.write_nr43(value),
            0xFF23 => self.apu.write_nr44(value),
            0xFF24 => self.apu.write_nr50(value),
            0xFF25 => self.apu.write_nr51(value),
            0xFF26 => self.apu.write_nr52(value),
            0xFF27..=0xFF2F => {} // $FF27-$FF2F always read back as $FF
            0xFF30..=0xFF3F => self.apu.write_wave_pattern_ram(value, address - 0xFF30),
            0xFF40 => self.ppu.write_lcd_control(value),
            0xFF41 => self.ppu.write_stat(value),
            0xFF42 => self.ppu.write_scroll_y(value),
            0xFF43 => self.ppu.write_scroll_x(value),
            0xFF45 => self.ppu.write_lcd_y_compare(value),
            0xFF46 => {
                // DMA
                let start_address = u16::from(value) * 0x100;
                for offset in 0..0xA0 {
                    let data = self.read_byte_address(start_address + offset);
                    self.write_byte_address(data, 0xFE00 + offset);
                }
            }
            0xFF47 => self.ppu.write_bg_palette(value),
            0xFF48 => self.ppu.write_obj_palette_0(value),
            0xFF49 => self.ppu.write_obj_palette_1(value),
            0xFF4A => self.ppu.write_window_y(value),
            0xFF4B => self.ppu.write_window_x(value),
            0xFF4C => {
                println!("write of 0x{:02X} to 0xFF4C", value);
                const CGB_MODE_FLAG_MASK: u8 = 1 << 7;
                const DMG_MODE_FLAG_MASK: u8 = 1 << 2;
                const PGB_MODE_FLAG_MASK: u8 = CGB_MODE_FLAG_MASK | DMG_MODE_FLAG_MASK;

                let ppu_mode = match value & PGB_MODE_FLAG_MASK {
                    PGB_MODE_FLAG_MASK => PpuMode::Pgb,
                    CGB_MODE_FLAG_MASK => PpuMode::Cgb,
                    DMG_MODE_FLAG_MASK => PpuMode::Dmg,
                    _ => unreachable!("0b{:08b}", value),
                };

                self.ppu.set_ppu_mode(ppu_mode);
            }
            0xFF4D => self.write_key_1(value),
            0xFF4F => self.ppu.write_vram_bank(value),
            0xFF50 => {
                println!("boot rom disabled");
                self.boot_rom_enabled = value == 0 // disable boot rom on non-zero write
            }
            0xFF51 => self.write_dma_source_high(value),
            0xFF52 => self.write_dma_source_low(value),
            0xFF53 => self.write_dma_destination_high(value),
            0xFF54 => self.write_dma_destination_low(value),
            0xFF55 => self.write_dma_start(value),
            0xFF68 => self.ppu.write_background_palette_index(value),
            0xFF69 => self.ppu.write_background_palette_data(value),
            0xFF6A => self.ppu.write_obj_palette_index(value),
            0xFF6B => self.ppu.write_obj_palette_data(value),
            0xFF70 => {
                self.wram_bank_index = value & 0b111;
                if self.wram_bank_index == 0 {
                    self.wram_bank_index = 1;
                }
            }
            0xFF80..=0xFFFE => {
                self.high_ram[usize::from(address - 0xFF80)] = value;
            }
            0xFFFF => self.interrupt_enable = value & 0b0001_1111,
            _ => eprintln!("write of 0x{:02X} to 0x{:02X}", value, address),
        }
    }

    fn read_dma_source_high(&self) -> u8 {
        (self.dma_source >> 8) as u8
    }

    fn write_dma_source_high(&mut self, value: u8) {
        self.dma_source &= !0xFF00;
        self.dma_source |= u16::from(value) << 8;
    }

    fn read_dma_source_low(&self) -> u8 {
        self.dma_source as u8
    }

    fn write_dma_source_low(&mut self, value: u8) {
        self.dma_source &= !0x00FF;
        self.dma_source |= u16::from(value & 0xF0);
    }

    fn read_dma_destination_high(&self) -> u8 {
        (self.dma_destination >> 8) as u8
    }

    fn write_dma_destination_high(&mut self, value: u8) {
        self.dma_destination &= !0xFF00;
        self.dma_destination |= u16::from((value & 0b0001_1111) | 0b1000_0000) << 8;
    }

    fn read_dma_destination_low(&self) -> u8 {
        self.dma_destination as u8
    }

    fn write_dma_destination_low(&mut self, value: u8) {
        self.dma_destination &= !0x00FF;
        self.dma_destination |= u16::from(value & 0xF0);
    }

    fn read_dma_start(&self) -> u8 {
        if self.hblank_dma_ongoing {
            self.hblank_dma_blocks_left.wrapping_sub(1)
        } else {
            self.hblank_dma_blocks_left.wrapping_sub(1) | 0b1000_0000
        }
    }

    const DMA_BLOCK_SIZE: u16 = 0x10;
    fn write_dma_start(&mut self, value: u8) {
        const HBLANK_DMA_MASK: u8 = 0b1000_0000;
        const DMA_LENGTH_MASK: u8 = 0b0111_1111;

        let transfer_blocks = (value & DMA_LENGTH_MASK) + 1;

        if (value & HBLANK_DMA_MASK) == HBLANK_DMA_MASK {
            self.hblank_dma_blocks_left = transfer_blocks;
            self.hblank_dma_ongoing = true;
        } else if self.hblank_dma_blocks_left > 0 {
            self.hblank_dma_ongoing = false;
        } else {
            for _ in 0..transfer_blocks {
                for _ in 0..Self::DMA_BLOCK_SIZE {
                    let data = self.read_byte_address(self.dma_source);
                    self.dma_source += 1;

                    self.write_byte_address(data, self.dma_destination);
                    self.dma_destination += 1;
                }
            }

            self.hblank_dma_blocks_left = 0;
            self.hblank_dma_ongoing = false;
        }
    }

    const KEY_1_PREPARE_SPEED_SWITCH_MASK: u8 = 1 << 0;
    const KEY_1_CURRENT_SPEED_MASK: u8 = 1 << 7;

    fn read_key_1(&self) -> u8 {
        let mut result = 0;

        if self.prepare_speed_switch {
            result |= Self::KEY_1_PREPARE_SPEED_SWITCH_MASK;
        }

        if matches!(self.current_speed, SpeedMode::Double) {
            result |= Self::KEY_1_CURRENT_SPEED_MASK;
        }

        result
    }

    fn write_key_1(&mut self, value: u8) {
        self.prepare_speed_switch = (value & Self::KEY_1_PREPARE_SPEED_SWITCH_MASK)
            == Self::KEY_1_PREPARE_SPEED_SWITCH_MASK;
    }
}

impl Bus {
    const VBLANK_INTERRUPT_MASK: u8 = 0b0000_0001;
    const LCD_STAT_INTERRUPT_MASK: u8 = 0b0000_0010;
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
        self.update_interrupt_flag();

        if !self.interrupt_master_enable {
            return None;
        }

        for bit_idx in 0..=4 {
            let mask = 1 << bit_idx;
            if ((self.interrupt_enable & mask) != 0) && ((self.interrupt_flag & mask) != 0) {
                self.interrupt_flag &= !mask;
                self.interrupt_master_enable = false;
                let result = match bit_idx {
                    0 => Some(InterruptType::VBlank),
                    1 => Some(InterruptType::LcdStat),
                    2 => Some(InterruptType::Timer),
                    3 => Some(InterruptType::Serial),
                    4 => Some(InterruptType::Joypad),
                    _ => unreachable!(),
                };
                return result;
            }
        }

        None
    }

    fn update_interrupt_flag(&mut self) {
        if self.timer.poll_interrupt() {
            self.interrupt_flag |= Self::TIMER_INTERRUPT_MASK;
        }

        if self.ppu.poll_vblank_interrupt() {
            self.interrupt_flag |= Self::VBLANK_INTERRUPT_MASK;
        }

        if self.ppu.poll_stat_interrupt() {
            self.interrupt_flag |= Self::LCD_STAT_INTERRUPT_MASK;
        }

        if self.joypad.poll_interrupt() {
            self.interrupt_flag |= Self::JOYPAD_INTERRUPT_MASK;
        }
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

    // Attempts to handle an executed stop instruction. If there is a pending speed
    // switch, that will consume the stop instruction and a speed switch will occur.
    // Otherwise, the stop instruction should be handled as usual.
    //
    // Returns whether the stop instruction was handled by the bus, otherwise the cpu
    // shoud handle it as usual.
    pub fn maybe_handle_stop(&mut self) -> bool {
        if self.prepare_speed_switch {
            self.current_speed = match self.current_speed {
                SpeedMode::Normal => SpeedMode::Double,
                SpeedMode::Double => SpeedMode::Normal,
            };
            println!("switched speed to {:?}", self.current_speed);
            self.prepare_speed_switch = false;

            true
        } else {
            false
        }
    }

    pub fn get_current_speed(&self) -> SpeedMode {
        self.current_speed
    }
}
