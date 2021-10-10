mod apu;
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

const ROM: &[u8] = include_bytes!("../super_mario_land_2.gb");

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

const DEBUG_KEYCODE: Keycode = Keycode::D;
const SAVE_KEYCODE: Keycode = Keycode::S;
const LOAD_KEYCODE: Keycode = Keycode::L;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cpu size: {}", std::mem::size_of::<Cpu>());
    let cartridge = Cartridge::new(ROM)?;
    let mut cpu = Cpu::new(cartridge);
    let mut save_state = cpu.clone();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            format!(
                "Aidan's Big-Brain GB Emulator - Playing {}",
                cpu.bus.cartridge.get_title()
            )
            .as_str(),
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

    let mut debug = false;
    let mut i = 0;

    loop {
        cpu.step(debug);
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

        if i % 1_000 == 0 {
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
                    Event::KeyDown {
                        keycode: Some(DEBUG_KEYCODE),
                        repeat: false,
                        ..
                    } => {
                        println!("debugging!");
                        debug = true;
                    }
                    Event::KeyDown {
                        keycode: Some(SAVE_KEYCODE),
                        repeat: false,
                        ..
                    } => save_state = cpu.clone(),
                    Event::KeyDown {
                        keycode: Some(LOAD_KEYCODE),
                        repeat: false,
                        ..
                    } => cpu = save_state.clone(),

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
                    Event::KeyUp {
                        keycode: Some(DEBUG_KEYCODE),
                        repeat: false,
                        ..
                    } => debug = false,
                    _ => {}
                }
            }
        }

        i += 1;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::cpu::AddressingModeByte;

    use super::*;

    fn test_blaarg_rom_passed(rom: &[u8]) {
        let cartridge = Cartridge::new(rom).unwrap();
        let mut cpu = Cpu::new(cartridge);

        for _ in 0..75_000_000 {
            cpu.step(false);
        }

        let serial_out = cpu.bus.serial.get_data_written();
        println!("result: {}", serial_out);
        assert!(serial_out.contains("Passed"));
    }

    fn test_mooneye_rom_passed(rom: &[u8]) {
        let cartridge = Cartridge::new(rom).unwrap();
        let mut cpu = Cpu::new(cartridge);

        for _ in 0..50_000_000 {
            cpu.step(false);
        }

        assert_eq!(cpu.read_register(cpu::RegisterByte::B), 03);
        assert_eq!(cpu.read_register(cpu::RegisterByte::C), 05);
        assert_eq!(cpu.read_register(cpu::RegisterByte::D), 08);
        assert_eq!(cpu.read_register(cpu::RegisterByte::E), 13);
        assert_eq!(cpu.read_register(cpu::RegisterByte::H), 21);
        assert_eq!(cpu.read_register(cpu::RegisterByte::L), 34);
    }

    #[test]
    fn test_01_special() {
        test_blaarg_rom_passed(include_bytes!("../tests/01_special.gb"));
    }

    #[test]
    fn test_02_interrupts() {
        test_blaarg_rom_passed(include_bytes!("../tests/02_interrupts.gb"));
    }

    #[test]
    fn test_03_sp_hl() {
        test_blaarg_rom_passed(include_bytes!("../tests/03_sp_hl.gb"));
    }

    #[test]
    fn test_04_op_r_imm() {
        test_blaarg_rom_passed(include_bytes!("../tests/04_op_r_imm.gb"));
    }

    #[test]
    fn test_05_op_rp() {
        test_blaarg_rom_passed(include_bytes!("../tests/05_op_rp.gb"));
    }

    #[test]
    fn test_06_ld_r_r() {
        test_blaarg_rom_passed(include_bytes!("../tests/06_ld_r_r.gb"));
    }

    #[test]
    fn test_07_jr_jp_call_ret_rst() {
        test_blaarg_rom_passed(include_bytes!("../tests/07_jr_jp_call_ret_rst.gb"));
    }

    #[test]
    fn test_08_misc_instructions() {
        test_blaarg_rom_passed(include_bytes!("../tests/08_misc_instructions.gb"));
    }

    #[test]
    fn test_09_op_r_r() {
        test_blaarg_rom_passed(include_bytes!("../tests/09_op_r_r.gb"));
    }

    #[test]
    fn test_10_bit_ops() {
        test_blaarg_rom_passed(include_bytes!("../tests/10_bit_ops.gb"));
    }

    #[test]
    fn test_11_op_a_hl() {
        test_blaarg_rom_passed(include_bytes!("../tests/11_op_a_hl.gb"));
    }

    #[test]
    fn test_instr_timing() {
        test_blaarg_rom_passed(include_bytes!("../tests/instr_timing.gb"));
    }

    #[test]
    #[should_panic]
    fn test_interrupt_time() {
        test_blaarg_rom_passed(include_bytes!("../tests/interrupt_time.gb"));
    }

    #[test]
    fn test_mooneye_daa() {
        test_mooneye_rom_passed(include_bytes!("../tests/daa.gb"));
    }

    #[test]
    fn test_mbc1_bits_bank_1() {
        test_mooneye_rom_passed(include_bytes!("../tests/mbc1_bits_bank1.gb"));
    }

    #[test]
    fn test_mbc1_bits_bank_2() {
        test_mooneye_rom_passed(include_bytes!("../tests/mbc1_bits_bank2.gb"));
    }

    #[test]
    fn test_mbc1_bits_mode() {
        test_mooneye_rom_passed(include_bytes!("../tests/mbc1_bits_mode.gb"));
    }

    #[test]
    fn test_mbc1_bits_ramg() {
        test_mooneye_rom_passed(include_bytes!("../tests/mbc1_bits_ramg.gb"));
    }

    #[test]
    fn test_mbc1_ram_64kb() {
        test_mooneye_rom_passed(include_bytes!("../tests/mbc1_ram_64kb.gb"));
    }

    #[test]
    fn test_mbc1_ram_256kb() {
        test_mooneye_rom_passed(include_bytes!("../tests/mbc1_ram_256kb.gb"));
    }

    #[test]
    fn test_mbc1_rom_512kb() {
        test_mooneye_rom_passed(include_bytes!("../tests/mbc1_rom_512kb.gb"));
    }

    #[test]
    fn test_mbc1_rom_1mb() {
        test_mooneye_rom_passed(include_bytes!("../tests/mbc1_rom_1mb.gb"));
    }

    #[test]
    fn test_mbc1_rom_2mb() {
        test_mooneye_rom_passed(include_bytes!("../tests/mbc1_rom_2mb.gb"));
    }

    #[test]
    fn test_mbc1_rom_4mb() {
        test_mooneye_rom_passed(include_bytes!("../tests/mbc1_rom_4mb.gb"));
    }

    #[test]
    fn test_mbc1_rom_8mb() {
        test_mooneye_rom_passed(include_bytes!("../tests/mbc1_rom_8mb.gb"));
    }

    #[test]
    fn test_mbc1_rom_16mb() {
        test_mooneye_rom_passed(include_bytes!("../tests/mbc1_rom_16mb.gb"));
    }

    #[test]
    fn test_div_write() {
        test_mooneye_rom_passed(include_bytes!("../tests/div_write.gb"));
    }

    #[test]
    fn test_rapid_toggle() {
        test_mooneye_rom_passed(include_bytes!("../tests/rapid_toggle.gb"));
    }

    #[test]
    fn test_tim00() {
        test_mooneye_rom_passed(include_bytes!("../tests/tim00.gb"));
    }

    #[test]
    fn test_tim00_div_trigger() {
        test_mooneye_rom_passed(include_bytes!("../tests/tim00_div_trigger.gb"));
    }

    #[test]
    fn test_tim01() {
        test_mooneye_rom_passed(include_bytes!("../tests/tim01.gb"));
    }

    #[test]
    fn test_tim01_div_trigger() {
        test_mooneye_rom_passed(include_bytes!("../tests/tim01_div_trigger.gb"));
    }

    #[test]
    fn test_tim10() {
        test_mooneye_rom_passed(include_bytes!("../tests/tim10.gb"));
    }

    #[test]
    fn test_tim10_div_trigger() {
        test_mooneye_rom_passed(include_bytes!("../tests/tim10_div_trigger.gb"));
    }

    #[test]
    fn test_tim11() {
        test_mooneye_rom_passed(include_bytes!("../tests/tim11.gb"));
    }

    #[test]
    fn test_tim11_div_trigger() {
        test_mooneye_rom_passed(include_bytes!("../tests/tim11_div_trigger.gb"));
    }

    #[test]
    fn test_tima_reload() {
        test_mooneye_rom_passed(include_bytes!("../tests/tima_reload.gb"));
    }

    #[test]
    #[should_panic]
    fn test_tima_write_reloading() {
        test_mooneye_rom_passed(include_bytes!("../tests/tima_write_reloading.gb"));
    }

    #[test]
    #[should_panic]
    fn test_tma_write_reloading() {
        test_mooneye_rom_passed(include_bytes!("../tests/tma_write_reloading.gb"));
    }
}
