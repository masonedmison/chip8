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
    value: [[u8; GRID_HEIGHT]; GRID_WIDTH],
}

impl GamePixels {
    pub fn new() -> GamePixels {
        GamePixels {
            value: [[0; GRID_HEIGHT]; GRID_WIDTH],
        }
    }
    pub fn to_rects(&self) -> Vec<(Rect, Color)> {
        let mut rects = Vec::new();
        for (row_idx, row) in self.value.iter().enumerate() {
            for (col_idx, col) in row.iter().enumerate() {
                let x = col_idx as u32 * DOT_SIZE_IN_PXS;
                let y = row_idx as u32 * DOT_SIZE_IN_PXS;
                let rect = Rect::new(
                    x as i32,
                    y as i32,
                    (col_idx as u32 * DOT_SIZE_IN_PXS),
                    row_idx as u32 * DOT_SIZE_IN_PXS,
                );
                rects.push(if *col == 1 {
                    (rect, Color::GREEN)
                } else {
                    (rect, BACKGROUND_COLOR)
                });
            }
        }
        rects
    }

    pub fn fill_in_bytes(&mut self, bytes: &[u8], (x, y): (usize, usize)) -> bool {
        let mut total_collision: bool = false;

        for (y_idx, byte) in bytes.iter().enumerate() {
            let cur_y_coord = (y + y_idx) % GRID_HEIGHT;

            for x_idx in 0..8 {
                let color = byte >> (7 - x_idx) & 1;
                let curr_val = &mut self.value[cur_y_coord][(x + x_idx as usize) % GRID_WIDTH];
                if *curr_val == 1 && *curr_val ^ color == 0 {
                    total_collision = true
                }
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
        self.draw_background();
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

        window
            .into_canvas()
            .build()
            .map_err(|e| e.to_string())
            .map(|canv| Display {
                canv: canv,
                pixels: GamePixels::new(),
            })
    }

    pub fn draw_background(&mut self) {
        self.canv.set_draw_color(BACKGROUND_COLOR);
        self.canv.clear()
    }
    fn draw_pixels(&mut self) -> Result<(), String> {
        self.canv.set_draw_color(Color::WHITE);
        self.pixels.to_rects().iter().for_each(|(rect, color)| {
            self.canv.set_draw_color(*color);
            self.canv.draw_rect(*rect).unwrap()
        });
        let drawn = self
            .pixels
            .to_rects()
            .iter()
            .map(|(rect, color)| {
                self.canv.set_draw_color(*color);
                self.canv.fill_rect(*rect)
            })
            .collect::<Result<Vec<_>, _>>();

        drawn.map(|_| self.canv.present())
    }
}

#[cfg(test)]
mod tests {
    use super::GamePixels;
    fn check_one_bytes<T>(pixels: &GamePixels, one_coords: T) -> bool
    where
        T: IntoIterator<Item = (usize, usize)>,
    {
        one_coords.into_iter().all(|(x, y)| pixels.value[y][x] == 1)
    }

    #[test]
    fn fill_in_bytes_no_collision() {
        let mut pixels = GamePixels::new();
        let bytes_to_write = [255, 255];
        let collided = pixels.fill_in_bytes(&bytes_to_write, (0, 0));

        let one_bytes = (0..8).zip([0; 8]).chain((0..8).zip([1; 8]));

        let one_bytes_assertion = check_one_bytes(&pixels, one_bytes);
        assert!(!collided && one_bytes_assertion)
    }

    #[test]
    fn fill_in_bytes_collision() {
        let mut pixels = GamePixels::new();
        let bytes_to_write1 = [255, 255];
        let collided_after_1 = pixels.fill_in_bytes(&bytes_to_write1, (0, 0));

        let bytes_to_write2 = [128];
        let collided_after_2 = pixels.fill_in_bytes(&bytes_to_write2, (0, 0));

        let one_bytes = [(0, 1)].into_iter().chain((0..8).zip([1; 8]));

        let one_bytes_assertion = check_one_bytes(&pixels, one_bytes);
        assert!(!collided_after_1 && one_bytes_assertion && collided_after_2)
    }
}
