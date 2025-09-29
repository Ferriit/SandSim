extern crate sdl3;

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use sdl3::pixels::Color;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::rect::Rect;
use sdl3::mouse::MouseButton;
use std::time::Duration;
use std::thread::sleep;


const MOVED_FLAG: u8 = 0b1000_0000;
const MATERIAL_MASK: u8 = 0b0111_1111;


/// Returns a Vec of 1D indices inside a circle of given radius around a 1D position index
/// `win_cell_w` is the width of the grid
pub fn indices_in_circle(pos_idx: usize, radius: i32, win_cell_w: usize) -> Vec<usize> {
    let mut indices = Vec::new();

    let cx = (pos_idx % win_cell_w) as i32;
    let cy = (pos_idx / win_cell_w) as i32;

    let r2 = radius * radius;

    for dx in -radius..=radius {
        for dy in -radius..=radius {
            if dx*dx + dy*dy <= r2 {
                let x = cx + dx;
                let y = cy + dy;
                if x >= 0 && y >= 0 {
                    indices.push(x as usize + y as usize * win_cell_w);
                }
            }
        }
    }

    indices
}


/// Generates a procedural ice texture using Voronoi noise for "shattered" effect
/// 
/// # Arguments
/// * `width` - width of the texture in tiles
/// * `height` - height of the texture in tiles
/// * `seed` - RNG seed for reproducibility
/// * `cell_count` - number of Voronoi cells (more cells = more cracks)
///
/// # Returns
/// A 2D vector of u8 brightness values (0–255)
pub fn generate_ice_texture(
    width: usize,
    height: usize,
    seed: u64,
    cell_count: usize,
) -> Vec<Vec<u8>> {
    let mut rng = StdRng::seed_from_u64(seed);

    // Generate Voronoi cell centers
    let mut centers: Vec<(f64, f64)> = Vec::new();
    for _ in 0..cell_count {
        let cx = rng.gen_range(0.0..width as f64);
        let cy = rng.gen_range(0.0..height as f64);
        centers.push((cx, cy));
    }

    // Create texture
    let mut texture: Vec<Vec<u8>> = vec![vec![180; height]; width]; // base ice brightness

    for x in 0..width {
        for y in 0..height {
            let mut distances: Vec<f64> = centers
                .iter()
                .map(|&(cx, cy)| {
                    let dx = cx - x as f64;
                    let dy = cy - y as f64;
                    dx*dx + dy*dy // squared distance
                })
                .collect();

            distances.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let d1 = distances[0];
            let d2 = distances[1];

            // Edge detection: if close to cell boundary, make it bright (crack)
            let edge_strength = ((d2 - d1) * 4.0).min(255.0); // scale difference
            let edge_brightness = if edge_strength < 20.0 { 255 } else { 180 + (rng.gen_range(0.0..30.0) as u8) };

            texture[x][y] = edge_brightness;
        }
    }

    texture
}


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
    let ice_texture = generate_ice_texture((win_w / square_size) as usize, (win_h / square_size) as usize, stone_rng.gen_range(0..167837262), 160);

    for x in 0..(win_w / square_size) {
        rock_texture.push(Vec::new());
        for _y in 0..(win_h / square_size) {
            material_vector.push(0);
            rock_texture[x as usize].push((stone_rng.gen_range(0.2..0.5) as f64 * 255.0 as f64).round() as u8);
        }
    }
    material_vector.push(0); // it crashes without this :(
    

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
                     red = (210 - clean_random_offset - rock_texture[x as usize][y as usize] / 2).max(0) as u8;
                     green = (192 - clean_random_offset - rock_texture[x as usize][y as usize] / 2).max(0) as u8;
                     blue = (140 - clean_random_offset - rock_texture[x as usize][y as usize] / 2).max(0) as u8;
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

                else if material_vector[(x + y * win_w / square_size) as usize] == 4 {
                     red = (255 - y - clean_random_offset as i32 / 8).max(0) as u8;
                     green = (128 - y - clean_random_offset as i32 / 8).max(0) as u8;
                     blue = (64 - y - clean_random_offset as i32 / 8).max(0) as u8;
                     if y != 0 {
                        if material_vector[(x + (y - 1) * (win_w / square_size)) as usize] != 4 {
                            let int_red = (red as i32 + 255) / 2;
                            let int_green = (green as i32 + 196) / 2;
                            let int_blue = (blue as i32 + 180) / 2;

                            red = int_red as u8;
                            green = int_green as u8;
                            blue = int_blue as u8;
                        }
                     }
                }

                else if material_vector[(x + y * win_w / square_size) as usize] == 5 {
                    red = 32;
                    green = 32;
                    blue = 32;

                }

                else if material_vector[(x + y * win_w / square_size) as usize] == 6 {
                    red = 128;
                    green = 196;
                    blue = 255;

                    let crack = ice_texture[x as usize][y as usize];

                    if crack > 250 {
                        red = 196;
                        green = 225;
                        blue = 255;
                    }
                }
                else if material_vector[(x + y * win_w / square_size) as usize] == 7 {
                    red = 255;
                    green = 0;
                    blue = 0;
                }
                else if material_vector[(x + y * win_w / square_size) as usize] == 8 {
                    red = 255;
                    green = 255;
                    blue = 255;
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
                let idx = (x + y * (win_w / square_size)) as usize;

                // Skip if already moved this frame
                if (material_vector[idx] & MOVED_FLAG) != 0 {
                    continue;
                }

                let mat = material_vector[idx] & MATERIAL_MASK;

                let width = win_w / square_size;
                let height = win_h / square_size;

                match mat {
                    // === SAND (1) ===
                    1 if y < height - 1 => {
                        let below_idx = (x + (y + 1) * width) as usize;
                        let below_mat = material_vector[below_idx] & MATERIAL_MASK;

                        if below_mat == 0 || below_mat == 2 {
                            material_vector[idx] = if below_mat != 2 { 0 } else { 2 };
                            material_vector[below_idx] = 1 | MOVED_FLAG;
                        } else {
                            let x_isize = x as isize;
                            let y_isize = y as isize;

                            let try_move = |dx: isize| -> Option<usize> {
                                let new_x = x_isize + dx;
                                let new_y = y_isize + 1;
                                if new_x >= 0 && new_x < width as isize && new_y < height as isize {
                                    Some((new_x + new_y * width as isize) as usize)
                                } else {
                                    None
                                }
                            };

                            let mut free_spots = Vec::new();
                            if let Some(dl) = try_move(-1) {
                                let m = material_vector[dl] & MATERIAL_MASK;
                                if m == 0 || m == 2 {
                                    free_spots.push(dl);
                                }
                            }
                            if let Some(dr) = try_move(1) {
                                let m = material_vector[dr] & MATERIAL_MASK;
                                if m == 0 || m == 2 {
                                    free_spots.push(dr);
                                }
                            }

                            if !free_spots.is_empty() {
                                use rand::seq::SliceRandom;
                                let mut rng = rand::thread_rng();
                                let target_idx = *free_spots.choose(&mut rng).unwrap();

                                material_vector[idx] = if (material_vector[target_idx] & MATERIAL_MASK) != 2 { 0 } else { 2 };
                                material_vector[target_idx] = 1 | MOVED_FLAG;
                            }
                        }
                    }

                    // === WATER (2) ===
                    2 if y < height - 1 => {
                        let x_isize = x as isize;
                        let y_isize = y as isize;

                        let try_move = |dx: isize, dy: isize| -> Option<usize> {
                            let new_x = x_isize + dx;
                            let new_y = y_isize + dy;

                            if new_x < 0 || new_y < 0 {
                                return None;
                            }

                            let new_x = new_x as usize;
                            let new_y = new_y as usize;

                            if new_x < width as usize && new_y < height as usize {
                                Some(new_x + new_y * width as usize)
                            } else {
                                None
                            }
                        };

                        // Check for interactions with lava or ice
                        let neighbors = [(0, -1), (0, 1), (-1, 0), (1, 0)];
                        let mut lava_hit: Option<usize> = None;
                        let mut touching_ice = false;

                        for (dx, dy) in neighbors {
                            if let Some(n_idx) = try_move(dx, dy) {
                                match material_vector[n_idx] & MATERIAL_MASK {
                                    4 => { lava_hit = Some(n_idx); break; }
                                    6 => touching_ice = true,
                                    _ => {}
                                }
                            }
                        }

                        if let Some(lava_idx) = lava_hit {
                            material_vector[idx] = 3;
                            material_vector[lava_idx] = 0;
                        } else if touching_ice {
                            material_vector[idx] = 6;
                        } else {
                            let below_idx = idx + width as usize;
                            if (material_vector[below_idx] & MATERIAL_MASK) == 0 {
                                material_vector[idx] = 0;
                                material_vector[below_idx] = 2 | MOVED_FLAG;
                            } else {
                                let mut rng = rand::thread_rng();
                                let directions = if rng.gen_bool(0.5) { [-1, 1] } else { [1, -1] };

                                let mut moved = false;
                                for &dir in &directions {
                                    if let Some(diag) = try_move(dir, 1) {
                                        if (material_vector[diag] & MATERIAL_MASK) == 0 {
                                            material_vector[idx] = 0;
                                            material_vector[diag] = 2 | MOVED_FLAG;
                                            moved = true;
                                            break;
                                        }
                                    }
                                }

                                if !moved {
                                    let mut free_sides = Vec::new();
                                    for &dir in &directions {
                                        if let Some(side) = try_move(dir, 0) {
                                            if (material_vector[side] & MATERIAL_MASK) == 0 {
                                                free_sides.push(side);
                                            }
                                        }
                                    }

                                    if !free_sides.is_empty() {
                                        use rand::seq::SliceRandom;
                                        let target_idx = *free_sides.choose(&mut rng).unwrap();
                                        material_vector[idx] = 0;
                                        material_vector[target_idx] = 2 | MOVED_FLAG;
                                    }
                                }
                            }
                        }
                    }

                    // === STONE (3) ===
                    3 if y < height - 1 => {
                        let below = idx + width as usize;
                        let below_mat = material_vector[below] & MATERIAL_MASK;
                        if below_mat == 0 || below_mat == 2 || below_mat == 4 {
                            material_vector[below] = 3 | MOVED_FLAG;
                            material_vector[idx] = if below_mat == 2 { 2 } else { 0 };
                        }
                    }

                    // === LAVA (4) ===
                    4 if y < height - 1 => {
                        let x_isize = x as isize;
                        let y_isize = y as isize;

                        let try_move = |dx: isize, dy: isize| -> Option<usize> {
                            let new_x = x_isize + dx;
                            let new_y = y_isize + dy;
                            if new_x >= 0 && new_x < width as isize && new_y < height as isize {
                                Some((new_x + new_y * width as isize) as usize)
                            } else {
                                None
                            }
                        };

                        let below_idx = idx + width as usize;
                        let mut rng = rand::thread_rng();
                        let directions = if rng.gen_bool(0.5) { [-1, 1] } else { [1, -1] };

                        let mut moved = false;
                        if (material_vector[below_idx] & MATERIAL_MASK) == 0 {
                            material_vector[idx] = 0;
                            material_vector[below_idx] = 4 | MOVED_FLAG;
                            moved = true;
                        }

                        if !moved {
                            for &dir in &directions {
                                if let Some(diag) = try_move(dir, 1) {
                                    if (material_vector[diag] & MATERIAL_MASK) == 0 {
                                        material_vector[idx] = 0;
                                        material_vector[diag] = 4 | MOVED_FLAG;
                                        moved = true;
                                        break;
                                    }
                                }
                            }
                        }

                        if !moved {
                            let mut free_sides = Vec::new();
                            for &dir in &directions {
                                if let Some(side) = try_move(dir, 0) {
                                    if (material_vector[side] & MATERIAL_MASK) == 0 {
                                        free_sides.push(side);
                                    }
                                }
                            }
                            if !free_sides.is_empty() {
                                use rand::seq::SliceRandom;
                                let target_idx = *free_sides.choose(&mut rng).unwrap();
                                material_vector[idx] = 0;
                                material_vector[target_idx] = 4 | MOVED_FLAG;
                            }
                        }
                    }

                    // === ICE (6) ===
                    6 if y < height - 1 => {
                        let x_isize = x as isize;
                        let y_isize = y as isize;

                        let in_bounds = |nx: isize, ny: isize| -> bool {
                            nx >= 0 && nx < width as isize && ny >= 0 && ny < height as isize
                        };

                        let mut touching_lava = false;
                        for dy in -1..=1 {
                            for dx in -1..=1 {
                                if dx == 0 && dy == 0 { continue; }
                                let nx = x_isize + dx;
                                let ny = y_isize + dy;
                                if in_bounds(nx, ny) {
                                    let n_idx = (nx + ny * width as isize) as usize;
                                    if (material_vector[n_idx] & MATERIAL_MASK) == 4 {
                                        touching_lava = true;
                                        break;
                                    }
                                }
                            }
                            if touching_lava { break; }
                        }

                        if touching_lava {
                            material_vector[idx] = 2;
                        } else {
                            let below_idx = idx + width as usize;
                            let below_mat = material_vector[below_idx] & MATERIAL_MASK;
                            if below_mat == 0 || below_mat == 2 {
                                material_vector[below_idx] = 6 | MOVED_FLAG;
                                material_vector[idx] = if below_mat == 2 { 2 } else { 0 };
                            }
                        }
                    }

                    // === BOMB (7) ===
                    7 => {
                        let damaged_cells = indices_in_circle(idx, 5, width as usize);
                        for i in damaged_cells {
                            if i > 1 && i < (width * height) as usize {
                                material_vector[i] = 0;
                            }
                        }
                    }
                    // === AIRPLANE (8) ===
                    8 => {
                        let cells_per_row = win_w / square_size; // number of horizontal cells

                        // Check if not at right edge
                        if (idx as i32) % cells_per_row != cells_per_row - 1 {
                            let right = idx + 1;
                            if material_vector[right] == 0 {
                                // Move one step right
                                material_vector[right] = 8;
                                material_vector[idx] = 0;
                            } else {
                                // Blocked by non-zero cell → turn into type 7
                                material_vector[idx] = 7;
                            }
                        } else {
                            // Reached the edge → turn into type 7
                            material_vector[idx] = 7;
                        }
                    }

                    _ => {}
                }
            }
        }

        // === End-of-frame cleanup: clear moved flags ===
        for cell in material_vector.iter_mut() {
            *cell &= MATERIAL_MASK;
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
                    selected_material %= 8;
                    if selected_material == -1 {
                        selected_material = 7;
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
            3 => { // Lava
                canvas.set_draw_color(Color::RGB(255, 128, 64));
            },
            4 => { // Steel
                canvas.set_draw_color(Color::RGB(32, 32, 32));
            },
            5 => { // Ice
                canvas.set_draw_color(Color::RGB(128, 196, 255));
            },
            6 => { // Bomb
                canvas.set_draw_color(Color::RGB(255, 0, 0));
            },
            7 => { // Airplane
                canvas.set_draw_color(Color::RGB(255, 255, 255));
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
