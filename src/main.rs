mod cpu;

use cpu::Cpu;

const BINARY: &[u8] = include_bytes!("../08_misc_instructions.gb");

fn main() {
    println!("Hello, world!");
    let mut cpu = Cpu::default();
    cpu.memory[..BINARY.len()].copy_from_slice(BINARY);

    for _ in 0..300_000 {
        let decoded = cpu.decode();
        // println!("{:x?}", decoded);
        cpu.execute(decoded);
        // println!("{:#x?}", cpu);
        // println!("-------------------------------------------------------------------");
    }
}
