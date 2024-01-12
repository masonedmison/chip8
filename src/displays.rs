extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::VideoSubsystem;

const DOT_SIZE_IN_PXS: u32 = 20;

const GRID_WIDTH: usize = 64;
const GRID_HEIGHT: usize = 32;

const SCREEN_WIDTH: u32 = GRID_WIDTH as u32 * DOT_SIZE_IN_PXS;
const SCREEN_HEIGHT: u32 = GRID_HEIGHT as u32 * DOT_SIZE_IN_PXS;

const BACKGROUND_COLOR: Color = Color::RGB(30, 30, 30);

pub struct GamePixels {
    pub value: [[u8; GRID_WIDTH]; GRID_HEIGHT],
}

impl GamePixels {
    pub fn new() -> GamePixels {
        GamePixels {
            value: [[0; GRID_WIDTH]; GRID_HEIGHT],
        }
    }

    pub fn fill_in_bytes(&mut self, bytes: &[u8], (x, y): (usize, usize)) -> bool {
        let mut total_collision: bool = false;

        for (y_idx, byte) in bytes.iter().enumerate() {
            let cur_y_coord = (y + y_idx) % GRID_HEIGHT;
            for x_idx in 0..8 {
                let color = (byte >> (7 - x_idx)) & 1;
                let curr_val = &mut self.value[cur_y_coord][(x + x_idx as usize) % GRID_WIDTH];
                if (color & *curr_val) == 1 { total_collision = true }
                *curr_val ^= color;
            }
        }

        total_collision
    }
}

pub trait Drawable {
    /*
     * Render the binary representation of `bytes` starting at start_coord.
     * If the bits to render expand outside of the display, wrap.
     *
     * Returns True if collision was detected, else False
     */
    fn draw_at(&mut self, bytes: &[u8], start_coord: (usize, usize)) -> bool;
    fn clear(&mut self);
}

impl Drawable for Display {
    fn draw_at(&mut self, bytes: &[u8], start_coord: (usize, usize)) -> bool {
        // convert each byte to [u8;8] - these should be "stacked"
        // update `pixels` with these bits starting at start_coord -- track if there is collision
        let collided = self.pixels.fill_in_bytes(bytes, start_coord);
        // display pixels
        self.draw_pixels().unwrap();

        collided
    }

    fn clear(&mut self) {
        self.pixels = GamePixels::new();
        self.draw_background()
    }
}

pub struct Display {
    pixels: GamePixels,
    canv: WindowCanvas,
}

impl Display {
    pub fn new(video_subsystem: VideoSubsystem) -> Result<Display, String> {
        let window = video_subsystem
            .window("Chip8", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

        canvas.set_draw_color(BACKGROUND_COLOR);
        canvas.clear();
        canvas.present();

        Ok(Display {
            canv: canvas,
            pixels: GamePixels::new(),
        })
    }

    pub fn draw_background(&mut self) {
        self.canv.set_draw_color(BACKGROUND_COLOR);
        self.canv.clear()
    }
    fn draw_pixels(&mut self) -> Result<(), String> {
        for (y, row) in self.pixels.value.iter().enumerate() {
            for (x, bit) in row.iter().enumerate() {
                let x = x as u32 * DOT_SIZE_IN_PXS;
                let y = y as u32 * DOT_SIZE_IN_PXS;
                let rect = Rect::new(x as i32, y as i32, DOT_SIZE_IN_PXS, DOT_SIZE_IN_PXS);
                self.canv.set_draw_color(if *bit == 1 {
                    Color::GREEN
                } else {
                    BACKGROUND_COLOR
                });

                self.canv.fill_rect(rect).unwrap();
            }
        }

        self.canv.present();
        Ok(())
    }
}
