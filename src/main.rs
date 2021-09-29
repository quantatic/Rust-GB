mod cpu;
mod mmu;
mod serial;
mod timer;

use cpu::Cpu;

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rom_passed(rom: &[u8]) {
        let mut cpu = Cpu::default();
        cpu.mmu.memory[..rom.len()].copy_from_slice(rom);

        for _ in 0..100_000_000 {
            cpu.step();
            cpu.mmu.timer.step();
        }

        let serial_out = cpu.mmu.serial.get_data_written();
        assert!(serial_out.contains("Passed"));
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
