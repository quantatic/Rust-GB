mod samples_queue;

use crate::samples_queue::samples_queue;

use emulator_core::cartridge::Cartridge;
use emulator_core::cpu::Cpu;
use emulator_core::joypad::Button;

use pixels::{wgpu::TextureFormat, PixelsBuilder, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use std::error::Error;
use std::fs::File;
use std::io::Read;
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

                        let pixel_color = ppu_buffer[ppu_pixel_y][ppu_pixel_x];
                        let pixel_red = (pixel_color.red << 3) | (pixel_color.red >> 2);
                        let pixel_green = (pixel_color.green << 3) | (pixel_color.green >> 2);
                        let pixel_blue = (pixel_color.blue << 3) | (pixel_color.blue >> 2);

                        let pixel_rgba = [pixel_red, pixel_green, pixel_blue, 255];
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
                    VirtualKeyCode::Z => cpu.set_button_pressed(Button::A, pressed),
                    VirtualKeyCode::X => cpu.set_button_pressed(Button::B, pressed),
                    VirtualKeyCode::RShift => cpu.set_button_pressed(Button::Select, pressed),
                    VirtualKeyCode::Return => cpu.set_button_pressed(Button::Start, pressed),
                    VirtualKeyCode::Up => cpu.set_button_pressed(Button::Up, pressed),
                    VirtualKeyCode::Right => cpu.set_button_pressed(Button::Right, pressed),
                    VirtualKeyCode::Down => cpu.set_button_pressed(Button::Down, pressed),
                    VirtualKeyCode::Left => cpu.set_button_pressed(Button::Left, pressed),
                    _ => {}
                };
            }
            _ => {}
        };
    });
}
