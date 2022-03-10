pub use ui::Screen;
pub use ui::Buffer;
pub use ui::WIDTH;
pub use ui::HEIGHT;
pub use ui::DisplayBuffer;

pub mod ui {
    use sdl2::Sdl;
    use sdl2::video::Window;
    use sdl2::render::Canvas;
    use sdl2::pixels::Color;
    use sdl2::rect::Rect;

    use crate::emulator::Chip8Result;

    pub const WIDTH: i32 = 64;
    pub const HEIGHT: i32 = 32;
    const PIXEL_WH: i32 = 10;
    const SCREEN_WIDTH: u32 = WIDTH as u32 * PIXEL_WH as u32;
    const SCREEN_HEIGHT: u32 = HEIGHT as u32 * PIXEL_WH as u32;

    const BACKGROUND: (u8, u8, u8) = (0, 0, 0);
    const FILL: (u8, u8, u8) = (255, 255, 255);

    pub type Buffer = [[bool; WIDTH as usize]; HEIGHT as usize];

    pub struct Screen {
        canvas: Canvas<Window>,
        pub display_buffer: DisplayBuffer,
    }

    pub struct DisplayBuffer {
        pub buffer: Buffer,
        pub is_dirty: bool,
    }

    impl Screen {
        pub fn new(sdl_context: &Sdl, title: &str) -> Chip8Result<Screen> {
            let video_subsystem = sdl_context.video()?;
            let window = video_subsystem.window(title, SCREEN_WIDTH, SCREEN_HEIGHT)
                .position_centered()
                .build()?;

            let canvas = window.into_canvas().build()?;
            let display_buffer = DisplayBuffer {
                buffer: [[false; WIDTH as usize]; HEIGHT as usize],
                is_dirty: true,
            };

            Ok(Screen {
                canvas,
                display_buffer,
            })
        }

        pub fn draw(&mut self) -> Chip8Result<()> {
            if !self.display_buffer.is_dirty {
                return Ok(());
            }

            self.canvas.set_draw_color(Color::RGB(BACKGROUND.0, BACKGROUND.1, BACKGROUND.2));
            self.canvas.clear();

            self.canvas.set_draw_color(Color::RGB(FILL.0, FILL.1, FILL.2));

            for (i, cols) in self.display_buffer.buffer.iter().enumerate() {
                for (j, is_on) in cols.iter().enumerate() {
                    if *is_on {
                        self.canvas.fill_rect(Rect::new(
                            j as i32 * PIXEL_WH,
                            i as i32 * PIXEL_WH,
                            PIXEL_WH as u32,
                            PIXEL_WH as u32
                        ))?;
                    }
                }
            }

            self.canvas.present();

            Ok(())
        }
    }
}
