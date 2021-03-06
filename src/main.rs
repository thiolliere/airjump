extern crate fps_clock;
#[macro_use] extern crate glium;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate stdweb;

mod spatial_hashing;
mod configuration;
pub mod math;
#[cfg(target_os= "emscripten")]
#[path="dummy_audio.rs"]
mod audio;
#[cfg(not(target_os= "emscripten"))]
mod audio;
mod app;
mod map;
mod physics;
pub mod graphics;
#[cfg(target_os = "emscripten")]
pub mod emscripten;

use configuration::CFG;
use stdweb::unstable::TryInto;
use glium::glutin;

pub trait OkOrExit {
    type OkType;
    fn ok_or_exit(self) -> Self::OkType;
}
impl<T, E: ::std::fmt::Display> OkOrExit for Result<T,E> {
    type OkType = T;
    fn ok_or_exit(self) -> T {
        match self {
            Ok(t) => t,
            Err(err) => {
                println!("ERROR: {}", err);
                ::std::process::exit(1);
            },
        }
    }
}

fn main() {
    safe_main().ok_or_exit();
}

fn safe_main() -> Result<(), String> {
    let mut events_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_multitouch()
        .with_title("airjump");

    let mut context_builder = glutin::ContextBuilder::new();

    context_builder = context_builder.with_vsync(CFG.window.vsync);
    context_builder = context_builder.with_multisampling(CFG.window.samples as u16);

    let window = glium::Display::new(window_builder, context_builder, &events_loop)
        .map_err(|e| format!("build glium: {}", e))?;

    let mut graphics = graphics::Graphics::new(&window).map_err(|e| format!("graphics: {}", e))?;

    let audio = audio::Audio::new().map_err(|e| format!("audio: {}", e))?;

    let mut app = app::App::new(audio);

    let mut last_set_inner_size = (0, 0);
    // return whereas main loop breaks
    set_main_loop(|dt| -> bool {
        {
            let w = js!{ return window.innerWidth; }.try_into().unwrap();
            let h = js!{ return window.innerHeight; }.try_into().unwrap();
            if last_set_inner_size != (w, h) {
                last_set_inner_size = (w, h);
                window.gl_window().set_inner_size(w, h);
            }
        }
        events_loop.poll_events(|event| {
            use glium::glutin::Event::*;
            use glium::glutin::WindowEvent::*;
            use glium::glutin::TouchPhase;
            match event {
                WindowEvent { event: Closed, .. } => app.must_quit = true,
                WindowEvent { event: Touch(touch), .. } => {
                    if touch.phase == TouchPhase::Started {
                        let (w, h) = window.gl_window().get_inner_size().unwrap();
                        let x = touch.location.0 - (w/2) as f64;
                        let y = - (touch.location.1 - (h/2) as f64);
                        app.set_jump_angle(y.atan2(x) + ::std::f64::consts::PI);
                        app.do_jump();
                    }
                },
                WindowEvent { event: Refresh, .. } => {
                    let mut target = window.draw();
                    {
                        let camera = app.camera();
                        let mut frame = graphics::Frame::new(&mut graphics, &mut target, &camera);
                        frame.clear();
                        app.draw(&mut frame);
                    }
                    target.finish().unwrap();
                }
                _ => (),
            }
        });

        app.update(dt);

        let mut target = window.draw();
        {
            let camera = app.camera();
            let mut frame = graphics::Frame::new(&mut graphics, &mut target, &camera);
            frame.clear();
            app.draw(&mut frame);
        }
        target.finish().unwrap();

        return app.must_quit
    });

    Ok(())
}

#[cfg(target_os = "emscripten")]
fn set_main_loop<F: FnMut(f64) -> bool>(mut main_loop: F) {
    let dt = 1.0 / 60f64;
    emscripten::set_main_loop_callback(|| {
        if main_loop(dt) {
            emscripten::cancel_main_loop();
        }
    });
}

// behavior differ from emscripten as it doesn't return
// as long as the main loop doesn't end
#[cfg(all(not(target_os = "emscripten")))]
fn set_main_loop<F: FnMut(f64) -> bool>(mut main_loop: F) {
    // If running out of time then slow down the game
    let mut fps_clock = fps_clock::FpsClock::new(CFG.event_loop.max_fps);
    let dt = 1.0 / CFG.event_loop.max_fps as f64;
    loop {
        if main_loop(dt) {
            break
        }
        fps_clock.tick();
    }
}
