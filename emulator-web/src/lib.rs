use emulator_core::{
    cartridge::Cartridge,
    cpu::Cpu,
    joypad::Button,
    ppu::{PPU_HEIGHT, PPU_WIDTH},
};

use wasm_bindgen::prelude::*;

#[derive(Debug)]
#[wasm_bindgen]
pub struct Emulator {
    emulator: Cpu,
}

#[wasm_bindgen]
pub enum ButtonType {
    A,
    B,
    Up,
    Down,
    Left,
    Right,
    Select,
    Start,
}

#[wasm_bindgen(js_name = getSize)]
pub fn get_size() -> usize {
    std::mem::size_of::<Cpu>()
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new(data: Box<[u8]>) -> Emulator {
        let cartridge = Cartridge::new(data.as_ref()).expect("invalid cartridge data");

        Self {
            emulator: Cpu::new(cartridge),
        }
    }

    pub fn step(&mut self) {
        self.emulator.fetch_decode_execute();
    }

    pub fn buffer(&self) -> Vec<u8> {
        let ppu_buffer = self.emulator.bus.ppu.get_buffer();
        ppu_buffer
            .into_iter()
            .flatten()
            .flat_map(|palette_color| {
                [palette_color.red, palette_color.green, palette_color.blue]
                    .map(|color| (color << 3) | (color >> 2))
            })
            .collect()
    }

    pub fn set_button_pressed(&mut self, button: ButtonType, pressed: bool) {
        match button {
            ButtonType::A => self.emulator.set_button_pressed(Button::A, pressed),
            ButtonType::B => self.emulator.set_button_pressed(Button::B, pressed),
            ButtonType::Up => self.emulator.set_button_pressed(Button::Up, pressed),
            ButtonType::Down => self.emulator.set_button_pressed(Button::Down, pressed),
            ButtonType::Left => self.emulator.set_button_pressed(Button::Left, pressed),
            ButtonType::Right => self.emulator.set_button_pressed(Button::Right, pressed),
            ButtonType::Select => self.emulator.set_button_pressed(Button::Select, pressed),
            ButtonType::Start => self.emulator.set_button_pressed(Button::Start, pressed),
        };
    }

    #[wasm_bindgen(js_name = ppuWidth)]
    pub fn ppu_width() -> usize {
        PPU_WIDTH
    }

    #[wasm_bindgen(js_name = ppuHeight)]
    pub fn ppu_height() -> usize {
        PPU_HEIGHT
    }
}
