mod apu;
mod bus;
mod cartridge;
mod cpu;
mod joypad;
mod ppu;
mod samples_queue;
mod serial;
mod timer;

use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::ppu::PaletteColor;
use crate::samples_queue::samples_queue;

use pixels::{wgpu::TextureFormat, PixelsBuilder, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use std::time::{Duration, Instant};

const PPU_WIDTH: u16 = 160;
const PPU_HEIGHT: u16 = 144;
const PPU_SCALE: u16 = 4;

const CLOCK_FREQUENCY: u32 = 4_194_304;
const AUDIO_SAMPLE_FREQUENCY: u32 = 44_100;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        return Err(format!("usage: ./{} <rom_file>", args[0]).into());
    }
    let mut rom = Vec::new();
    File::open(&args[1])?.read_to_end(&mut rom)?;

    println!("cpu size: {}", std::mem::size_of::<Cpu>());
    let cartridge = Cartridge::new(&rom)?;
    let mut cpu = Cpu::new(cartridge);

    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(PPU_WIDTH * PPU_SCALE, PPU_HEIGHT * PPU_SCALE);
        WindowBuilder::new()
            .with_title("Aidan's Gameboy Emulator")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)?
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        PixelsBuilder::new(u32::from(PPU_WIDTH), u32::from(PPU_HEIGHT), surface_texture)
            .enable_vsync(false)
            .texture_format(TextureFormat::Rgba8UnormSrgb)
            .build()?
    };

    let (_stream, stream_handle) = rodio::OutputStream::try_default()?;

    let (samples_input, samples_output) = samples_queue(2, AUDIO_SAMPLE_FREQUENCY);
    stream_handle.play_raw(samples_output)?;

    let emulation_start = Instant::now();
    let mut emulation_steps = 0;
    let mut audio_steps = 0;

    let mut last_fps_calculation = Instant::now();
    let mut frames_since_fps_calculation = 0;
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::MainEventsCleared => {
                if cpu.bus.ppu.get_lcd_ppu_enable() {
                    let ppu_buffer = cpu.bus.ppu.get_buffer();
                    for (pixel_idx, pixel) in pixels.get_frame().chunks_exact_mut(4).enumerate() {
                        let ppu_pixel_x = pixel_idx % usize::from(PPU_WIDTH);
                        let ppu_pixel_y = pixel_idx / usize::from(PPU_WIDTH);
                        let pixel_rgba = match ppu_buffer[ppu_pixel_y][ppu_pixel_x] {
                            PaletteColor::White => [255, 255, 255, 255],
                            PaletteColor::LightGray => [170, 170, 170, 255],
                            PaletteColor::DarkGray => [85, 85, 85, 255],
                            PaletteColor::Black => [0, 0, 0, 255],
                        };
                        pixel.copy_from_slice(&pixel_rgba);
                    }

                    pixels.render().expect("failed to render frame");
                }

                // Run the CPU until we have caught up to the proper step.
                while emulation_start.elapsed()
                    >= Duration::from_nanos(
                        1_000_000_000 * emulation_steps / u64::from(CLOCK_FREQUENCY),
                    )
                {
                    cpu.step();

                    // While number of cycles for which we have played audio is less than the
                    // number of cpu cycles actually run, take another sound sample.
                    //
                    // This while loop should never add two samples inside of a single cpu cycle,
                    // unless the audio sample rate is somehow higher than the cpu frequency.
                    while (audio_steps * u64::from(CLOCK_FREQUENCY)
                        / u64::from(AUDIO_SAMPLE_FREQUENCY))
                        < emulation_steps
                    {
                        samples_input.append(cpu.bus.apu.sample());
                        audio_steps += 1;
                    }

                    emulation_steps += 1;
                }

                frames_since_fps_calculation += 1;

                let time_since_fps_calculation = last_fps_calculation.elapsed();
                if time_since_fps_calculation.as_secs() >= 1 {
                    let fps = 1_000_000_000 * frames_since_fps_calculation
                        / time_since_fps_calculation.as_nanos();
                    window.set_title(format!("FPS: {:03}", fps).as_str());
                    frames_since_fps_calculation = 0;
                    last_fps_calculation = Instant::now();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::DeviceEvent {
                event:
                    DeviceEvent::Key(KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    }),
                ..
            } => {
                let pressed = match state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
                match keycode {
                    VirtualKeyCode::Z => cpu.bus.joypad.set_a_pressed(pressed),
                    VirtualKeyCode::X => cpu.bus.joypad.set_b_pressed(pressed),
                    VirtualKeyCode::RShift => cpu.bus.joypad.set_select_pressed(pressed),
                    VirtualKeyCode::Return => cpu.bus.joypad.set_start_pressed(pressed),
                    VirtualKeyCode::Up => cpu.bus.joypad.set_up_pressed(pressed),
                    VirtualKeyCode::Right => cpu.bus.joypad.set_right_pressed(pressed),
                    VirtualKeyCode::Down => cpu.bus.joypad.set_down_pressed(pressed),
                    VirtualKeyCode::Left => cpu.bus.joypad.set_left_pressed(pressed),
                    _ => {}
                };
            }
            _ => {}
        };
    });
}

#[cfg(test)]
mod tests {
    use super::*;

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
