mod bus;
mod cartridge;
mod cpu;
mod ppu;
mod serial;
mod timer;

use cpu::Cpu;
use ppu::PaletteColor;

use std::error::Error;

use crate::{bus::Bus, cartridge::Cartridge};

const ROM: &[u8] = include_bytes!("../tetris.gb");
const RENDER: bool = true;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cpu size: {}", std::mem::size_of::<Cpu>());
    let cartridge = Cartridge::new(ROM)?;
    let mut cpu = Cpu::new(cartridge);

    let mut i = 0;
    let mut tick = 0;
    loop {
        //cpu.step(tick > 206_405_000);
        cpu.step(false);

        if RENDER
            && cpu.bus.ppu.should_print()
            && cpu.bus.ppu.get_buffer().iter().any(|line| {
                line.iter()
                    .any(|pixel| !matches!(pixel, PaletteColor::White))
            })
        {
            if i % 50 == 0 {
                print!("\x1Bc"); // clear screen
                for line in cpu.bus.ppu.get_buffer() {
                    for pixel in line {
                        let pixel_data = match pixel {
                            PaletteColor::White => ' ',
                            PaletteColor::LightGray => '-',
                            PaletteColor::DarkGray => '+',
                            PaletteColor::Black => '@',
                        };
                        print!("{}", pixel_data);
                    }
                    println!();
                }
            }

            i += 1;
        }

        tick += 1;
    }

    println!("{}", cpu.bus.serial.get_data_written());
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
