mod cpu;
mod mmu;
mod timer;

use cpu::Cpu;

const TEST_ROMS: [&[u8]; 10] = [
    include_bytes!("../02_interrupts.gb"),
    include_bytes!("../03_sp_hl.gb"),
    include_bytes!("../04_op_r_imm.gb"),
    include_bytes!("../05_op_rp.gb"),
    include_bytes!("../06_ld_r_r.gb"),
    include_bytes!("../07_jr_jp_call_ret_rst.gb"),
    include_bytes!("../08_misc_instructions.gb"),
    include_bytes!("../09_op_r_r.gb"),
    include_bytes!("../10_bit_ops.gb"),
    include_bytes!("../11_op_a_hl.gb"),
];

fn main() {
    for rom_data in TEST_ROMS {
        let mut cpu = Cpu::default();
        cpu.mmu.memory[..rom_data.len()].copy_from_slice(rom_data);

        for _ in 0..100_000_000 {
            cpu.step();
            cpu.mmu.timer.step();
        }
        println!("-------------------------");
    }
}
