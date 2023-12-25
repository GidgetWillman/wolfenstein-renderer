#[allow(unused)]
mod raycast_render;
pub mod load_vec_buffer;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::Texture;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::image::{self, LoadTexture, InitFlag};
use sdl2::surface::Surface;
use raycast_render::{LevelMap, Camera};
use load_vec_buffer::TexturePixelBuffer;
use std::time::{Duration, Instant};

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem.window("Raycaster", 1280, 960)
        .position_centered()
        .resizable()
        .build()
        .expect("couldn't initialize video subsystem");
    let mut canvas = window.into_canvas().build()
        .expect("could not make a canvas");
    let texture_creator = canvas.texture_creator();
    let _image_context = image::init(InitFlag::PNG | InitFlag::JPG)?;
    let load_texture = |file: &str| -> Result<Texture, String> {
        texture_creator.load_texture(file)
    };

    canvas.set_draw_color(Color::RGB(0,0,0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;
    let mut _i = 0;

    //timer values just to follow the tutorial, replace if we find something better
    let mut frame_time = Instant::now(); //will be the time the current frame took to draw
    let mut prev_frame_time = Instant::now();
    let mut delta_time: Duration;
    let mut time_mod: f32 = 0.0;

    //establish the map for testing purposes
    let mut testing_level_map = LevelMap::new(10, 10);
    let new_grid = vec![ //so the grid still works as x, y with the top being 'north'
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 2, 2, 0, 0, 3, 3, 0, 1],
        vec![1, 0, 2, 0, 0, 0, 0, 3, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 5, 0, 0, 0, 0, 4, 0, 1],
        vec![1, 0, 5, 5, 0, 0, 4, 4, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 2, 3, 4, 5, 5, 4, 3, 2, 1],
    ];
    testing_level_map.map_grid = new_grid;

    //instantiate camera
    let mut camera = Camera::new(
        5.5,
        5.5,
        0.0,
        0.66,
        320,
        240,
        -0.66,
        0.0,
        32,
        32);

    //load images into SDL textures
    //TODO: replace this list with a texture file input later
    let texture_files = [
        "res/DCSS CC0 SPRITES/dungeon/floor/grass/grass_0_old.png",
        "res/DCSS CC0 SPRITES/dungeon/wall/brick_brown_0.png",
        "res/DCSS CC0 SPRITES/dungeon/wall/bars_red_2.png",
        "res/DCSS CC0 SPRITES/dungeon/wall/beehives_0.png",
        "res/DCSS CC0 SPRITES/dungeon/wall/church_0.png",
        "res/DCSS CC0 SPRITES/dungeon/wall/cobalt_stone_1.png",
        "res/DCSS CC0 SPRITES/dungeon/floor/ice_0_old.png",
    ];
    let mut textures: Vec<TexturePixelBuffer> = Vec::new();
    for file in texture_files.iter() {
        let mut texture_buffer = TexturePixelBuffer::new(32, 32);
        texture_buffer.from_texture(file);
        textures.push(texture_buffer);
    }

    //establish a screen surface to quickly write to
    let mut screen_buffer: TexturePixelBuffer = TexturePixelBuffer::new(320,240);
    let mut screen_texture = texture_creator.create_texture_target(
        PixelFormatEnum::RGBA8888,
        320,
        240,
    ).expect("Couldn't Create Texture");

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                    camera.move_player(&time_mod, 10.0);
                },
                Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                    camera.turn_camera(&time_mod, 1, 10.0);
                },
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    camera.move_player(&time_mod, -10.0);
                },
                Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                    camera.turn_camera(&time_mod, -1, 10.0);
                },
                Event::MouseMotion { xrel, ..} => {
                    camera.rotate_camera(&time_mod, -xrel, 1.0);
                },
                _ => {}
            }
        }

        //run the raycast
        camera.raycast(&testing_level_map, &mut canvas, &textures, &mut screen_buffer, &mut screen_texture);

        //calculate frame time
        frame_time = Instant::now();
        delta_time = frame_time.duration_since(prev_frame_time);
        prev_frame_time = frame_time;
        time_mod = delta_time.as_secs_f32();
        let fps = 1.0 / time_mod;
        canvas.window_mut().set_title(&format!("Raycaster - {:.2} FPS", fps)).unwrap();
        //canvas.clear();

        //time management - 60 FPS - commented out for now
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
