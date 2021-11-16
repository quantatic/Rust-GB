pub mod apu;
pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod joypad;
pub mod ppu;
pub mod serial;
pub mod timer;

pub const PPU_WIDTH: u16 = 160;
pub const PPU_HEIGHT: u16 = 144;
pub const CLOCK_FREQUENCY: u32 = 4_194_304;

#[cfg(test)]
mod tests {
    use super::*;

    use super::cartridge::Cartridge;
    use super::cpu::Cpu;

    fn test_blaarg_rom_passed(rom: &[u8]) {
        let cartridge = Cartridge::new(rom).unwrap();
        let mut cpu = Cpu::new(cartridge);

        for _ in 0..100_000_000 {
            cpu.step();
        }

        let serial_out = cpu.bus.serial.get_data_written();
        println!("result: {}", serial_out);
        assert!(serial_out.contains("Passed"));
    }

    fn test_mooneye_rom_passed(rom: &[u8]) {
        let cartridge = Cartridge::new(rom).unwrap();
        let mut cpu = Cpu::new(cartridge);

        for _ in 0..50_000_000 {
            cpu.step();
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
    fn test_mbc5_rom_512kb() {
        test_mooneye_rom_passed(include_bytes!("../tests/mbc5_rom_512kb.gb"));
    }

    #[test]
    fn test_mbc5_rom_1mb() {
        test_mooneye_rom_passed(include_bytes!("../tests/mbc5_rom_1mb.gb"));
    }

    #[test]
    fn test_mbc5_rom_2mb() {
        test_mooneye_rom_passed(include_bytes!("../tests/mbc5_rom_2mb.gb"));
    }

    #[test]
    fn test_mbc5_rom_4mb() {
        test_mooneye_rom_passed(include_bytes!("../tests/mbc5_rom_4mb.gb"));
    }

    #[test]
    fn test_mbc5_rom_8mb() {
        test_mooneye_rom_passed(include_bytes!("../tests/mbc5_rom_8mb.gb"));
    }

    #[test]
    fn test_mbc5_rom_16mb() {
        test_mooneye_rom_passed(include_bytes!("../tests/mbc5_rom_16mb.gb"));
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
