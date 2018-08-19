extern crate chrono;
extern crate env_logger;
extern crate glfw_window;
extern crate graphics;
extern crate image;
extern crate opengl_graphics;
extern crate percent_encoding;
extern crate piston;
extern crate reqwest;
extern crate serde_json;
extern crate threadpool;
extern crate wallflower;

use chrono::{DateTime, Local};
use piston::event_loop::*;
use piston::input::*;
use piston::window::{Size, Window, WindowSettings};
use graphics::*;
use opengl_graphics::*;
use glfw_window::GlfwWindow;

use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;
use std::env;
use std::path::{Path};

use wallflower::weather::{self, Observation};
use wallflower::flickr;
use wallflower::{slideshow, WallflowerError, statusbar};

const FLICKR_DATA_FILE: &str = ".flickr-data.json";

struct Timer {
    now: DateTime<Local>,
    weather: Option<Observation>,
}

struct Idle {
    time: f64,
    image: Texture,
}

struct Transitioning {
    time: f64,
    image: Texture,
    next_image: Texture,
}

enum State {
    Idle(Idle),
    Transitioning(Transitioning),
}

impl State {
    fn alpha(&self) -> [f32; 4] {
        let alpha = match self {
            State::Idle(_) => 0.,
            State::Transitioning(Transitioning { time, .. }) => *time as f32,
        };

        color::alpha(alpha)
    }

    fn alpha2(&self) -> [f32; 4] {
        let alpha = match self {
            State::Idle(_) => 0.,
            State::Transitioning(Transitioning { time, .. }) => *time as f32,
        };

        color::alpha(1.0 - alpha)
    }
}

fn main() -> Result<(), WallflowerError> {
    env_logger::init();

    let api_key = env::var("FLICKR_API_KEY").expect("FLICKR_API_KEY must be set");
    let api_secret = env::var("FLICKR_API_SECRET").expect("FLICKR_API_SECRET must be set");

    let client = flickr::Client::new(&api_key, &api_secret);
    let client = slideshow::load_access_token(client, FLICKR_DATA_FILE)?;

    // Verify token, and get user info
    let token_info = client.check_token()?;

    println!("{:?}", token_info);

    slideshow::update_photostream(&token_info.user.nsid, &client)?;

    // Load the list of available photos
    let photos = slideshow::available_photos("photos")?;
    if photos.len() == 0 {
        panic!("No photos to show"); // TODO: Make nicer
    }
    let mut photos = photos.iter().cycle();

    // Start graphics
    let opengl = OpenGL::V2_1;
    let mut window: GlfwWindow = WindowSettings::new("Wallflower", [1366, 768])
        .exit_on_esc(true)
        //.fullscreen(true)
        .opengl(opengl)
        .build()
        .unwrap();

    let mut state = State::Idle(Idle {
        time: 0.,
        image: slideshow::load_photo(&photos.next().unwrap())?,
    }); // unwrap should be safe because there are elements in the Vec and cycle means it will never return None

    // Start the time updater thread
    let timer = Arc::new(Mutex::new(Timer {
        now: Local::now(),
        weather: None,
    }));
    let bg_timer = timer.clone();
    let time_update = Duration::from_secs(5);
    thread::spawn(move || loop {
        sleep(time_update);
        {
            let mut timer = bg_timer.lock().unwrap();
            timer.now = Local::now();
        }
    });

    let bg_timer = timer.clone();
    let weather_update = Duration::from_secs(5 * 60);
    let bom = weather::Client::new();
    thread::spawn(move || loop {
        let observation = bom.observations().ok().and_then(statusbar::latest_observation);
        {
            let mut timer = bg_timer.lock().unwrap();
            timer.weather = observation;
        }
        println!("updated weather");
        sleep(weather_update);
    });

    let assets = Path::new("assets");
    let ttf = assets.join("ttf");
    let font = ttf.join("iosevka-ss08-semibold.ttf");
    let mut glyphs = GlyphCache::new(font, (), TextureSettings::new()).expect("error loading font");

    let mut gl = GlGraphics::new(opengl);
    let event_settings = EventSettings {
        max_fps: 24,
        ups: 24,
        ..Default::default()
    };
    let mut events = Events::new(event_settings);

    while let Some(event) = events.next(&mut window) {
        let window_size = window.size();

        if let Some(args) = event.update_args() {
            state = match state {
                State::Idle(mut idle) => {
                    if idle.time > 5. {
                        println!("Transitioning!");
                        State::Transitioning(Transitioning {
                            time: 0.,
                            image: idle.image,
                            next_image: slideshow::load_photo(photos.next().unwrap())
                                .expect("error loading image FIXME"),
                        })
                    } else {
                        idle.time += args.dt;
                        State::Idle(idle)
                    }
                }
                State::Transitioning(mut transitioning) => {
                    if transitioning.time > 1. {
                        println!("Idling!");
                        State::Idle(Idle {
                            time: 0.,
                            image: transitioning.next_image,
                        })
                    } else {
                        transitioning.time += args.dt;
                        State::Transitioning(transitioning)
                    }
                }
            };
        }

        if let Some(press_args) = event.press_args() {
            match press_args {
                Button::Keyboard(Key::Left) => println!("left"),
                Button::Keyboard(Key::Right) => println!("right"),
                Button::Keyboard(Key::Space) => println!("TODO pause"),
                _ => (),
            }
        }

        if let Some(args) = event.render_args() {
            gl.draw(args.viewport(), |context, gfx| {
                clear([0.0; 4], gfx);

                match state {
                    State::Idle(ref idle) => {
                        let (im_width, im_height) = idle.image.get_size();
                        let image_size = Size {
                            width: im_width,
                            height: im_height,
                        };
                        let zoom = slideshow::zoom_for_image(window_size, image_size);
                        // Position in the middle of the view
                        let trans = slideshow::translation_for_image(
                            window_size.width,
                            image_size.width as f64 * zoom,
                        );
                        image(
                            &idle.image,
                            context.transform.trans(trans, 0.).zoom(zoom),
                            gfx,
                        );
                    }
                    State::Transitioning(ref transitioning) => {
                        let (im_width, im_height) = transitioning.image.get_size();
                        let image_size = Size {
                            width: im_width,
                            height: im_height,
                        };
                        let zoom = slideshow::zoom_for_image(window_size, image_size);
                        // Position in the middle of the view
                        let trans = slideshow::translation_for_image(
                            window_size.width,
                            image_size.width as f64 * zoom,
                        );
                        // image(&transitioning.image, context.transform.trans(trans, 0.).zoom(zoom), gfx);
                        Image::new_color(state.alpha2()).draw(
                            &transitioning.image,
                            &Default::default(),
                            context.transform.trans(trans, 0.).zoom(zoom),
                            gfx,
                        );

                        let (im_width, im_height) = transitioning.next_image.get_size();
                        let image_size = Size {
                            width: im_width,
                            height: im_height,
                        };
                        let zoom = slideshow::zoom_for_image(window_size, image_size);
                        // Position in the middle of the view
                        let trans = slideshow::translation_for_image(
                            window_size.width,
                            image_size.width as f64 * zoom,
                        );
                        //image(&photo, context.transform.trans(trans, 0.).zoom(zoom), gfx);
                        Image::new_color(state.alpha()).draw(
                            &transitioning.next_image,
                            &Default::default(),
                            context.transform.trans(trans, 0.).zoom(zoom),
                            gfx,
                        );
                    }
                }

                // Draw status bar
                let (time, weather) = {
                    let timer = timer.lock().unwrap();
                    (
                        timer.now.format("%-I:%M %p"),
                        statusbar::format_observation(&timer.weather),
                    )
                };

                let text_size = 30;
                Rectangle::new([0., 0., 0., 0.75]).draw(
                    [
                        0.,
                        window_size.height as f64 - (text_size as f64 * 2.),
                        window_size.width as f64,
                        text_size as f64 * 2.,
                    ],
                    &context.draw_state,
                    context.transform,
                    gfx,
                );

                let transform = context
                    .transform
                    .trans(10.0, window_size.height as f64 - 20.); // TODO: Centre?
                Text::new_color([1.0, 1.0, 1.0, 0.75], text_size)
                    .draw(
                        &format!("{}     {}", time, weather),
                        &mut glyphs,
                        &context.draw_state,
                        transform,
                        gfx,
                    )
                    .expect("text drawing error");
            });
        }
    }

    Ok(())
}
