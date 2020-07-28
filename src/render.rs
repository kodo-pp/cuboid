use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::video::WindowSurfaceRef;
use super::SdlError;

pub struct Renderer {}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer{}
    }

    pub fn render<'a>(&mut self, mut surface_ref: WindowSurfaceRef<'a>) -> Result<(), SdlError> {
        surface_ref.fill_rect(None, Color::BLACK)?;
        let _width = surface_ref.width();
        let _height = surface_ref.height();

        let pixel_format_enum = surface_ref.pixel_format_enum();
        match pixel_format_enum {
            PixelFormatEnum::RGB888 | PixelFormatEnum::RGBA8888 | PixelFormatEnum::RGBX8888 => {},
            _ => panic!("Unsupported pixel format: {:?}", pixel_format_enum),
        }

        let bpp = pixel_format_enum.byte_size_per_pixel();
        assert_eq!(bpp, 4, "Non 4-byte pixels are not supported");

        surface_ref.with_lock_mut(|data| {
            for slice in data.chunks_exact_mut(pixel_format_enum.byte_size_per_pixel()) {
                slice[0] = 0;
                slice[1] = 100;
                slice[2] = 200;
                slice[3] = 255;
            }
        });
        surface_ref.finish()?;
        Ok(())
    }
}
