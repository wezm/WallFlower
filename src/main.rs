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
use reqwest::Url;
use threadpool::ThreadPool;
use piston::event_loop::*;
use piston::input::*;
use piston::window::{Size, Window, WindowSettings};
use graphics::*;
use opengl_graphics::*;
use glfw_window::GlfwWindow;
use image::DynamicImage;

use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;
use std::borrow::Borrow;
use std::env;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::ffi::OsStr;

use wallflower::flickr::{self, AccessToken, AuthenticatedClient, Photo};
use wallflower::weather::{self, Observation};
use wallflower::WallflowerError;

enum Dimension {
    Width(u32),
    Height(u32),
}

fn download_file(url: &Url, path: &Path) -> Result<(), WallflowerError> {
    let mut file = File::create(path)?;
    // TODO: Check that content type suggests it's actually an image
    // FIXME: reqwest::get creates a new client for each request. Ideally each thread would have its own client and that would be reused for each request that worker serviced
    reqwest::get(url.as_str())?.copy_to(&mut file)?;

    Ok(())
}

fn do_fetch_photo(url: &Url) -> Result<(), WallflowerError> {
    // let path = Path::new("photos");
    let percent_encoded_path = url.path();
    let cow = percent_encoding::percent_decode(percent_encoded_path.as_bytes()).decode_utf8()?;
    let path: &str = cow.borrow();
    let path = Path::new(path);
    let filename = path.file_name().ok_or_else(|| {
        WallflowerError::IoError(io::Error::new(
            io::ErrorKind::Other,
            "URL does not have file name",
        ))
    })?;

    // Check if photo has already been downloaded
    let mut storage_path = PathBuf::new();
    storage_path.push("photos");
    storage_path.push(filename);

    if storage_path.is_file() {
        println!("{} -> exists", url);
        Ok(())
    } else {
        // download the file
        println!("{} -> downloading", url);
        download_file(url, &storage_path)
    }
}

fn fetch_photo(photo: Photo, tx: std::sync::mpsc::Sender<Result<(), WallflowerError>>) {
    tx.send(do_fetch_photo(&photo.url_k))
        .expect("error sending to channel");
}

fn update_photostream(user_id: &str, client: &AuthenticatedClient) -> Result<(), WallflowerError> {
    // Request list of photos from Flickr
    // Download the ones that aren't in the cache
    // (optional) Clean up old images
    // Generate new JSON, move into place atomically

    let pool = ThreadPool::new(8);
    let (tx, rx) = channel();

    // Check the last 500 photos
    // TODO: photos page="2" pages="89" perpage="10" total="881">
    // Stop if there are fewer than 5 pages
    for page in 1..3 {
        let arguments = [
            ("min_taken_date", "1388494800".to_string()),
            ("content_type", "1".to_string()), // Photos only
            ("per_page", "100".to_string()),
            ("page", page.to_string()),
            ("extras", "url_k".to_string()),
        ];
        let photos = client.photos(user_id, &arguments)?;

        //println!("{:?}", photos);

        let photo_count = photos.len();

        for photo in photos {
            let tx = tx.clone();
            pool.execute(move || fetch_photo(photo, tx))
        }

        rx.iter().take(photo_count).for_each(|result| {
            if result.is_err() {
                println!("{:?}", result)
            }
        });
    }

    Ok(())
}

const FLICKR_DATA_FILE: &str = ".flickr-data.json";

fn load_access_token(client: flickr::Client) -> Result<AuthenticatedClient, WallflowerError> {
    match File::open(FLICKR_DATA_FILE) {
        Ok(file) => {
            let access_token: AccessToken = serde_json::from_reader(file)?;
            Ok(AuthenticatedClient::new(client, access_token))
        }
        Err(e) => {
            println!("{:?}", e);
            let client = client.authenticate()?;

            // Save app data for using on the next run.
            let file = File::create(FLICKR_DATA_FILE)?;
            let _ = serde_json::to_writer_pretty(file, client.access_token())?;

            Ok(client)
        }
    }
}

fn largest_dimension(size: Size) -> Dimension {
    if size.width > size.height {
        Dimension::Width(size.width)
    } else {
        Dimension::Height(size.height)
    }
}

fn zoom_for_image(window_size: Size, image_size: Size) -> f64 {
    match largest_dimension(image_size) {
        Dimension::Width(width) => window_size.width as f64 / width as f64,
        Dimension::Height(height) => window_size.height as f64 / height as f64,
    }
}

fn translation_for_image(window_width: u32, image_width: f64) -> f64 {
    (window_width as f64 / 2.) - (image_width / 2.)
}

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

fn load_photo<P: AsRef<Path>>(path: P) -> Result<Texture, WallflowerError> {
    println!("loading {:?}", path.as_ref());

    let photo = image::open(&path).map_err(|_err| {
        println!("{:?}", _err);
        WallflowerError::GraphicsError
    })?;

    let photo = match photo {
        DynamicImage::ImageRgba8(photo) => photo,
        x => x.to_rgba(),
    };

    Ok(Texture::from_image(&photo, &TextureSettings::new()))
}

fn available_photos(dir: &str) -> Result<Vec<PathBuf>, WallflowerError> {
    let mut photos = vec![];
    let jpg = OsStr::new("jpg");

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension()
            .map(|ext| ext == jpg && path.is_file())
            .unwrap_or(false)
        {
            println!("adding {:?}", path);
            photos.push(path);
        }
    }

    Ok(photos)
}

fn latest_observation(observations: Vec<Observation>) -> Option<Observation> {
    observations.into_iter().nth(0)
}

fn format_observation(observation: &Option<Observation>) -> String {
    if let Some(o) = observation {
        format!(
            "{}°C feels like {}°C   Rain since 9am: {}mm   {}% humidity",
            o.air_temp, o.apparent_t, o.rain_trace, o.rel_hum
        )
    } else {
        let default = "--";
        format!(
            "{}°C feels like {}°C   Rain since 9am: {}mm   {}% humidity",
            default, default, default, default
        )
    }
}

fn main() -> Result<(), WallflowerError> {
    env_logger::init();

    let api_key = env::var("FLICKR_API_KEY").expect("FLICKR_API_KEY must be set");
    let api_secret = env::var("FLICKR_API_SECRET").expect("FLICKR_API_SECRET must be set");

    let client = flickr::Client::new(&api_key, &api_secret);
    let client = load_access_token(client)?;

    // Verify token, and get user info
    let token_info = client.check_token()?;

    println!("{:?}", token_info);

    update_photostream(&token_info.user.nsid, &client)?;

    // Load the list of available photos
    let photos = available_photos("photos")?;
    if photos.len() == 0 {
        panic!("No photos to show"); // TODO: Make nicer
    }
    let mut photos = photos.iter().cycle();

    // Start graphics
    let opengl = OpenGL::V2_1;
    let mut window: GlfwWindow = WindowSettings::new("Wallflower", [1920, 1080])
        .exit_on_esc(true)
        //.fullscreen(true)
        .opengl(opengl)
        .build()
        .unwrap();

    let mut state = State::Idle(Idle {
        time: 0.,
        image: load_photo(&photos.next().unwrap())?,
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
        let observation = bom.observations().ok().and_then(latest_observation);
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
                            next_image: load_photo(photos.next().unwrap())
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
                        let zoom = zoom_for_image(window_size, image_size);
                        // Position in the middle of the view
                        let trans = translation_for_image(
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
                        let zoom = zoom_for_image(window_size, image_size);
                        // Position in the middle of the view
                        let trans = translation_for_image(
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
                        let zoom = zoom_for_image(window_size, image_size);
                        // Position in the middle of the view
                        let trans = translation_for_image(
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
                        format_observation(&timer.weather),
                    )
                };

                Rectangle::new([0., 0., 0., 0.75]).draw(
                    [
                        0.,
                        window_size.height as f64 - 80.,
                        window_size.width as f64,
                        100.,
                    ],
                    &context.draw_state,
                    context.transform,
                    gfx,
                );

                let transform = context
                    .transform
                    .trans(10.0, window_size.height as f64 - 20.); // TODO: Centre?
                Text::new_color([1.0, 1.0, 1.0, 0.75], 50)
                    .draw(
                        &format!("{}     {}", time, weather),
                        &mut glyphs,
                        &context.draw_state,
                        transform,
                        gfx,
                    )
                    .expect("weather text drawing error");
            });
        }
    }

    Ok(())
}
