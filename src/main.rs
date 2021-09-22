mod cpu;

use cpu::Cpu;

const BINARY: &[u8] = include_bytes!("../03_sp_hl.gb");

fn main() {
    println!("Hello, world!");
    let mut cpu = Cpu::default();
    cpu.memory[..BINARY.len()].copy_from_slice(BINARY);

    loop {
        let decoded = cpu.decode();
        // println!("{:x?}", decoded);
        cpu.execute(decoded);
        // println!("{:#x?}", cpu);
        // println!("-------------------------------------------------------------------");
    }
}
