mod bus;
mod cartridge;
mod cpu;
mod joypad;
mod ppu;
mod serial;
mod timer;

use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::ppu::PaletteColor;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use std::error::Error;

const ROM: &[u8] = include_bytes!("../tetris.gb");

const PPU_WIDTH: u32 = 160;
const PPU_HEIGHT: u32 = 144;
const PPU_SCALE: u32 = 4;

const UP_KEYCODE: Keycode = Keycode::Up;
const DOWN_KEYCODE: Keycode = Keycode::Down;
const LEFT_KEYCODE: Keycode = Keycode::Left;
const RIGHT_KEYCODE: Keycode = Keycode::Right;

const START_KEYCODE: Keycode = Keycode::Return;
const SELECT_KEYCODE: Keycode = Keycode::RShift;
const A_KEYCODE: Keycode = Keycode::Z;
const B_KEYCODE: Keycode = Keycode::X;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cpu size: {}", std::mem::size_of::<Cpu>());
    let cartridge = Cartridge::new(ROM)?;
    let mut cpu = Cpu::new(cartridge);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "Aidan's Big-Brain GB Emulator",
            PPU_WIDTH * PPU_SCALE,
            PPU_HEIGHT * PPU_SCALE,
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    loop {
        cpu.step(false);
        if cpu.bus.ppu.should_print() {
            for (y, row) in cpu.bus.ppu.get_buffer().iter().enumerate() {
                for (x, pixel) in row.iter().cloned().enumerate() {
                    let color = match pixel {
                        PaletteColor::White => Color::WHITE,
                        PaletteColor::LightGray => Color::RGB(170, 170, 170),
                        PaletteColor::DarkGray => Color::RGB(85, 85, 85),
                        PaletteColor::Black => Color::BLACK,
                    };
                    canvas.set_draw_color(color);
                    canvas.fill_rect(Rect::new(
                        (x as i32) * (PPU_SCALE as i32),
                        (y as i32) * (PPU_SCALE as i32),
                        PPU_SCALE,
                        PPU_SCALE,
                    ))?;
                }
            }
            canvas.present();
        }
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return Ok(()),
                Event::KeyDown {
                    keycode: Some(UP_KEYCODE),
                    repeat: false,
                    ..
                } => {
                    cpu.bus.joypad.set_up_pressed(true);
                }
                Event::KeyDown {
                    keycode: Some(DOWN_KEYCODE),
                    repeat: false,
                    ..
                } => cpu.bus.joypad.set_down_pressed(true),
                Event::KeyDown {
                    keycode: Some(LEFT_KEYCODE),
                    repeat: false,
                    ..
                } => cpu.bus.joypad.set_left_pressed(true),
                Event::KeyDown {
                    keycode: Some(RIGHT_KEYCODE),
                    repeat: false,
                    ..
                } => cpu.bus.joypad.set_right_pressed(true),
                Event::KeyDown {
                    keycode: Some(SELECT_KEYCODE),
                    repeat: false,
                    ..
                } => {
                    cpu.bus.joypad.set_select_pressed(true);
                }
                Event::KeyDown {
                    keycode: Some(START_KEYCODE),
                    repeat: false,
                    ..
                } => {
                    cpu.bus.joypad.set_start_pressed(true);
                }
                Event::KeyDown {
                    keycode: Some(A_KEYCODE),
                    repeat: false,
                    ..
                } => cpu.bus.joypad.set_a_pressed(true),
                Event::KeyDown {
                    keycode: Some(B_KEYCODE),
                    repeat: false,
                    ..
                } => cpu.bus.joypad.set_b_pressed(true),
                Event::KeyUp {
                    keycode: Some(UP_KEYCODE),
                    repeat: false,
                    ..
                } => cpu.bus.joypad.set_up_pressed(false),
                Event::KeyUp {
                    keycode: Some(DOWN_KEYCODE),
                    repeat: false,
                    ..
                } => cpu.bus.joypad.set_down_pressed(false),
                Event::KeyUp {
                    keycode: Some(LEFT_KEYCODE),
                    repeat: false,
                    ..
                } => cpu.bus.joypad.set_left_pressed(false),
                Event::KeyUp {
                    keycode: Some(RIGHT_KEYCODE),
                    repeat: false,
                    ..
                } => cpu.bus.joypad.set_right_pressed(false),
                Event::KeyUp {
                    keycode: Some(SELECT_KEYCODE),
                    repeat: false,
                    ..
                } => cpu.bus.joypad.set_select_pressed(false),
                Event::KeyUp {
                    keycode: Some(START_KEYCODE),
                    repeat: false,
                    ..
                } => cpu.bus.joypad.set_start_pressed(false),
                Event::KeyUp {
                    keycode: Some(A_KEYCODE),
                    repeat: false,
                    ..
                } => cpu.bus.joypad.set_a_pressed(false),
                Event::KeyUp {
                    keycode: Some(B_KEYCODE),
                    repeat: false,
                    ..
                } => cpu.bus.joypad.set_b_pressed(false),
                _ => {}
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rom_passed(rom: &[u8]) {
        let cartridge = Cartridge::new(rom).unwrap();
        let mut cpu = Cpu::new(cartridge);

        for _ in 0..100_000_000 {
            cpu.step(false);
        }

        let serial_out = cpu.bus.serial.get_data_written();
        assert!(serial_out.contains("Passed"));
    }

    #[test]
    fn test_01_special() {
        test_rom_passed(include_bytes!("../01_special.gb"));
    }

    #[test]
    fn test_02_interrupts() {
        test_rom_passed(include_bytes!("../02_interrupts.gb"));
    }

    #[test]
    fn test_03_sp_hl() {
        test_rom_passed(include_bytes!("../03_sp_hl.gb"));
    }

    #[test]
    fn test_04_op_r_imm() {
        test_rom_passed(include_bytes!("../04_op_r_imm.gb"));
    }

    #[test]
    fn test_05_op_rp() {
        test_rom_passed(include_bytes!("../05_op_rp.gb"));
    }

    #[test]
    fn test_06_ld_r_r() {
        test_rom_passed(include_bytes!("../06_ld_r_r.gb"));
    }

    #[test]
    fn test_07_jr_jp_call_ret_rst() {
        test_rom_passed(include_bytes!("../07_jr_jp_call_ret_rst.gb"));
    }

    #[test]
    fn test_08_misc_instructions() {
        test_rom_passed(include_bytes!("../08_misc_instructions.gb"));
    }

    #[test]
    fn test_09_op_r_r() {
        test_rom_passed(include_bytes!("../09_op_r_r.gb"));
    }

    #[test]
    fn test_10_bit_ops() {
        test_rom_passed(include_bytes!("../10_bit_ops.gb"));
    }

    #[test]
    fn test_11_op_a_hl() {
        test_rom_passed(include_bytes!("../11_op_a_hl.gb"));
    }
}
