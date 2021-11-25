use std::collections::HashSet;
use std::convert::TryFrom;
use std::default::Default;
use std::fmt::Debug;

pub const PPU_WIDTH: usize = 160;
pub const PPU_HEIGHT: usize = 144;

#[derive(Clone, Copy, Debug)]
pub enum PpuRenderStatus {
    HBlank,
    VBlank,
    OAMSearch,
    PixelTransfer,
}

#[derive(Clone, Copy, Debug)]
pub enum PpuMode {
    Cgb,
    Dmg,
    Pgb,
}

#[derive(Clone, Copy, Debug)]
enum ObjSize {
    EightByEight,
    EightBySixteen,
}

#[derive(Clone, Copy, Debug)]
enum StatInterruptSource {
    LycEqualsLy,
    OAMSearch,
    HBlank,
    VBlank,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PaletteColorRgb555 {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Clone, Copy, Default)]
struct SpriteAttributeInfo {
    pub y_position: u8,
    pub x_position: u8,
    pub tile_index: u8,
    pub flags: u8,
}

#[derive(Clone, Copy, Debug)]
struct BackgroundPixelInfo {
    pub color: PaletteColorRgb555,
    pub palette_idx: usize,
    pub priority_over_sprite: bool,
}

#[derive(Clone, Copy, Debug)]
struct SpritePixelInfo {
    pub color: PaletteColorRgb555,
    pub palette_idx: usize,
    pub priority_under_bg: bool,
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

    fn use_low_grayscale_palette(&self) -> bool {
        const LOW_PALETTE_MASK: u8 = 1 << 4;
        (self.flags & LOW_PALETTE_MASK) == 0
    }

    fn get_tile_vram_bank(&self) -> u8 {
        const TILE_VRAM_BANK_SHIFT: u8 = 3;
        const TILE_VRAM_BANK_MASK: u8 = 1 << TILE_VRAM_BANK_SHIFT;

        (self.flags & TILE_VRAM_BANK_MASK) >> TILE_VRAM_BANK_SHIFT
    }

    fn get_rgb_palette_number(&self) -> u8 {
        const RGB_PALETTE_NUMBER_MASK: u8 = 0b111;

        self.flags & RGB_PALETTE_NUMBER_MASK
    }
}

#[derive(Clone, Copy, Default)]
struct TileMapAttributeInfo {
    pub tile_idx: u8,
    flags: u8,
}

impl TileMapAttributeInfo {
    fn bg_has_priority(&self) -> bool {
        const BG_HAS_PRIORITY_MASK: u8 = 1 << 7;

        (self.flags & BG_HAS_PRIORITY_MASK) == BG_HAS_PRIORITY_MASK
    }

    fn get_y_flip(&self) -> bool {
        const Y_FLIP_MASK: u8 = 1 << 6;
        (self.flags & Y_FLIP_MASK) == Y_FLIP_MASK
    }

    fn get_x_flip(&self) -> bool {
        const X_FLIP_MASK: u8 = 1 << 5;
        (self.flags & X_FLIP_MASK) == X_FLIP_MASK
    }

    fn get_tile_vram_bank_number(&self) -> u8 {
        const TILE_VRAM_BANK_NUMBER_SHIFT: u8 = 3;
        const TILE_VRAM_BANK_NUMBER_MASK: u8 = 1 << TILE_VRAM_BANK_NUMBER_SHIFT;

        (self.flags & TILE_VRAM_BANK_NUMBER_MASK) >> TILE_VRAM_BANK_NUMBER_SHIFT
    }

    fn get_palette_number(&self) -> u8 {
        const BACKGROUND_PALETTE_NUMBER_MASK: u8 = 0b111;

        self.flags & BACKGROUND_PALETTE_NUMBER_MASK
    }
}

#[derive(Clone)]
pub struct Ppu {
    tile_data: Box<[[u8; 0x1800]; 2]>,
    bg_map_0: Box<[TileMapAttributeInfo; 0x400]>,
    bg_map_1: Box<[TileMapAttributeInfo; 0x400]>,
    vram_bank_index: u8,
    object_attributes: Box<[SpriteAttributeInfo; 40]>,
    vblank_interrupt_waiting: bool,
    stat_interrupt_waiting: bool,
    dot: u16,
    lcd_y: u8,
    window_lcd_y: u8,
    window_y_condition_triggered: bool,
    window_x_condition_triggered: bool,
    lcd_y_compare: u8,
    stat: u8,
    lcd_control: u8,
    scroll_x: u8,
    scroll_y: u8,
    window_x: u8,
    window_y: u8,
    back_buffer: Box<[[PaletteColorRgb555; 160]; 144]>, // access as buffer[y][x]
    front_buffer: Box<[[PaletteColorRgb555; 160]; 144]>, // access as buffer[y][x]
    bg_palette: u8,
    obj_palette_0: u8,
    obj_palette_1: u8,
    scanline_seen_sprites: HashSet<usize>,
    bg_color_palette_index: u8,
    bg_color_palette_data: Box<[[PaletteColorRgb555; 4]; 8]>,
    obj_color_palette_index: u8,
    obj_color_palette_data: Box<[[PaletteColorRgb555; 4]; 8]>,
    dmg_mode: bool,
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            tile_data: Box::new([[0; 0x1800]; 2]),
            bg_map_0: Box::new([TileMapAttributeInfo::default(); 0x400]),
            bg_map_1: Box::new([TileMapAttributeInfo::default(); 0x400]),
            vram_bank_index: 0,
            object_attributes: Box::new([SpriteAttributeInfo::default(); 40]),
            vblank_interrupt_waiting: Default::default(),
            stat_interrupt_waiting: Default::default(),
            dot: Default::default(),
            lcd_y: Default::default(),
            window_lcd_y: Default::default(),
            window_x_condition_triggered: Default::default(),
            window_y_condition_triggered: Default::default(),
            lcd_y_compare: Default::default(),
            stat: Default::default(),
            lcd_control: Default::default(),
            scroll_x: Default::default(),
            scroll_y: Default::default(),
            window_x: Default::default(),
            window_y: Default::default(),
            back_buffer: Box::new([[PaletteColorRgb555::default(); 160]; 144]),
            front_buffer: Box::new([[PaletteColorRgb555::default(); 160]; 144]),
            bg_palette: Default::default(),
            obj_palette_0: Default::default(),
            obj_palette_1: Default::default(),
            scanline_seen_sprites: HashSet::default(),
            bg_color_palette_index: Default::default(),
            bg_color_palette_data: Box::new([[PaletteColorRgb555::default(); 4]; 8]),
            obj_color_palette_index: Default::default(),
            obj_color_palette_data: Box::new([[PaletteColorRgb555::default(); 4]; 8]),
            dmg_mode: false,
        }
    }
}

impl Ppu {
    pub fn step(&mut self) {
        // If lcd/ppu is disabled, don't do anything.
        if !self.get_lcd_ppu_enable() {
            return;
        }

        if self.lcd_y == self.lcd_y_compare {
            self.set_stat_lyc_equals_ly(true);
        } else {
            self.set_stat_lyc_equals_ly(false);
        }

        if self.lcd_y < 144 {
            if self.dot == 0 {
                self.set_stat_mode(PpuRenderStatus::OAMSearch);
                self.window_y_condition_triggered |= self.lcd_y == self.window_y
            } else if self.dot == 80 {
                self.set_stat_mode(PpuRenderStatus::PixelTransfer);

                self.scanline_seen_sprites.clear();

                let obj_size = self.get_obj_size();
                self.scanline_seen_sprites.extend(
                    self.object_attributes
                        .iter()
                        .enumerate()
                        .filter(|(_, attribute_info)| match obj_size {
                            ObjSize::EightByEight => {
                                self.lcd_y + 16 >= attribute_info.y_position
                                    && self.lcd_y + 8 < attribute_info.y_position
                            }
                            ObjSize::EightBySixteen => {
                                self.lcd_y + 16 >= attribute_info.y_position
                                    && self.lcd_y < attribute_info.y_position
                            }
                        })
                        .map(|(i, _)| i)
                        .take(10),
                );
            } else if self.dot == 252 {
                self.set_stat_mode(PpuRenderStatus::HBlank);

                // Window displayed falling edge increments hidden window lcd y.
                let old_window_displayed = self.get_window_displayed();
                self.window_x_condition_triggered = false;
                let new_window_displayed = self.get_window_displayed();
                if old_window_displayed && !new_window_displayed {
                    self.window_lcd_y += 1;
                }
            }
        } else if self.lcd_y == 144 {
            if self.dot == 0 {
                self.set_stat_mode(PpuRenderStatus::VBlank);
                self.vblank_interrupt_waiting = true;
                self.window_y_condition_triggered = false;
            }
        }

        if matches!(self.get_stat_mode(), PpuRenderStatus::PixelTransfer) {
            let buffer_x = u8::try_from(self.dot - 80).unwrap();
            let buffer_y = self.lcd_y;

            if buffer_x < 160 {
                let background_pixel_info = self.get_background_pixel(buffer_x, buffer_y);

                self.back_buffer[usize::from(buffer_y)][usize::from(buffer_x)] =
                    background_pixel_info.color;

                // window_x is "actual_window_x + 7". Values less than 7 result in
                // buggy behavior. For now, when window_x < 7, trigger window x
                // condition iff render_x == 0.
                if self.window_x >= 7 {
                    self.window_x_condition_triggered |= buffer_x + 7 == self.window_x;
                } else {
                    self.window_x_condition_triggered |= buffer_x == 0;
                };

                let window_pixel_info = self.get_window_pixel(buffer_x);
                if let Some(BackgroundPixelInfo { color, .. }) = window_pixel_info {
                    self.back_buffer[usize::from(buffer_y)][usize::from(buffer_x)] = color;
                }

                let sprite_pixel_info = self.get_sprite_pixel(buffer_x, buffer_y);
                if let Some(SpritePixelInfo {
                    color,
                    priority_under_bg,
                    ..
                }) = sprite_pixel_info
                {
                    let window_drawn =
                        window_pixel_info.map_or(false, |info| info.palette_idx != 0);
                    let background_drawn = background_pixel_info.palette_idx != 0;

                    let window_over_sprite = window_pixel_info
                        .map_or(false, |info| info.priority_over_sprite && window_drawn);
                    let background_over_sprite =
                        background_pixel_info.priority_over_sprite && background_drawn;
                    let sprite_under_bg_window =
                        priority_under_bg && (background_drawn || window_drawn);

                    let sprite_drawn = if self.get_bg_window_enable_priority() {
                        !(background_over_sprite || window_over_sprite || sprite_under_bg_window)
                    } else {
                        true
                    };

                    if sprite_drawn {
                        self.back_buffer[usize::from(buffer_y)][usize::from(buffer_x)] = color;
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
                self.window_lcd_y = 0;
                self.front_buffer = self.back_buffer.clone();
            }
        }
    }

    fn get_background_pixel(&self, pixel_x: u8, pixel_y: u8) -> BackgroundPixelInfo {
        let bg_render_x = u16::from(pixel_x.wrapping_add(self.scroll_x));
        let bg_render_y = u16::from(pixel_y.wrapping_add(self.scroll_y));

        let bg_tile_x = bg_render_x / 8;
        let bg_tile_y = bg_render_y / 8;
        let bg_tile_map_idx = bg_tile_x + (bg_tile_y * 32);

        let bg_tile_attributes = self.get_bg_tile_attributes(bg_tile_map_idx);
        let bg_tile_data = self.get_bg_window_tile_data(bg_tile_attributes);

        let bg_tile_row = if bg_tile_attributes.get_y_flip() {
            7 - (bg_render_y % 8)
        } else {
            bg_render_y % 8
        };

        let bg_lsb_row_color = bg_tile_data[usize::from(bg_tile_row) * 2];
        let bg_msb_row_color = bg_tile_data[(usize::from(bg_tile_row) * 2) + 1];

        let bg_tile_col = if bg_tile_attributes.get_x_flip() {
            7 - (bg_render_x % 8)
        } else {
            bg_render_x % 8
        };

        let bg_lsb_pixel_color = (bg_lsb_row_color & (1 << (7 - bg_tile_col))) != 0;
        let bg_msb_pixel_color = (bg_msb_row_color & (1 << (7 - bg_tile_col))) != 0;
        let bg_pixel_palette_idx =
            (usize::from(bg_msb_pixel_color) << 1) | usize::from(bg_lsb_pixel_color);

        let result_color =
            self.get_background_palette_color(bg_tile_attributes, bg_pixel_palette_idx);

        BackgroundPixelInfo {
            color: result_color,
            palette_idx: bg_pixel_palette_idx,
            priority_over_sprite: bg_tile_attributes.bg_has_priority(),
        }
    }

    fn get_window_pixel(&self, pixel_x: u8) -> Option<BackgroundPixelInfo> {
        if self.get_window_displayed() {
            let window_render_x = u16::from(pixel_x + 7 - self.window_x);
            let window_render_y = u16::from(self.window_lcd_y);

            let window_tile_x = window_render_x / 8;
            let window_tile_y = window_render_y / 8;
            let window_tile_map_idx = window_tile_x + (window_tile_y * 32);

            let window_tile_attributes = self.get_window_tile_attributes(window_tile_map_idx);
            let window_tile_data = self.get_bg_window_tile_data(window_tile_attributes);

            let window_tile_row = if window_tile_attributes.get_y_flip() {
                7 - (window_render_y % 8)
            } else {
                window_render_y % 8
            };

            let window_lsb_row_color = window_tile_data[usize::from(window_tile_row) * 2];
            let window_msb_row_color = window_tile_data[(usize::from(window_tile_row) * 2) + 1];

            let window_tile_col = if window_tile_attributes.get_x_flip() {
                7 - (window_render_x % 8)
            } else {
                window_render_x % 8
            };
            let window_lsb_pixel_color = (window_lsb_row_color & (1 << (7 - window_tile_col))) != 0;
            let window_msb_pixel_color = (window_msb_row_color & (1 << (7 - window_tile_col))) != 0;
            let window_pixel_palette_idx =
                (usize::from(window_msb_pixel_color) << 1) | usize::from(window_lsb_pixel_color);

            let result_color =
                self.get_background_palette_color(window_tile_attributes, window_pixel_palette_idx);

            Some(BackgroundPixelInfo {
                color: result_color,
                palette_idx: window_pixel_palette_idx,
                priority_over_sprite: window_tile_attributes.bg_has_priority(),
            })
        } else {
            None
        }
    }

    fn get_sprite_pixel(&self, pixel_x: u8, pixel_y: u8) -> Option<SpritePixelInfo> {
        if self.get_obj_enable() {
            for sprite_attribute_info in self.object_attributes.into_iter() {
                match self.get_obj_size() {
                    ObjSize::EightByEight => {
                        if pixel_y + 16 >= sprite_attribute_info.y_position
                            && pixel_y + 8 < sprite_attribute_info.y_position
                            && pixel_x + 8 >= sprite_attribute_info.x_position
                            && pixel_x < sprite_attribute_info.x_position
                        {
                            let sprite_y_offset = if sprite_attribute_info.get_y_flip() {
                                7 - (pixel_y + 16 - sprite_attribute_info.y_position)
                            } else {
                                pixel_y + 16 - sprite_attribute_info.y_position
                            };

                            let sprite_x_offset = if sprite_attribute_info.get_x_flip() {
                                7 - (pixel_x + 8 - sprite_attribute_info.x_position)
                            } else {
                                pixel_x + 8 - sprite_attribute_info.x_position
                            };

                            let sprite_data =
                                self.get_obj_tile_data(sprite_attribute_info, sprite_y_offset);

                            let lsb_row_color = sprite_data[usize::from(sprite_y_offset) * 2];
                            let msb_row_color = sprite_data[(usize::from(sprite_y_offset) * 2) + 1];

                            let lsb_pixel_color =
                                (lsb_row_color & (1 << (7 - sprite_x_offset))) != 0;
                            let msb_pixel_color =
                                (msb_row_color & (1 << (7 - sprite_x_offset))) != 0;

                            let sprite_pixel_palette_idx =
                                (usize::from(msb_pixel_color) << 1) | usize::from(lsb_pixel_color);

                            if sprite_pixel_palette_idx != 0 {
                                let pixel_color = self.get_obj_palette_color(
                                    sprite_attribute_info,
                                    sprite_pixel_palette_idx,
                                );
                                return Some(SpritePixelInfo {
                                    color: pixel_color,
                                    palette_idx: sprite_pixel_palette_idx,
                                    priority_under_bg: sprite_attribute_info
                                        .get_bg_window_over_obj(),
                                });
                            }
                        }
                    }
                    ObjSize::EightBySixteen => {
                        if pixel_y + 16 >= sprite_attribute_info.y_position
                            && pixel_y < sprite_attribute_info.y_position
                            && pixel_x + 8 >= sprite_attribute_info.x_position
                            && pixel_x < sprite_attribute_info.x_position
                        {
                            let sprite_y_offset = if sprite_attribute_info.get_y_flip() {
                                15 - (pixel_y + 16 - sprite_attribute_info.y_position)
                            } else {
                                pixel_y + 16 - sprite_attribute_info.y_position
                            };

                            let sprite_x_offset = if sprite_attribute_info.get_x_flip() {
                                7 - (pixel_x + 8 - sprite_attribute_info.x_position)
                            } else {
                                pixel_x + 8 - sprite_attribute_info.x_position
                            };

                            let sprite_data =
                                self.get_obj_tile_data(sprite_attribute_info, sprite_y_offset);

                            let lsb_row_color = sprite_data[usize::from(sprite_y_offset % 8) * 2];
                            let msb_row_color =
                                sprite_data[(usize::from(sprite_y_offset % 8) * 2) + 1];

                            let lsb_pixel_color =
                                (lsb_row_color & (1 << (7 - sprite_x_offset))) != 0;
                            let msb_pixel_color =
                                (msb_row_color & (1 << (7 - sprite_x_offset))) != 0;

                            let sprite_pixel_palette_idx =
                                (usize::from(msb_pixel_color) << 1) | usize::from(lsb_pixel_color);

                            if sprite_pixel_palette_idx != 0 {
                                let pixel_color = self.get_obj_palette_color(
                                    sprite_attribute_info,
                                    sprite_pixel_palette_idx,
                                );
                                return Some(SpritePixelInfo {
                                    color: pixel_color,
                                    palette_idx: sprite_pixel_palette_idx,
                                    priority_under_bg: sprite_attribute_info
                                        .get_bg_window_over_obj(),
                                });
                            }
                        }
                    }
                };
            }
        }

        None
    }

    pub fn get_buffer(&self) -> &[[PaletteColorRgb555; 160]; 144] {
        &self.front_buffer
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
            PpuRenderStatus::HBlank => {
                self.stat_interrupt_source_enabled(StatInterruptSource::HBlank)
            }
            PpuRenderStatus::OAMSearch => {
                self.stat_interrupt_source_enabled(StatInterruptSource::OAMSearch)
            }
            PpuRenderStatus::PixelTransfer => false,
            PpuRenderStatus::VBlank => {
                self.stat_interrupt_source_enabled(StatInterruptSource::VBlank)
            }
        };

        lyc_equals_ly_interrupt_line || mode_interrupt_line
    }

    const STAT_MODE_MASK: u8 = 0b0000_0011;
    const STAT_HBLANK_MODE_MASK: u8 = 0b0000_0000;
    const STAT_VBLANK_MODE_MASK: u8 = 0b0000_0001;
    const STAT_OAM_SEARCH_MODE_MASK: u8 = 0b0000_0010;
    const STAT_PIXEL_TRANSFER_MODE_MASK: u8 = 0b0000_0011;

    fn set_stat_mode(&mut self, mode: PpuRenderStatus) {
        let old_interrupt_line = self.get_stat_interrupt_line();
        let line_high = match mode {
            PpuRenderStatus::HBlank => {
                self.stat_interrupt_source_enabled(StatInterruptSource::HBlank)
            }
            PpuRenderStatus::OAMSearch => {
                self.stat_interrupt_source_enabled(StatInterruptSource::OAMSearch)
            }
            PpuRenderStatus::PixelTransfer => false,
            PpuRenderStatus::VBlank => {
                self.stat_interrupt_source_enabled(StatInterruptSource::VBlank)
            }
        };

        if !old_interrupt_line && line_high {
            self.stat_interrupt_waiting = true
        }

        self.stat &= !Self::STAT_MODE_MASK;
        self.stat |= match mode {
            PpuRenderStatus::HBlank => Self::STAT_HBLANK_MODE_MASK,
            PpuRenderStatus::VBlank => Self::STAT_VBLANK_MODE_MASK,
            PpuRenderStatus::OAMSearch => Self::STAT_OAM_SEARCH_MODE_MASK,
            PpuRenderStatus::PixelTransfer => Self::STAT_PIXEL_TRANSFER_MODE_MASK,
        };
    }

    pub fn get_stat_mode(&self) -> PpuRenderStatus {
        match self.stat & Self::STAT_MODE_MASK {
            Self::STAT_HBLANK_MODE_MASK => PpuRenderStatus::HBlank,
            Self::STAT_VBLANK_MODE_MASK => PpuRenderStatus::VBlank,
            Self::STAT_OAM_SEARCH_MODE_MASK => PpuRenderStatus::OAMSearch,
            Self::STAT_PIXEL_TRANSFER_MODE_MASK => PpuRenderStatus::PixelTransfer,
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

impl Ppu {
    pub fn read_lcd_control(&self) -> u8 {
        self.lcd_control
    }

    pub fn write_lcd_control(&mut self, data: u8) {
        let old_window_displayed = self.get_window_displayed();
        self.lcd_control = data;
        let new_window_displayed = self.get_window_displayed();

        // Window displayed falling edge increments hidden window lcd y.
        if old_window_displayed && !new_window_displayed {
            self.window_lcd_y += 1;
        }

		if !self.get_lcd_ppu_enable() {
			self.stat = 0;
			self.dot = 0;
			self.lcd_y = 0;
		}
    }

    pub fn get_lcd_ppu_enable(&self) -> bool {
        const LCD_PPU_ENABLE_MASK: u8 = 1 << 7;
        (self.lcd_control & LCD_PPU_ENABLE_MASK) == LCD_PPU_ENABLE_MASK
    }

    fn get_window_tile_attributes(&self, index: u16) -> TileMapAttributeInfo {
        const WINDOW_TILE_MAP_AREA_MASK: u8 = 1 << 6;

        if (self.lcd_control & WINDOW_TILE_MAP_AREA_MASK) != WINDOW_TILE_MAP_AREA_MASK {
            self.bg_map_0[usize::from(index)]
        } else {
            self.bg_map_1[usize::from(index)]
        }
    }

    fn get_window_enable(&self) -> bool {
        const WINDOW_ENABLE_MASK: u8 = 1 << 5;
        (self.lcd_control & WINDOW_ENABLE_MASK) == WINDOW_ENABLE_MASK
    }

    fn get_window_displayed(&self) -> bool {
        self.window_x_condition_triggered
            && self.window_y_condition_triggered
            && self.get_window_enable()
    }

    fn get_bg_window_tile_data(&self, tile_map_attributes: TileMapAttributeInfo) -> &[u8] {
        const BG_WINDOW_TILE_DATA_AREA_MASK: u8 = 1 << 4;
        // When LCDC.4 == 0 and tile_id < 128, we start indexing at an offset of
        // 0x1000. In all other situations, start indexing at 0x0000.
        if (self.lcd_control & BG_WINDOW_TILE_DATA_AREA_MASK) != BG_WINDOW_TILE_DATA_AREA_MASK
            && tile_map_attributes.tile_idx < 128
        {
            &self.tile_data[usize::from(tile_map_attributes.get_tile_vram_bank_number())][0x1000..]
                [usize::from(tile_map_attributes.tile_idx) * 16..][..16]
        } else {
            &self.tile_data[usize::from(tile_map_attributes.get_tile_vram_bank_number())]
                [usize::from(tile_map_attributes.tile_idx) * 16..][..16]
        }
    }

    fn get_bg_tile_attributes(&self, index: u16) -> TileMapAttributeInfo {
        const BG_TILE_MAP_AREA_MASK: u8 = 1 << 3;

        if (self.lcd_control & BG_TILE_MAP_AREA_MASK) == 0 {
            self.bg_map_0[usize::from(index)]
        } else {
            self.bg_map_1[usize::from(index)]
        }
    }

    fn get_obj_tile_data(&self, attribute_info: SpriteAttributeInfo, sprite_y_offset: u8) -> &[u8] {
        let real_tile_idx = match self.get_obj_size() {
            ObjSize::EightByEight => attribute_info.tile_index,
            ObjSize::EightBySixteen if sprite_y_offset < 8 => attribute_info.tile_index & 0xFE,
            ObjSize::EightBySixteen => attribute_info.tile_index | 0x01,
        };

        if self.dmg_mode {
            assert_eq!(attribute_info.get_tile_vram_bank(), 0);
            &self.tile_data[0][usize::from(real_tile_idx) * 16..][..16]
        } else {
            &self.tile_data[usize::from(attribute_info.get_tile_vram_bank())]
                [usize::from(real_tile_idx) * 16..][..16]
        }
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

    fn get_bg_window_enable_priority(&self) -> bool {
        const BG_WINDOW_ENABLE_MASK: u8 = 1 << 0;
        (self.lcd_control & BG_WINDOW_ENABLE_MASK) != 0
    }

    fn get_background_color_palette_auto_increment(&self) -> bool {
        const BACKGROUND_PALETTE_AUTO_INCREMENT_MASK: u8 = 1 << 7;

        (self.bg_color_palette_index & BACKGROUND_PALETTE_AUTO_INCREMENT_MASK)
            == BACKGROUND_PALETTE_AUTO_INCREMENT_MASK
    }

    const BACKGROUND_COLOR_PALETTE_ADDRESS_MASK: u8 = 0b0011_1111;

    fn get_background_color_palette_address(&self) -> u8 {
        self.bg_color_palette_index & Self::BACKGROUND_COLOR_PALETTE_ADDRESS_MASK
    }

    fn set_background_color_palette_address(&mut self, value: u8) {
        self.bg_color_palette_index &= !Self::BACKGROUND_COLOR_PALETTE_ADDRESS_MASK;
        self.bg_color_palette_index |= value & Self::BACKGROUND_COLOR_PALETTE_ADDRESS_MASK
    }

    fn get_obj_color_palette_auto_increment(&self) -> bool {
        const OBJ_PALETTE_AUTO_INCREMENT_MASK: u8 = 1 << 7;

        (self.obj_color_palette_index & OBJ_PALETTE_AUTO_INCREMENT_MASK)
            == OBJ_PALETTE_AUTO_INCREMENT_MASK
    }

    const OBJ_COLOR_PALETTE_ADDRESS_MASK: u8 = 0b0011_1111;

    fn get_obj_color_palette_address(&self) -> u8 {
        self.obj_color_palette_index & Self::OBJ_COLOR_PALETTE_ADDRESS_MASK
    }

    fn set_obj_color_palette_address(&mut self, value: u8) {
        self.obj_color_palette_index &= !Self::OBJ_COLOR_PALETTE_ADDRESS_MASK;
        self.obj_color_palette_index |= value & Self::OBJ_COLOR_PALETTE_ADDRESS_MASK
    }

    fn get_background_palette_color(
        &self,
        attribute_info: TileMapAttributeInfo,
        palette_index: usize,
    ) -> PaletteColorRgb555 {
        if self.dmg_mode {
            let color_palette_idx = match palette_index {
                0 => (self.bg_palette >> 0) & 0b11,
                1 => (self.bg_palette >> 2) & 0b11,
                2 => (self.bg_palette >> 4) & 0b11,
                3 => (self.bg_palette >> 6) & 0b11,
                _ => unreachable!(),
            };

            self.bg_color_palette_data[0][usize::from(color_palette_idx)]
        } else {
            self.bg_color_palette_data[usize::from(attribute_info.get_palette_number())]
                [usize::from(palette_index)]
        }
    }

    fn get_obj_palette_color(
        &self,
        attribute_info: SpriteAttributeInfo,
        palette_index: usize,
    ) -> PaletteColorRgb555 {
        if self.dmg_mode {
            let used_obj_palette = if attribute_info.use_low_grayscale_palette() {
                self.obj_palette_0
            } else {
                self.obj_palette_1
            };

            let color_palette_idx = match palette_index {
                0 => (used_obj_palette >> 0) & 0b11,
                1 => (used_obj_palette >> 2) & 0b11,
                2 => (used_obj_palette >> 4) & 0b11,
                3 => (used_obj_palette >> 6) & 0b11,
                _ => unreachable!(),
            };

            if attribute_info.use_low_grayscale_palette() {
                self.obj_color_palette_data[0][usize::from(color_palette_idx)]
            } else {
                self.obj_color_palette_data[1][usize::from(color_palette_idx)]
            }
        } else {
            self.obj_color_palette_data[usize::from(attribute_info.get_rgb_palette_number())]
                [palette_index]
        }
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
        let old_window_displayed = self.get_window_displayed();
        self.window_y = value;
        self.window_y_condition_triggered = false;
        let new_window_displayed = self.get_window_displayed();

        // Window displayed falling edge increments hidden window lcd y.
        if old_window_displayed && !new_window_displayed {
            self.window_lcd_y += 1;
        }
    }

    pub fn read_window_x(&self) -> u8 {
        self.window_x
    }

    pub fn write_window_x(&mut self, value: u8) {
        let old_window_displayed = self.get_window_displayed();
        self.window_x = value;
        self.window_x_condition_triggered = false;
        let new_window_displayed = self.get_window_displayed();

        // Window displayed falling edge increments hidden window lcd y.
        if old_window_displayed && !new_window_displayed {
            self.window_lcd_y += 1;
        }
    }

    pub fn read_bg_palette(&self) -> u8 {
        self.bg_palette
    }

    pub fn write_bg_palette(&mut self, value: u8) {
        self.bg_palette = value;
    }

    pub fn read_obj_palette_0(&self) -> u8 {
        self.obj_palette_0
    }

    pub fn write_obj_palette_0(&mut self, value: u8) {
        self.obj_palette_0 = value;
    }

    pub fn read_obj_palette_1(&self) -> u8 {
        self.obj_palette_1
    }

    pub fn write_obj_palette_1(&mut self, value: u8) {
        self.obj_palette_1 = value;
    }

    pub fn read_vram(&self, offset: u16) -> u8 {
        match offset {
            0x0000..=0x17FF => {
                self.tile_data[usize::from(self.vram_bank_index)][usize::from(offset)]
            }
            0x1800..=0x1BFF => match self.vram_bank_index {
                0 => self.bg_map_0[usize::from(offset - 0x1800)].tile_idx,
                1 => self.bg_map_0[usize::from(offset - 0x1800)].flags,
                _ => unreachable!(),
            },
            0x1C00..=0x1FFF => match self.vram_bank_index {
                0 => self.bg_map_1[usize::from(offset - 0x1C00)].tile_idx,
                1 => self.bg_map_1[usize::from(offset - 0x1C00)].flags,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    pub fn write_vram(&mut self, value: u8, offset: u16) {
        match offset {
            0x0000..=0x17FF => {
                self.tile_data[usize::from(self.vram_bank_index)][usize::from(offset)] = value
            }
            0x1800..=0x1BFF => match self.vram_bank_index {
                0 => self.bg_map_0[usize::from(offset - 0x1800)].tile_idx = value,
                1 => self.bg_map_0[usize::from(offset - 0x1800)].flags = value,
                _ => unreachable!(),
            },
            0x1C00..=0x1FFF => match self.vram_bank_index {
                0 => self.bg_map_1[usize::from(offset - 0x1C00)].tile_idx = value,
                1 => self.bg_map_1[usize::from(offset - 0x1C00)].flags = value,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
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

    pub fn read_background_palette_index(&self) -> u8 {
        self.bg_color_palette_index
    }

    pub fn write_background_palette_index(&mut self, value: u8) {
        self.bg_color_palette_index = value
    }

    pub fn read_background_palette_data(&self) -> u8 {
        let address = self.get_background_color_palette_address();
        let palette_idx = usize::from((address >> 1) / 4);
        let color_idx = usize::from((address >> 1) % 4);

        let color_low = (address & 0b1) != 0b1;
        let color = self.bg_color_palette_data[palette_idx][color_idx];

        if color_low {
            (color.red >> 3) | ((color.green & 0b0000_0111) << 5)
        } else {
            ((color.green & 0b0011_1000) >> 3) | (color.blue << 3)
        }
    }

    pub fn write_background_palette_data(&mut self, value: u8) {
        let address = self.get_background_color_palette_address();
        let palette_idx = usize::from((address >> 1) / 4);
        let color_idx = usize::from((address >> 1) % 4);

        let color_low = (address & 0b1) != 0b1;
        let color = &mut self.bg_color_palette_data[palette_idx][color_idx];

        if color_low {
            color.red = value & 0b0001_1111;
            color.green &= !0b0000_0111;
            color.green |= (value >> 5) & 0b0000_0111;
        } else {
            color.green &= !0b0001_1000;
            color.green |= (value << 3) & 0b0001_1000;
            color.blue = (value >> 2) & 0b0001_1111;
        }

        if self.get_background_color_palette_auto_increment() {
            self.set_background_color_palette_address(address + 1);
        }
    }

    pub fn read_obj_palette_index(&self) -> u8 {
        self.obj_color_palette_index
    }

    pub fn write_obj_palette_index(&mut self, value: u8) {
        self.obj_color_palette_index = value
    }

    pub fn read_obj_palette_data(&self) -> u8 {
        let address = self.get_obj_color_palette_address();
        let palette_idx = usize::from((address >> 1) / 4);
        let color_idx = usize::from((address >> 1) % 4);

        let color_low = (address & 0b1) != 0b1;
        let color = self.obj_color_palette_data[palette_idx][color_idx];

        if color_low {
            (color.red >> 3) | ((color.green & 0b0000_0111) << 5)
        } else {
            ((color.green & 0b0011_1000) >> 3) | (color.blue << 3)
        }
    }

    pub fn write_obj_palette_data(&mut self, value: u8) {
        let address = self.get_obj_color_palette_address();
        let palette_idx = usize::from((address >> 1) / 4);
        let color_idx = usize::from((address >> 1) % 4);

        let color_low = (address & 0b1) != 0b1;
        let color = &mut self.obj_color_palette_data[palette_idx][color_idx];

        if color_low {
            color.red = value & 0b0001_1111;
            color.green &= !0b0000_0111;
            color.green |= (value >> 5) & 0b0000_0111;
        } else {
            color.green &= !0b0001_1000;
            color.green |= (value << 3) & 0b0001_1000;
            color.blue = (value >> 2) & 0b0001_1111;
        }

        if self.get_obj_color_palette_auto_increment() {
            self.set_obj_color_palette_address(address + 1);
        }
    }

    pub fn read_vram_bank(&self) -> u8 {
        self.vram_bank_index
    }

    pub fn write_vram_bank(&mut self, value: u8) {
        self.vram_bank_index = value & 0b1;
    }

    pub fn set_ppu_mode(&mut self, mode: PpuMode) {
        match mode {
            PpuMode::Cgb => self.dmg_mode = false,
            PpuMode::Dmg => self.dmg_mode = true,
            PpuMode::Pgb => unimplemented!(),
        };
        println!("mode: {:?}", mode);
    }
}
