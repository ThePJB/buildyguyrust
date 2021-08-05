mod game;

use sdl2::pixels::Color;
use sdl2::keyboard::{KeyboardState, Keycode};
use sdl2::event::Event;
use game::GameState;

fn main() {
    let xres = 800;
    let yres = 600;
    let a = xres as f32 / yres as f32;
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("buildy guy rust", xres, yres)
        .position_centered()
        .build()
        .expect("failed making window");

    let mut canvas = window.into_canvas().build()
        .expect("couldnt make canvas");

    let mut event_pump = sdl_context.event_pump().unwrap();

    let gravity = 3.5;
    let cam_vx = 0.4;

    let mut state = GameState::new(a, gravity, cam_vx);
    let dt = 1.0f64 / 60f64;

    'running: loop {
        
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    break 'running;
                }
                Event::KeyDown {keycode: Some(Keycode::R), ..} => {
                    println!("===== reset =====");
                    state = GameState::new(a, gravity, cam_vx);
                }
                _ => {state.handle_input(event)}
            }
        }
        
        canvas.set_draw_color(Color::RGB(200, 200, 255));
        canvas.clear();
        
        state.update_held_keys(&event_pump.keyboard_state());
        state.update(dt);
        state.draw(&mut canvas, xres, yres);

        canvas.present();

        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}
