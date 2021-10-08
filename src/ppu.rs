use std::convert::TryFrom;
use std::default::Default;

#[derive(Clone, Copy, Debug)]
enum PpuMode {
    HBlank,
    VBlank,
    OAMSearch,
    PixelTransfer,
}

#[derive(Clone, Copy, Debug)]
enum StatInterruptSource {
    LycEqualsLy,
    OAMSearch,
    HBlank,
    VBlank,
}

#[derive(Clone, Copy, Debug)]
pub enum PaletteColor {
    White,
    LightGray,
    DarkGray,
    Black,
}

#[derive(Clone, Copy, Default)]
struct SpriteAttributeInfo {
    pub y_position: u8,
    pub x_position: u8,
    pub tile_index: u8,
    pub flags: u8,
}

impl SpriteAttributeInfo {
    fn get_bg_window_over_obj(&self) -> bool {
        const BG_WINDOW_OVER_OBJ_MASK: u8 = 1 << 7;
        (self.flags & BG_WINDOW_OVER_OBJ_MASK) != 0
    }

    fn get_y_flip(&self) -> bool {
        const Y_FLIP_MASK: u8 = 1 << 6;
        (self.flags & Y_FLIP_MASK) != 0
    }

    fn get_x_flip(&self) -> bool {
        const X_FLIP_MASK: u8 = 1 << 5;
        (self.flags & X_FLIP_MASK) != 0
    }

    fn use_low_palette(&self) -> bool {
        const LOW_PALETTE_MASK: u8 = 1 << 4;
        (self.flags & LOW_PALETTE_MASK) != 0
    }
}

#[derive(Clone)]
pub struct Ppu {
    character_ram: [u8; 0x1800],
    bg_map_data_1: [u8; 0x400],
    bg_map_data_2: [u8; 0x400],
    object_attributes: [SpriteAttributeInfo; 40],
    vblank_interrupt_waiting: bool,
    stat_interrupt_waiting: bool,
    dot: u16,
    lcd_y: u8,
    lcd_y_compare: u8,
    stat: u8,
    lcd_control: u8,
    scroll_x: u8,
    scroll_y: u8,
    window_x: u8,
    window_y: u8,
    buffer: [[PaletteColor; 160]; 144], // access as buffer[y][x]
    bg_palette: [PaletteColor; 4],
    obj_palette_1: [PaletteColor; 4],
    obj_palette_2: [PaletteColor; 4],
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            character_ram: [0; 0x1800],
            bg_map_data_1: [0; 0x400],
            bg_map_data_2: [0; 0x400],
            object_attributes: [Default::default(); 40],
            vblank_interrupt_waiting: Default::default(),
            stat_interrupt_waiting: Default::default(),
            dot: Default::default(),
            lcd_y: Default::default(),
            lcd_y_compare: Default::default(),
            stat: Default::default(),
            lcd_control: Default::default(),
            scroll_x: Default::default(),
            scroll_y: Default::default(),
            window_x: Default::default(),
            window_y: Default::default(),
            buffer: [[PaletteColor::White; 160]; 144],
            bg_palette: [PaletteColor::White; 4],
            obj_palette_1: [PaletteColor::White; 4],
            obj_palette_2: [PaletteColor::White; 4],
        }
    }
}

impl Ppu {
    pub fn step(&mut self) {
        if self.lcd_y == self.lcd_y_compare {
            self.set_stat_lyc_equals_ly(true);
        } else {
            self.set_stat_lyc_equals_ly(false);
        }

        if self.lcd_y < 144 {
            if self.dot == 0 {
                self.set_stat_mode(PpuMode::OAMSearch);
            } else if self.dot == 80 {
                self.set_stat_mode(PpuMode::PixelTransfer);
            } else if self.dot == 252 {
                self.set_stat_mode(PpuMode::HBlank);
            }
        } else if self.lcd_y == 144 {
            if self.dot == 0 {
                self.set_stat_mode(PpuMode::VBlank);
                self.vblank_interrupt_waiting = true;
            }
        }

        if matches!(self.get_stat_mode(), PpuMode::PixelTransfer) {
            let buffer_x = u8::try_from(self.dot - 80).unwrap();
            let buffer_y = self.lcd_y;

            if buffer_x < 160 {
                let mut non_zero_bg_window_pixel_drawn = false;

                if self.get_bg_window_enable() {
                    let bg_render_x = u16::from(buffer_x.wrapping_add(self.scroll_x));
                    let bg_render_y = u16::from(buffer_y.wrapping_add(self.scroll_y));

                    let bg_tile_x = bg_render_x / 8;
                    let bg_tile_y = bg_render_y / 8;
                    let bg_tile_idx = bg_tile_x + (bg_tile_y * 32);

                    let bg_tile_id = self.get_bg_tile_map(bg_tile_idx);
                    let bg_tile_data = self.get_bg_window_tile_data(bg_tile_id);

                    let bg_tile_row = bg_render_y % 8;
                    let bg_lsb_row_color = bg_tile_data[usize::from(bg_tile_row) * 2];
                    let bg_msb_row_color = bg_tile_data[(usize::from(bg_tile_row) * 2) + 1];

                    let bg_tile_col = bg_render_x % 8;
                    let bg_lsb_pixel_color = (bg_lsb_row_color & (1 << (7 - bg_tile_col))) != 0;
                    let bg_msb_pixel_color = (bg_msb_row_color & (1 << (7 - bg_tile_col))) != 0;
                    let bg_pixel_palette_idx =
                        (usize::from(bg_msb_pixel_color) << 1) | usize::from(bg_lsb_pixel_color);

                    let bg_pixel_color = self.bg_palette[bg_pixel_palette_idx];

                    self.buffer[usize::from(buffer_y)][usize::from(buffer_x)] = bg_pixel_color;

                    non_zero_bg_window_pixel_drawn |= bg_pixel_palette_idx != 0;

                    if self.get_window_enable()
                        && self.window_y <= buffer_y
                        && self.window_x <= buffer_x + 7
                    {
                        let window_render_x = u16::from(buffer_x + 7 - self.window_x);
                        let window_render_y = u16::from(buffer_y - self.window_y);

                        let window_tile_x = window_render_x / 8;
                        let window_tile_y = window_render_y / 8;
                        let window_tile_idx = window_tile_x + (window_tile_y * 32);

                        let window_tile_id = self.get_window_tile_map(window_tile_idx);
                        let window_tile_data = self.get_bg_window_tile_data(window_tile_id);

                        let window_tile_row = window_render_y % 8;
                        let window_lsb_row_color =
                            window_tile_data[usize::from(window_tile_row) * 2];
                        let window_msb_row_color =
                            window_tile_data[(usize::from(window_tile_row) * 2) + 1];

                        let window_tile_col = window_render_x % 8;
                        let window_lsb_pixel_color =
                            (window_lsb_row_color & (1 << (7 - window_tile_col))) != 0;
                        let window_msb_pixel_color =
                            (window_msb_row_color & (1 << (7 - window_tile_col))) != 0;
                        let window_pixel_palette_idx = (usize::from(window_msb_pixel_color) << 1)
                            | usize::from(window_lsb_pixel_color);

                        let window_pixel_color = self.bg_palette[window_pixel_palette_idx];

                        self.buffer[usize::from(buffer_y)][usize::from(buffer_x)] =
                            window_pixel_color;

                        non_zero_bg_window_pixel_drawn |= window_pixel_palette_idx != 0;
                    }
                } else {
                    self.buffer[usize::from(buffer_y)][usize::from(buffer_x)] = self.bg_palette[0];
                }

                if self.get_obj_enable() {
                    for attribute_info in self.object_attributes {
                        if attribute_info.get_bg_window_over_obj() && non_zero_bg_window_pixel_drawn
                        {
                            continue;
                        }

                        if buffer_y + 16 >= attribute_info.y_position
                            && buffer_y + 8 < attribute_info.y_position
                            && buffer_x + 8 >= attribute_info.x_position
                            && buffer_x < attribute_info.x_position
                        {
                            let sprite_y_offset = if attribute_info.get_y_flip() {
                                7 - (buffer_y + 16 - attribute_info.y_position)
                            } else {
                                buffer_y + 16 - attribute_info.y_position
                            };

                            let sprite_x_offset = if attribute_info.get_x_flip() {
                                7 - (buffer_x + 8 - attribute_info.x_position)
                            } else {
                                buffer_x + 8 - attribute_info.x_position
                            };

                            let sprite_data = match self.get_obj_size() {
                                ObjSize::EightByEight => {
                                    self.get_obj_tile_data(attribute_info.tile_index)
                                }
                                ObjSize::EightBySixteen => {
                                    self.get_obj_tile_data(attribute_info.tile_index & (!0x01))
                                }
                            };
                            let lsb_row_color = sprite_data[usize::from(sprite_y_offset) * 2];
                            let msb_row_color = sprite_data[(usize::from(sprite_y_offset) * 2) + 1];

                            let lsb_pixel_color =
                                (lsb_row_color & (1 << (7 - sprite_x_offset))) != 0;
                            let msb_pixel_color =
                                (msb_row_color & (1 << (7 - sprite_x_offset))) != 0;

                            let pixel_palette_idx =
                                (usize::from(msb_pixel_color) << 1) | usize::from(lsb_pixel_color);

                            if pixel_palette_idx != 0 {
                                let pixel_color = if attribute_info.use_low_palette() {
                                    self.obj_palette_2[pixel_palette_idx]
                                } else {
                                    self.obj_palette_1[pixel_palette_idx]
                                };

                                self.buffer[usize::from(buffer_y)][usize::from(buffer_x)] =
                                    pixel_color;

                                break;
                            }
                        } else if matches!(self.get_obj_size(), ObjSize::EightBySixteen)
                            && buffer_y + 8 >= attribute_info.y_position
                            && buffer_y < attribute_info.y_position
                            && buffer_x + 8 >= attribute_info.x_position
                            && buffer_x < attribute_info.x_position
                        {
                            let sprite_y_offset = if attribute_info.get_y_flip() {
                                7 - (buffer_y + 8 - attribute_info.y_position)
                            } else {
                                buffer_y + 8 - attribute_info.y_position
                            };

                            let sprite_x_offset = if attribute_info.get_x_flip() {
                                7 - (buffer_x + 8 - attribute_info.x_position)
                            } else {
                                buffer_x + 8 - attribute_info.x_position
                            };

                            let sprite_data =
                                self.get_obj_tile_data(attribute_info.tile_index | 0x01);
                            let lsb_row_color = sprite_data[usize::from(sprite_y_offset) * 2];
                            let msb_row_color = sprite_data[(usize::from(sprite_y_offset) * 2) + 1];

                            let lsb_pixel_color =
                                (lsb_row_color & (1 << (7 - sprite_x_offset))) != 0;
                            let msb_pixel_color =
                                (msb_row_color & (1 << (7 - sprite_x_offset))) != 0;

                            let pixel_palette_idx =
                                (usize::from(msb_pixel_color) << 1) | usize::from(lsb_pixel_color);

                            if pixel_palette_idx != 0 {
                                let pixel_color = if attribute_info.use_low_palette() {
                                    self.obj_palette_2[pixel_palette_idx]
                                } else {
                                    self.obj_palette_1[pixel_palette_idx]
                                };

                                self.buffer[usize::from(buffer_y)][usize::from(buffer_x)] =
                                    pixel_color;

                                break;
                            }
                        }
                    }
                }
            }
        }

        self.dot += 1;
        if self.dot > 455 {
            self.dot = 0;
            self.lcd_y += 1;

            if self.lcd_y > 153 {
                self.lcd_y = 0;
            }
        }
    }

    pub fn should_print(&self) -> bool {
        self.lcd_y == 0 && self.dot == 0
    }

    pub fn get_buffer(&self) -> &[[PaletteColor; 160]; 144] {
        &self.buffer
    }

    pub fn poll_vblank_interrupt(&mut self) -> bool {
        if self.vblank_interrupt_waiting {
            self.vblank_interrupt_waiting = false;
            true
        } else {
            false
        }
    }

    pub fn poll_stat_interrupt(&mut self) -> bool {
        if self.stat_interrupt_waiting {
            self.stat_interrupt_waiting = false;
            true
        } else {
            false
        }
    }
}

impl Ppu {
    pub fn read_stat(&self) -> u8 {
        self.stat
    }

    pub fn write_stat(&mut self, data: u8) {
        const STAT_WRITE_MASK: u8 = 0b0111_1000;

        let old_interrupt_line = self.get_stat_interrupt_line();

        self.stat = (data & STAT_WRITE_MASK) | (self.stat & (!STAT_WRITE_MASK));

        let new_interrupt_line = self.get_stat_interrupt_line();

        if !old_interrupt_line && new_interrupt_line {
            self.stat_interrupt_waiting = true;
        }
    }

    fn stat_interrupt_source_enabled(&self, source_type: StatInterruptSource) -> bool {
        const LYC_EQUAL_LY_INTERRUPT_SOURCE_MASK: u8 = 0b0100_0000;
        const OAM_INTERRUPT_SOURCE_MASK: u8 = 0b0010_0000;
        const VBLANK_INTERRUPT_SOURCE_MASK: u8 = 0b0001_0000;
        const HBLANK_INTERRUPT_SOURCE_MASK: u8 = 0b0000_1000;

        match source_type {
            StatInterruptSource::LycEqualsLy => {
                (self.stat & LYC_EQUAL_LY_INTERRUPT_SOURCE_MASK) != 0
            }
            StatInterruptSource::OAMSearch => (self.stat & OAM_INTERRUPT_SOURCE_MASK) != 0,
            StatInterruptSource::HBlank => (self.stat & HBLANK_INTERRUPT_SOURCE_MASK) != 0,
            StatInterruptSource::VBlank => (self.stat & VBLANK_INTERRUPT_SOURCE_MASK) != 0,
        }
    }

    fn get_stat_interrupt_line(&self) -> bool {
        let ppu_mode = self.get_stat_mode();
        let lyc_equals_ly_interrupt_line = self
            .stat_interrupt_source_enabled(StatInterruptSource::LycEqualsLy)
            && self.get_stat_lyc_equals_ly();
        let mode_interrupt_line = match ppu_mode {
            PpuMode::HBlank => self.stat_interrupt_source_enabled(StatInterruptSource::HBlank),
            PpuMode::OAMSearch => {
                self.stat_interrupt_source_enabled(StatInterruptSource::OAMSearch)
            }
            PpuMode::PixelTransfer => false,
            PpuMode::VBlank => self.stat_interrupt_source_enabled(StatInterruptSource::VBlank),
        };

        lyc_equals_ly_interrupt_line || mode_interrupt_line
    }

    const STAT_MODE_MASK: u8 = 0b0000_0011;
    const STAT_HBLANK_MODE_MASK: u8 = 0b0000_0000;
    const STAT_VBLANK_MODE_MASK: u8 = 0b0000_0001;
    const STAT_OAM_SEARCH_MODE_MASK: u8 = 0b0000_0010;
    const STAT_PIXEL_TRANSFER_MODE_MASK: u8 = 0b0000_0011;

    fn set_stat_mode(&mut self, mode: PpuMode) {
        let old_interrupt_line = self.get_stat_interrupt_line();
        let line_high = match mode {
            PpuMode::HBlank => self.stat_interrupt_source_enabled(StatInterruptSource::HBlank),
            PpuMode::OAMSearch => {
                self.stat_interrupt_source_enabled(StatInterruptSource::OAMSearch)
            }
            PpuMode::PixelTransfer => false,
            PpuMode::VBlank => self.stat_interrupt_source_enabled(StatInterruptSource::VBlank),
        };

        if !old_interrupt_line && line_high {
            self.stat_interrupt_waiting = true
        }

        self.stat &= !Self::STAT_MODE_MASK;
        self.stat |= match mode {
            PpuMode::HBlank => Self::STAT_HBLANK_MODE_MASK,
            PpuMode::VBlank => Self::STAT_VBLANK_MODE_MASK,
            PpuMode::OAMSearch => Self::STAT_OAM_SEARCH_MODE_MASK,
            PpuMode::PixelTransfer => Self::STAT_PIXEL_TRANSFER_MODE_MASK,
        };
    }

    fn get_stat_mode(&self) -> PpuMode {
        match self.stat & Self::STAT_MODE_MASK {
            Self::STAT_HBLANK_MODE_MASK => PpuMode::HBlank,
            Self::STAT_VBLANK_MODE_MASK => PpuMode::VBlank,
            Self::STAT_OAM_SEARCH_MODE_MASK => PpuMode::OAMSearch,
            Self::STAT_PIXEL_TRANSFER_MODE_MASK => PpuMode::PixelTransfer,
            _ => unreachable!(),
        }
    }

    const STAT_LYC_EQUAL_LY_MASK: u8 = 0b0000_0100;

    fn set_stat_lyc_equals_ly(&mut self, equals: bool) {
        if equals {
            let old_interrupt_line = self.get_stat_interrupt_line();
            let line_high = self.stat_interrupt_source_enabled(StatInterruptSource::LycEqualsLy);
            if !old_interrupt_line && line_high {
                self.stat_interrupt_waiting = true;
            }

            self.stat |= Self::STAT_LYC_EQUAL_LY_MASK;
        } else {
            self.stat &= !Self::STAT_LYC_EQUAL_LY_MASK;
        }
    }

    fn get_stat_lyc_equals_ly(&self) -> bool {
        (self.stat & Self::STAT_LYC_EQUAL_LY_MASK) != 0
    }
}

#[derive(Clone, Copy, Debug)]
enum ObjSize {
    EightByEight,
    EightBySixteen,
}

impl Ppu {
    pub fn read_lcd_control(&self) -> u8 {
        self.lcd_control
    }

    pub fn write_lcd_control(&mut self, data: u8) {
        self.lcd_control = data;
    }

    fn get_lcd_ppu_enable(&self) -> bool {
        const LCD_PPU_ENABLE_MASK: u8 = 1 << 7;
        (self.lcd_control & LCD_PPU_ENABLE_MASK) != 0
    }

    fn get_window_tile_map(&self, index: u16) -> u8 {
        const WINDOW_TILE_MAP_AREA_MASK: u8 = 1 << 6;
        if (self.lcd_control & WINDOW_TILE_MAP_AREA_MASK) == 0 {
            self.bg_map_data_1[usize::from(index)]
        } else {
            self.bg_map_data_2[usize::from(index)]
        }
    }

    fn get_window_enable(&self) -> bool {
        const WINDOW_ENABLE_MASK: u8 = 1 << 5;
        (self.lcd_control & WINDOW_ENABLE_MASK) != 0
    }

    fn get_bg_window_tile_data(&self, tile_id: u8) -> &[u8] {
        const BG_WINDOW_TILE_DATA_AREA_MASK: u8 = 1 << 4;
        // When LCDC.4 == 0 and tile_id < 128, we start indexing at an offset of
        // 0x1000. In all other situations, start indexing at 0x0000.
        if (self.lcd_control & BG_WINDOW_TILE_DATA_AREA_MASK) == 0 && tile_id < 128 {
            &self.character_ram[0x1000..][usize::from(tile_id) * 16..][..16]
        } else {
            &self.character_ram[usize::from(tile_id) * 16..][..16]
        }
    }

    fn get_bg_tile_map(&self, index: u16) -> u8 {
        const BG_TILE_MAP_AREA_MASK: u8 = 1 << 3;
        if (self.lcd_control & BG_TILE_MAP_AREA_MASK) == 0 {
            self.bg_map_data_1[usize::from(index)]
        } else {
            self.bg_map_data_2[usize::from(index)]
        }
    }

    fn get_obj_tile_data(&self, tile_id: u8) -> &[u8] {
        &self.character_ram[usize::from(tile_id) * 16..][..16]
    }

    fn get_obj_size(&self) -> ObjSize {
        const OBJ_SIZE_MASK: u8 = 1 << 2;
        if (self.lcd_control & OBJ_SIZE_MASK) == 0 {
            ObjSize::EightByEight
        } else {
            ObjSize::EightBySixteen
        }
    }

    fn get_obj_enable(&self) -> bool {
        const OBJ_ENABLE_MASK: u8 = 1 << 1;
        (self.lcd_control & OBJ_ENABLE_MASK) != 0
    }

    fn get_bg_window_enable(&self) -> bool {
        const BG_WINDOW_ENABLE_MASK: u8 = 1 << 0;
        (self.lcd_control & BG_WINDOW_ENABLE_MASK) != 0
    }
}

impl Ppu {
    pub fn read_scroll_y(&self) -> u8 {
        self.scroll_y
    }

    pub fn write_scroll_y(&mut self, value: u8) {
        self.scroll_y = value;
    }

    pub fn read_scroll_x(&self) -> u8 {
        self.scroll_x
    }

    pub fn write_scroll_x(&mut self, value: u8) {
        self.scroll_x = value;
    }

    pub fn read_lcd_y(&self) -> u8 {
        self.lcd_y
    }

    pub fn read_lcd_y_compare(&self) -> u8 {
        self.lcd_y_compare
    }

    pub fn write_lcd_y_compare(&mut self, value: u8) {
        self.lcd_y_compare = value
    }

    pub fn read_window_y(&self) -> u8 {
        self.window_y
    }

    pub fn write_window_y(&mut self, value: u8) {
        self.window_y = value;
    }

    pub fn read_window_x(&self) -> u8 {
        self.window_x
    }

    pub fn write_window_x(&mut self, value: u8) {
        self.window_x = value
    }

    pub fn read_bg_palette(&self) -> u8 {
        let mut result = 0;
        for color in self.bg_palette.iter().rev() {
            result |= match color {
                PaletteColor::White => 0b00,
                PaletteColor::LightGray => 0b01,
                PaletteColor::DarkGray => 0b10,
                PaletteColor::Black => 0b11,
            };

            result <<= 2;
        }

        result
    }

    pub fn write_bg_palette(&mut self, mut value: u8) {
        for palette in self.bg_palette.iter_mut() {
            *palette = match value & 0b11 {
                0b00 => PaletteColor::White,
                0b01 => PaletteColor::LightGray,
                0b10 => PaletteColor::DarkGray,
                0b11 => PaletteColor::Black,
                _ => unreachable!(),
            };

            value >>= 2;
        }
    }

    pub fn read_obj_palette_1(&self) -> u8 {
        let mut result = 0;
        for color in self.obj_palette_1.iter().rev() {
            result |= match color {
                PaletteColor::White => 0b00,
                PaletteColor::LightGray => 0b01,
                PaletteColor::DarkGray => 0b10,
                PaletteColor::Black => 0b11,
            };

            result <<= 2;
        }

        result
    }

    pub fn write_obj_palette_1(&mut self, mut value: u8) {
        for palette in self.obj_palette_1.iter_mut() {
            *palette = match value & 0b11 {
                0b00 => PaletteColor::White,
                0b01 => PaletteColor::LightGray,
                0b10 => PaletteColor::DarkGray,
                0b11 => PaletteColor::Black,
                _ => unreachable!(),
            };

            value >>= 2;
        }
    }

    pub fn read_obj_palette_2(&self) -> u8 {
        let mut result = 0;
        for color in self.obj_palette_2.iter().rev() {
            result |= match color {
                PaletteColor::White => 0b00,
                PaletteColor::LightGray => 0b01,
                PaletteColor::DarkGray => 0b10,
                PaletteColor::Black => 0b11,
            };

            result <<= 2;
        }

        result
    }

    pub fn write_obj_palette_2(&mut self, mut value: u8) {
        for palette in self.obj_palette_2.iter_mut() {
            *palette = match value & 0b11 {
                0b00 => PaletteColor::White,
                0b01 => PaletteColor::LightGray,
                0b10 => PaletteColor::DarkGray,
                0b11 => PaletteColor::Black,
                _ => unreachable!(),
            };

            value >>= 2;
        }
    }

    pub fn read_character_ram(&self, offset: u16) -> u8 {
        self.character_ram[usize::from(offset)]
    }

    pub fn write_character_ram(&mut self, data: u8, offset: u16) {
        self.character_ram[usize::from(offset)] = data;
    }

    pub fn read_bg_map_data_1(&self, offset: u16) -> u8 {
        self.bg_map_data_1[usize::from(offset)]
    }

    pub fn write_bg_map_data_1(&mut self, data: u8, offset: u16) {
        self.bg_map_data_1[usize::from(offset)] = data;
    }

    pub fn read_bg_map_data_2(&self, offset: u16) -> u8 {
        self.bg_map_data_2[usize::from(offset)]
    }

    pub fn write_bg_map_data_2(&mut self, data: u8, offset: u16) {
        self.bg_map_data_2[usize::from(offset)] = data;
    }

    pub fn read_object_attribute_memory(&self, offset: u16) -> u8 {
        let attribute_info = &self.object_attributes[usize::from(offset / 4)];
        match offset % 4 {
            0 => attribute_info.y_position,
            1 => attribute_info.x_position,
            2 => attribute_info.tile_index,
            3 => attribute_info.flags,
            _ => unreachable!(),
        }
    }

    pub fn write_object_attribute_memory(&mut self, data: u8, offset: u16) {
        let attribute_info = &mut self.object_attributes[usize::from(offset / 4)];
        match offset % 4 {
            0 => attribute_info.y_position = data,
            1 => attribute_info.x_position = data,
            2 => attribute_info.tile_index = data,
            3 => attribute_info.flags = data,
            _ => unreachable!(),
        };
    }
}
