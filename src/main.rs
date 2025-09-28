extern crate sdl3;

use sdl3::pixels::Color;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::rect::Rect;
use sdl3::mouse::MouseButton;
use std::time::Duration;
use std::thread::sleep;

pub fn main() {
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl3 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut sand_vector: Vec<u8> = Vec::new();

    // Window size
    let win_w = 800;
    let win_h = 600;
    // Square size
    let square_size = 10;

    for _x in 0..(win_w / square_size) {
        for _y in 0..(win_h / square_size) {
            sand_vector.push(0);
        }
    }

    let mut prev_mouse_state = true;

    'running: loop {
        // Background color
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        canvas.clear();

        // Draw a white square in the center
        for x in 0..(win_w / square_size) {
            for y in 0..(win_h / square_size) {
                //let color: u8 = ((x + y) % 255) as u8;

                let color: u8 = (sand_vector[(x + y * (win_w / square_size)) as usize] * 255) as u8;

                canvas.set_draw_color(Color::RGB(color, color, color));
                let square = Rect::new(
                    x * square_size,
                    y * square_size,
                    square_size as u32,
                    square_size as u32,
                );
                let _ = canvas.fill_rect(square);
            }
        }

        // Get mouse state from the event pump
        let mouse_state = event_pump.mouse_state();
        let x = mouse_state.x() as i32 / square_size;
        let y = mouse_state.y() as i32 / square_size;

        if mouse_state.is_mouse_button_pressed(MouseButton::Left) && !prev_mouse_state {
            println!("Mouse left pressed at ({}, {})", x, y);
            sand_vector[(x + y * (win_w / square_size)) as usize] += 1;
            sand_vector[(x + y * (win_w / square_size)) as usize] %= 2;
        }
        prev_mouse_state = mouse_state.is_mouse_button_pressed(MouseButton::Left);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..}
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                _ => {}
            }
        }

        canvas.present();
        sleep(Duration::from_millis(16));
    }
}
