mod cpu;

use cpu::Cpu;

const BINARY: &[u8] = include_bytes!("../ld.gb");

fn main() {
    println!("Hello, world!");
    let mut cpu = Cpu::default();
    cpu.memory[..BINARY.len()].copy_from_slice(BINARY);
    cpu.pc = 0x100;

    for _ in 0.. {
        let decoded = cpu.decode();
        println!("{:#x?}", decoded);
        cpu.execute(decoded);
    }
}
