use emulator_core::{
    cartridge::Cartridge,
    cpu::Cpu,
    joypad::Button,
    ppu::{PPU_HEIGHT, PPU_WIDTH},
};

use pixels::{Pixels, SurfaceTexture};

use wasm_bindgen::JsCast;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::web::WindowExtWebSys,
    window::WindowBuilder,
};

use std::rc::Rc;

const ROM: &[u8] = include_bytes!("../../emulator-core/tests/pocket.gb");

const AUDIO_SAMPLE_FREQUENCY: u32 = 44_100;

pub fn main() {
    console_error_panic_hook::set_once();
    wasm_bindgen_futures::spawn_local(run());
}

async fn run() {
    let event_loop = EventLoop::new();

    let get_window_size = || {
        let client_window = web_sys::window().unwrap();
        LogicalSize::new(
            client_window.inner_width().unwrap().as_f64().unwrap(),
            client_window.inner_height().unwrap().as_f64().unwrap(),
        )
    };

    let window = {
        let size = get_window_size();
        WindowBuilder::new()
            .with_title("A fantastic window!")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let canvas = window.canvas();
    let window = Rc::new(window);

    let html_window = web_sys::window().unwrap();
    let document = html_window.document().unwrap();
    let body = document.body().unwrap();

    body.append_child(&canvas)
        .expect("Append canvas to HTML body");

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
        Pixels::new_async(PPU_WIDTH as u32, PPU_HEIGHT as u32, surface_texture)
            .await
            .expect("Pixels error")
    };

    let closure = {
        let window = Rc::clone(&window);
        wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
            let size = get_window_size();
            window.set_inner_size(size);
        }) as Box<dyn FnMut(_)>)
    };
    html_window
        .add_event_listener_with_callback("resize", closure.as_ref().dyn_ref().unwrap())
        .unwrap();
    closure.forget();

    let closure = {
        wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {}) as Box<dyn Fn(_)>)
    };
    let audio_button = document.get_element_by_id("audio-playback").unwrap();
    audio_button
        .add_event_listener_with_callback("click", closure.as_ref().dyn_ref().unwrap())
        .unwrap();
    closure.forget();

    let mut cpu = Cpu::new(Cartridge::new(ROM).expect("failed to initialize cartridge"));
    let mut i = 0;
    let mut audio_buffer_left = Vec::new();
    let mut audio_buffer_right = Vec::new();
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            window_id,
        } if window_id == window.id() => *control_flow = ControlFlow::Exit,
        Event::MainEventsCleared => {
            for _ in 0..70_224 {
                cpu.fetch_decode_execute();
                if i % 95 == 0 {
                    let [sample_left, sample_right] = cpu.bus.apu.sample();
                    audio_buffer_left.push(sample_left);
                    audio_buffer_right.push(sample_right);
                    if audio_buffer_left.len() == usize::try_from(AUDIO_SAMPLE_FREQUENCY).unwrap() {
                        let mut options = web_sys::AudioBufferOptions::new(AUDIO_SAMPLE_FREQUENCY, AUDIO_SAMPLE_FREQUENCY as f32);
                        options.number_of_channels(2);
                        let buffer = web_sys::AudioBuffer::new(&options).unwrap();

                        buffer.copy_to_channel(&audio_buffer_left, 0).unwrap();
                        buffer.copy_to_channel(&audio_buffer_right, 1).unwrap();

                        let context = web_sys::AudioContext::new().unwrap();
                        let source_node = web_sys::AudioBufferSourceNode::new(&context).unwrap();
                        source_node.set_buffer(Some(&buffer));
                        source_node.connect_with_audio_node(&context.destination()).unwrap();
                        source_node.start().unwrap();

                        audio_buffer_left.clear();
                        audio_buffer_right.clear();
                    }
                }
                i += 1;
            }

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
        }
        Event::WindowEvent {
            event:
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(keycode),
                            ..
                        },
                    ..
                },
            window_id,
        } if window_id == window.id() => {
            web_sys::console::log_1(&format!("{:?}, {:?}", state, keycode).into());
            let pressed = match state {
                ElementState::Pressed => true,
                ElementState::Released => false,
            };
            match keycode {
                VirtualKeyCode::Z => cpu.set_button_pressed(Button::B, pressed),
                VirtualKeyCode::X => cpu.set_button_pressed(Button::A, pressed),
                VirtualKeyCode::RShift => cpu.set_button_pressed(Button::Select, pressed),
                VirtualKeyCode::Return => cpu.set_button_pressed(Button::Start, pressed),
                VirtualKeyCode::Up => cpu.set_button_pressed(Button::Up, pressed),
                VirtualKeyCode::Right => cpu.set_button_pressed(Button::Right, pressed),
                VirtualKeyCode::Down => cpu.set_button_pressed(Button::Down, pressed),
                VirtualKeyCode::Left => cpu.set_button_pressed(Button::Left, pressed),
                VirtualKeyCode::H if pressed => web_sys::console::log_1(
                    &format!(
                        "current checksum: 0x{:08X}",
                        emulator_core::calculate_ppu_buffer_checksum(&cpu)
                    )
                    .into(),
                ),
                _ => {}
            };
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(new_size),
            window_id,
        } if window_id == window.id() => pixels.resize_surface(new_size.width, new_size.height),
        _ => (),
    });
}
