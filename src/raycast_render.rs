//we'll initialize with a wolfenstein style renderer and expand outward from there
//we'll do integers once we're onto a scanline rasterizer, for now use 32 bit floats where needed
//why 32 bit?  Eh, why not.  Maybe it'll run on old-ass hardware, maybe it'll add some jank.

use sdl2::render::{Canvas, TextureCreator, Texture};
use sdl2::video::Window;
use sdl2::rect::Point;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::image::{self, LoadSurface, InitFlag};
use sdl2::surface::Surface;
use crate::load_vec_buffer::TexturePixelBuffer;


//declare some SDL2 RGB
static RGB_RED: Color = Color::RGB(255, 0, 0);
static RGB_BLUE: Color = Color::RGB(0, 0, 255);
static RGB_GREEN: Color = Color::RGB(0, 255, 0);
static RGB_YELLOW: Color = Color::RGB(255, 255, 0);
static RGB_WHITE: Color = Color::RGB(255, 255, 255);

pub struct Camera {
    pos_x: f32,
    pos_y: f32,
    plane_x: f32,
    plane_y: f32,
    pub screen_width: u32,
    pub screen_height: u32,
    dir_x: f32,
    dir_y: f32,
    pub tex_width: usize,
    pub tex_height: usize,
    pub pixel_buffer: Vec<Vec<u32>>,
}

impl Camera {
    pub fn new(posx: f32, posy: f32, planex: f32, planey: f32, width: u32, height: u32, dirx: f32, diry: f32, texwidth: usize, texheight: usize) -> Self {
        Camera {
            pos_x: posx, //camera position in world
            pos_y: posy,
            plane_x: planex, //plane edge position for raycasting
            plane_y: planey,
            screen_width: width, //screen information for raycast calculations
            screen_height: height,
            dir_x: dirx, //the initial direction vector
            dir_y: diry,
            tex_width: texwidth, //we may need to move texture size out of camera but we'll keep it for now
            tex_height: texheight,
            pixel_buffer: vec![vec![0; height as usize]; width as usize] //we fill this in before writing to the screen
            //pixel_buffer goes y, x because it works per scanline and we're scanning left to right, so height then width
        }
    }
    //raycast function takes a reference to self, it should probably take a reference to the SDL renderer too
    pub fn raycast(self: &mut Self, level: &LevelMap, canvas: &mut Canvas<Window>, textures: &Vec<TexturePixelBuffer>, screen: &mut TexturePixelBuffer, screen_texture: &mut Texture) {
        let mut cam_x: f32 = 0.0; //x position on the screen line
        let mut ray_x: f32 = 0.0; //vector we're casting the ray through
        let mut ray_y: f32 = 0.0;
        let mut grid_x: i32 = self.pos_x as i32; //which cell of the grid we're in
        let mut grid_y: i32 = self.pos_y as i32;
        let mut side_x: f32 = 0.0; //distance to the side of the grid cell we're in
        let mut side_y: f32 = 0.0;
        let mut delta_x: f32 = 0.0; //distance to the next X or Y grid line
        let mut delta_y: f32 = 0.0;
        let mut step_x: i8 = 0; //step direction, either +1 or -1 for either one
        let mut step_y: i8 = 0;
        let mut perp_wall_dist: f32 = 0.0; //distance to wall (may be perpendicular, not sure)
        let mut hit: bool = false; //has there been a hit in the raycast?
        let mut side: i8 = 0; //value of north/south or east/west wall
        let mut line_height: i32 = 0; //height of line to draw on the screen in flat color renderer
        let mut draw_start: i32 = 0; //Y positions to draw the line
        let mut draw_end: i32 = 0;
        let mut current_color: Color = RGB_RED; //color for drawing
        let mut floor_tex: usize = 0;
        let mut ceiling_tex: usize = 6; //set these directly for now

        let ray_x_0 = self.dir_x - self.plane_x;
        let ray_y_0 = self.dir_y - self.plane_y;
        let ray_x_1 = self.dir_x + self.plane_x;
        let ray_y_1 = self.dir_y + self.plane_y;
        let mut sch = self.screen_height;
        let mut scw = self.screen_width;

        //draw floor and ceiling
        for y in 0..self.screen_height
        {
            //current y position relative to screen center
            let screen_pos_y: i32 = y as i32 - self.screen_height as i32 / 2;

            //vertical position of camera
            let pos_z: f32 = 0.5 * self.screen_height as f32;

            //distance from camera to floor in current row;
            let row_distance = pos_z / screen_pos_y as f32;

            //calculate real world step vector to add for each x parallel to camera plane
            //avoids multiplications with weight in inner loop
            let floor_step_x: f32 = row_distance * (ray_x_1 - ray_x_0) / self.screen_width as f32;
            let floor_step_y: f32 = row_distance * (ray_y_1 - ray_y_0) / self.screen_width as f32;

            //real world coordinates of leftmost column, updated as we step to the right
            let mut floor_x = self.pos_x + row_distance * ray_x_0;
            let mut floor_y = self.pos_y + row_distance * ray_y_0;

            for x in 0..self.screen_width
            {
                //cell coordinate is just integer parts of floor_x and floor_y
                let cell_x = floor_x as i32;
                let cell_y = floor_y as i32;

                //get texture coordinate from fractional part
                let fractional_x = floor_x - floor_x.floor();
                let fractional_y = floor_y - floor_y.floor();

                let tex_x = ((self.tex_width as f32 * fractional_x) as usize) % self.tex_width;
                let tex_y = ((self.tex_height as f32 * fractional_y) as usize) % self.tex_height;

                floor_x += floor_step_x;
                floor_y += floor_step_y;

                screen.pixel_buffer[y as usize][x as usize] = textures[floor_tex].pixel_buffer[tex_y][tex_x];
                screen.pixel_buffer[(sch - y - 1) as usize][x as usize] = textures[ceiling_tex].pixel_buffer[tex_y][tex_x];
            }
        }

        //calculate ray position and direction
        for screen_x in 0..self.screen_width
        {
            cam_x = 2.0 * (screen_x as f32) / (self.screen_width as f32) - 1.0; //x coordinate in camera space
            ray_x = self.dir_x + self.plane_x * cam_x;
            ray_y = self.dir_y + self.plane_y * cam_x;
            delta_x = (1.0/ray_x).abs(); //length of ray from one x or y side to the next
            delta_y = (1.0/ray_y).abs();
            grid_x = self.pos_x as i32;
            grid_y = self.pos_y as i32;
            hit = false;
            side = 0;

            //calculate step and initial side distance
            if ray_x < 0.0
            {
                step_x = -1;
                side_x = (self.pos_x - grid_x as f32) * delta_x;
            }
            else
            {
                step_x = 1;
                side_x = (grid_x as f32 + 1.0 - self.pos_x) * delta_x;
            }
            if ray_y < 0.0
            {
                step_y = -1;
                side_y = (self.pos_y - grid_y as f32) * delta_y;
            }
            else
            {
                step_y = 1;
                side_y = (grid_y as f32 + 1.0 - self.pos_y) * delta_y;
            }

            //perform DDA/ ray cast
            while !hit
            {
                //jump to next map square either in x or y
                if side_x < side_y
                {
                    side_x += delta_x;
                    grid_x += step_x as i32;
                    side = 0;
                }
                else
                {
                    side_y += delta_y;
                    grid_y += step_y as i32;
                    side = 1;
                }
                //check for hit
                if level.map_grid[grid_x as usize][grid_y as usize] > 0
                {
                    hit = true;
                }
            }

            //calculate distance based on camera direction
            if side == 0 {
                perp_wall_dist = (grid_x as f32 - self.pos_x + (1.0 - step_x as f32) / 2.0) / ray_x;
            } else {
                perp_wall_dist = (grid_y as f32 - self.pos_y + (1.0 - step_y as f32) / 2.0) / ray_y;
            }

            //calculate height of the line to draw on screen
            line_height = (self.screen_height as f32 / perp_wall_dist) as i32;

            //calculate lowest and heighest pixel to fill (commenting out offscreen ones for now)
            //if framerate somehow becomes an issue we can look into clamping them again
            draw_start = -line_height / 2 + self.screen_height as i32 / 2;
            let tex_start = draw_start;
            if draw_start < 0 { draw_start = 0; }
            draw_end = line_height / 2 + self.screen_height as i32 / 2;
            let tex_end = draw_end;
            if draw_end >= self.screen_height as i32 { draw_end = self.screen_height as i32 - 1; }

            //texturing calculation
            let mut tex_num = level.map_grid[grid_x as usize][grid_y as usize];

            //calculate exactly where the wall was hit
            let mut wall_x: f32;
            if side == 0 { wall_x = self.pos_y + perp_wall_dist * ray_y;}
            else { wall_x = self.pos_x + perp_wall_dist * ray_x}
            wall_x -= wall_x.floor(); //normalize wall_x

            //x coordinate on the texture
            let mut tex_x: u32 = (wall_x * 32 as f32).floor() as u32; //just using 32 for texture width and height everywhere right now
            if side == 0 && ray_x > 0.0 { tex_x = 32 - tex_x - 1; }
            if side == 1 && ray_y < 0.0 { tex_x = 32 - tex_x - 1; }

            //texture coordinate step per screen pixel
            let tex_step = 1.0 * 32.0 / line_height as f32;

            let texture = &textures[tex_num as usize];

            //TODO: fix this code to use the vec buffers
            for screen_y in draw_start..draw_end {
                let tex_y = (((screen_y - tex_start) as f32 / (tex_end - tex_start) as f32) * self.tex_height as f32).floor() as usize;
                screen.pixel_buffer[screen_y as usize][screen_x as usize] = texture.pixel_buffer[(tex_y) as usize][tex_x as usize];
            }
        }
        let buffer_width = &screen.pixel_buffer[1].len();

        let mut flat_pixel_data = Vec::new();
        for row in &screen.pixel_buffer {
            for &pixel in row {
                flat_pixel_data.push((pixel>>24) as u8);
                flat_pixel_data.push((pixel>>16) as u8);
                flat_pixel_data.push((pixel>>8) as u8);
                flat_pixel_data.push((pixel) as u8);
            }
        }
        screen_texture.update(None, &flat_pixel_data, buffer_width * 4);
        canvas.clear();
        canvas.copy(&screen_texture, None, None).unwrap();
        canvas.present();
    }

    pub fn move_player(self: &mut Self, time_mod: &f32, speed: f32) {
        //TODO: once we stabilize the framerate and get around to polishing,
        //it's probably a good idea to dig into why we seem to move faster at lower framerates.
        self.pos_x += self.dir_x * (time_mod * speed);
        self.pos_y += self.dir_y * (time_mod * speed);
    }

    pub fn turn_camera(self: &mut Self, time_mod: &f32, direction: i8, turn_speed: f32) {
        let turn_angle = turn_speed * direction as f32 * time_mod;
        let old_dir_x = self.dir_x;
        let old_dir_y = self.dir_y;
        let old_plane_x = self.plane_x;
        let old_plane_y = self.plane_y;

        self.dir_x = old_dir_x * turn_angle.cos() - old_dir_y * turn_angle.sin();
        self.dir_y = old_dir_x * turn_angle.sin() + old_dir_y * turn_angle.cos();
        self.plane_x = old_plane_x * turn_angle.cos() - old_plane_y * turn_angle.sin();
        self.plane_y = old_plane_x * turn_angle.sin() + old_plane_y * turn_angle.cos();
    }

    pub fn rotate_camera(self: &mut Self, time_mod: &f32, mouse_rate: i32, sensitivity: f32) {
        let turn_angle = mouse_rate as f32 * time_mod * sensitivity;
        let old_dir_x = self.dir_x;
        let old_dir_y = self.dir_y;
        let old_plane_x = self.plane_x;
        let old_plane_y = self.plane_y;

        self.dir_x = old_dir_x * turn_angle.cos() - old_dir_y * turn_angle.sin();
        self.dir_y = old_dir_x * turn_angle.sin() + old_dir_y * turn_angle.cos();
        self.plane_x = old_plane_x * turn_angle.cos() - old_plane_y * turn_angle.sin();
        self.plane_y = old_plane_x * turn_angle.sin() + old_plane_y * turn_angle.cos();
    }
}

pub struct LevelMap {
    map_width: u32,
    map_height: u32,
    pub map_grid: Vec<Vec<u32>>,
}

impl LevelMap {
    pub fn new(height: u32, width: u32) -> Self {
        LevelMap {
            map_width: width,
            map_height: height,
            map_grid: vec![vec![0; width as usize]; height as usize],
        }
    }
}
