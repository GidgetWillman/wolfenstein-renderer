//we'll use this file to load textures into vectors
//consider a '3D' vector... first dimension is the index, second dimension is y, third dimension is x
use sdl2::pixels::PixelFormatEnum;
use sdl2::surface::Surface;
use sdl2::image::LoadSurface;

pub struct TexturePixelBuffer {
    pub tex_height: usize,
    pub tex_width: usize,
    pub pixel_buffer: Vec<Vec<u32>>,
}

//here we generate the basic buffer for a single texture
impl TexturePixelBuffer {
    pub fn new(height: usize, width: usize) -> Self {
        //blank TexturePixelBuffer
        TexturePixelBuffer {
            tex_height: height,
            tex_width: width,
            pixel_buffer: vec![vec![0; height]; width],
        }
    }

    pub fn from_texture(self: &mut Self, image_path: &str) {
        let surface: Surface = LoadSurface::from_file(image_path).unwrap();
        let format_surface = surface.convert_format(PixelFormatEnum::RGBA8888).unwrap();
        let surface_data = format_surface.without_lock().unwrap();
        let pitch = format_surface.pitch() as usize;
        let mut offset: usize;

        for y in 0..self.tex_height {
            for x in 0..self.tex_width {
                offset = y as usize * pitch + x as usize * 4;

                let red = surface_data[offset];
                let green = surface_data[offset + 1];
                let blue = surface_data[offset + 2];
                let alpha = surface_data[offset + 3];

                let color: u32 = ((red as u32) << 24) | ((green as u32) << 16) | ((blue as u32) << 8) | (alpha as u32);

                self.pixel_buffer[y][x] = color;
            }
        }
    }
}
