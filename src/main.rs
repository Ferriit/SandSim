extern crate sdl3;

use rand::Rng;
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

    let window = video_subsystem.window("SandSim", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut material_vector: Vec<u8> = Vec::new();

    let mut selected_material: i8 = 0;

    // Window size
    let win_w: i32 = 800;
    let win_h: i32 = 600;
    // Square size
    let square_size: i32 = 10;

    let mut stone_rng = rand::thread_rng();

    let mut rock_texture: Vec<Vec<u8>> = Vec::new();

    for x in 0..(win_w / square_size) {
        rock_texture.push(Vec::new());
        for _y in 0..(win_h / square_size) {
            material_vector.push(0);
            rock_texture[x as usize].push((stone_rng.gen_range(0.2..0.5) as f64 * 255.0 as f64).round() as u8);
        }
    }
    material_vector.push(0); // it crashes without this :(

//    let mut prev_mouse_state = true;
//    let mut prev_mouse_x: i32 = -1;
//    let mut prev_mouse_y: i32 = -1;

    let mut frame_count = 0;

    'running: loop {
        let prioritize_left = frame_count % 2 == 0;
        let directions = if prioritize_left { [-1, 1] } else { [1, -1] };
        // Background color
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        canvas.clear();

        for x in 0..(win_w / square_size) {
            for y in 0..(win_h / square_size) {
                //let color: u8 = ((x + y) % 255) as u8;

                // Checkerboard pattern
                let clean_random_offset = (255 - (((x + y) % 2 + 1) * 215).min(255)) as u8;

                let mut red: u8 = 0;
                let mut green: u8 = 0;
                let mut blue: u8 = 0;

                if material_vector[(x + y * (win_w / square_size)) as usize] == 1 {
                     red = (210 - clean_random_offset).max(0) as u8;
                     green = (192 - clean_random_offset).max(0) as u8;
                     blue = (140 - clean_random_offset).max(0) as u8;
                }
                else if material_vector[(x + y * (win_w / square_size)) as usize] == 2 {
                     red = (64 - y - clean_random_offset as i32 / 8).max(0) as u8;
                     green = (128 - y - clean_random_offset as i32 / 8).max(0) as u8;
                     blue = (255 - y - clean_random_offset as i32 / 8).max(0) as u8;
                     if y != 0 {
                        if material_vector[(x + (y - 1) * (win_w / square_size)) as usize] != 2 {
                            let int_red = (red as i32 + 196) / 2;
                            let int_green = (green as i32 + 196) / 2;
                            let int_blue = (blue as i32 + 255) / 2;

                            red = int_red as u8;
                            green = int_green as u8;
                            blue = int_blue as u8;
                        }
                     }
                }
                else if material_vector[(x + y * win_w / square_size) as usize] == 3 {
                    red = rock_texture[x as usize][y as usize];
                    green = rock_texture[x as usize][y as usize];
                    blue = rock_texture[x as usize][y as usize];
                }

                canvas.set_draw_color(Color::RGB(red, green, blue));
                let square = Rect::new(
                    x * square_size,
                    y * square_size,
                    square_size as u32,
                    square_size as u32,
                );
                let _ = canvas.fill_rect(square);
            }
        }

        for x in (0..(win_w / square_size)).rev() {
            for y in (0..(win_h / square_size)).rev() {
                if material_vector[(x + y * (win_w / square_size)) as usize] == 1
                    && y < (win_h / square_size - 1)
                {
                    let idx = (x + y * (win_w / square_size)) as usize;
                    let below_idx = (x + (y + 1) * (win_w / square_size)) as usize;

                    // Step down if empty or water
                    if material_vector[below_idx] == 0 || material_vector[below_idx] == 2 {
                        material_vector[idx] = if material_vector[below_idx] != 2 { 0 } else { 2 };
                        material_vector[below_idx] = 1;
                    } else {
                        // Randomly decide whether to try left or right first
                        let mut rng = rand::thread_rng();
                        let try_right_first = rng.gen_bool(0.5);

                        let width = win_w / square_size;
                        let height = win_h / square_size;

                        let x_isize = x as isize;
                        let y_isize = y as isize;

                        let try_move = |dx: isize| -> Option<usize> {
                            let new_x = x_isize + dx;
                            let new_y = y_isize + 1;
                            if new_x >= 0 && new_x < width as isize && new_y >= 0 && new_y < height as isize {
                                Some((new_x + new_y * width as isize) as usize)
                            } else {
                                None
                            }
                        };

                        let directions = if try_right_first { [1, -1] } else { [-1, 1] };

                        for &dir in &directions {
                            if let Some(target_idx) = try_move(dir) {
                                if material_vector[target_idx] == 0 || material_vector[target_idx] == 2 {
                                    material_vector[idx] = if material_vector[target_idx] != 2 { 0 } else { 2 };
                                    material_vector[target_idx] = 1;
                                    break;
                                }
                            }
                        }
                    }
                }
                else if material_vector[(x + y * (win_w / square_size)) as usize] == 2 && y < (win_h / square_size - 1) {
                    let idx: usize = (x + y * (win_w / square_size)) as usize;
                    let below_idx = idx + (win_w / square_size) as usize;

                    if material_vector[below_idx] == 0 {
                        // Move down if empty
                        material_vector[idx] = 0;
                        material_vector[below_idx] = 2;
                    } else {
                        let width = win_w / square_size;
                        let height = win_h / square_size;
                        let x_isize = x as isize;
                        let y_isize = y as isize;

                        let try_move = |dx: isize, dy: isize| -> Option<usize> {
                            let new_x = x_isize + dx;
                            let new_y = y_isize + dy;
                            if new_x >= 0 && new_x < width as isize && new_y >= 0 && new_y < height as isize {
                                Some((new_x + new_y * width as isize) as usize)
                            } else {
                                None
                            }
                        };

                        // Alternate direction priority each frame
                        let prioritize_left = frame_count % 2 == 0;
                        let directions = if prioritize_left { [-1, 1] } else { [1, -1] };

                        // Try diagonal down first
                        let mut moved = false;
                        for &dir in &directions {
                            if let Some(diag) = try_move(dir, 1) {
                                if material_vector[diag] == 0 {
                                    material_vector[idx] = 0;
                                    material_vector[diag] = 2;
                                    moved = true;
                                    break;
                                }
                            }
                        }

                        // If diagonals failed, try horizontal spread
                        if !moved {
                            for &dir in &directions {
                                if let Some(side) = try_move(dir, 0) {
                                    if material_vector[side] == 0 {
                                        material_vector[idx] = 0;
                                        material_vector[side] = 2;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                else if material_vector[(x + y * (win_w / square_size)) as usize] == 3 && y < (win_h / square_size - 1) {
                    let idx: usize = (x + y * (win_w / square_size)) as usize;
                    let below = idx + (win_w / square_size) as usize;

                    // Move straight down if empty or water
                    if material_vector[below] == 0 || material_vector[below] == 2 {
                        material_vector[below] = 3;
                        material_vector[idx] = if material_vector[below] == 2 { 2 } else { 0 };
                    }
                }
            }
        }

        // Get mouse state from the event pump
        let mouse_state = event_pump.mouse_state();
        let mut x = mouse_state.x() as i32 / square_size;
        let mut y = mouse_state.y() as i32 / square_size;

        x = x.min(win_w / square_size - 1).max(0);
        y = y.min(win_h / square_size - 1).max(0);

        if mouse_state.is_mouse_button_pressed(MouseButton::Left) {//&& (!prev_mouse_state || x != prev_mouse_x || y != prev_mouse_y) {
            material_vector[(x + y * (win_w / square_size)) as usize] = selected_material as u8 + 1;
        }

        if mouse_state.is_mouse_button_pressed(MouseButton::Right) {
            material_vector[(x + y * (win_w / square_size)) as usize] = 0;
        }

        //prev_mouse_state = mouse_state.is_mouse_button_pressed(MouseButton::Left);
        //prev_mouse_x = x;
        //prev_mouse_y = y;

        //if frame_count % 1 == 0 {
        //    prev_mouse_y = -1;
        //}

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..}
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                Event::MouseWheel { x, y, .. } => {
                    selected_material += y as i8;
                    selected_material %= 3;
                    if selected_material == -1 {
                        selected_material = 2;
                    }
                }
                _ => {}
            }
        }


        match selected_material {
            0 => { // Sand
                canvas.set_draw_color(Color::RGB(210, 192, 140));
            },
            1 => { // Water
                canvas.set_draw_color(Color::RGB(34, 98, 225));
            },
            2 => { // Rock
                canvas.set_draw_color(Color::RGB(64, 64, 64));
            },

            _ => {}
        }

        let square = Rect::new(10, 10, 40, 40);
        let _ = canvas.fill_rect(square);

        canvas.present();

        frame_count += 1;
        sleep(Duration::from_millis(((1.0 / 30.0) * 1000.0) as u64));
    }
}
