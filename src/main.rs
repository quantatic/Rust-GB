mod cpu;

use cpu::Cpu;

const BINARY: &'static [u8] = include_bytes!("../ld.gb");

fn main() {
    println!("Hello, world!");
    let mut cpu = Cpu::default();
    cpu.memory[..BINARY.len()].copy_from_slice(BINARY);
    cpu.pc = 0x100;

    for i in 0u64.. {
        let decoded = cpu.decode();
        // println!("{:#x?}", decoded);
        cpu.execute(decoded);
        if i % 1_000_000 == 0 {
            println!("step {}", i);
        }
    }
}
