extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::video::Window;

const GRID_X_SIZE: usize = 64;
const GRID_Y_SIZE: usize = 32;
// TODO make configurable
const DOT_SIZE_IN_PXS: u32 = 20;

const DESTRUCTURE_U8: [u8; 8] = [0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01];

struct GamePixels {
    value: [[u8; GRID_Y_SIZE]; GRID_X_SIZE],
}

impl GamePixels {
    fn new() -> GamePixels {
        GamePixels {
            value: [[0; GRID_Y_SIZE]; GRID_X_SIZE],
        }
    }
    fn to_rects(&self) -> Vec<Rect> {
        let mut rects = Vec::new();
        for (row_idx, row) in self.value.iter().enumerate() {
            for (col_idx, col) in row.iter().enumerate() {
                if *col == 1 {
                    let rect = Rect::new(
                        (col_idx * GRID_X_SIZE) as i32,
                        (row_idx * GRID_Y_SIZE) as i32,
                        DOT_SIZE_IN_PXS,
                        DOT_SIZE_IN_PXS,
                    );
                    rects.push(rect);
                }
            }
        }
        rects
    }

    fn byte_to_binary(byte: u8) -> [u8; 8] {
        DESTRUCTURE_U8.map(|mask| if (mask & byte) > 0 { 1 } else { 0 })
    }

    fn fill_in_bytes(&mut self, bytes: &[u8], (x, y): (usize, usize)) -> bool {
        let mut total_collision: bool = false;

        for (y_idx, b) in bytes.iter().enumerate() {
            let bit_array = GamePixels::byte_to_binary(*b);
            let cur_y_coord = (y + y_idx) % GRID_Y_SIZE;

            for (x_idx, bit) in bit_array.iter().enumerate() {
                let curr_val = &mut self.value[cur_y_coord][(x + x_idx) % GRID_X_SIZE];
                if *curr_val == 1 && *curr_val ^ *bit == 0 {
                    total_collision = true
                }
                *curr_val = *bit;
            }
        }

        total_collision
    }
}

pub struct Display {
    pixels: GamePixels,
    canv: WindowCanvas,
}

impl Display {
    pub fn new(window: Window) -> Result<Display, String> {
        window
            .into_canvas()
            .build()
            .map_err(|e| e.to_string())
            .map(|canv| Display {
                canv: canv,
                pixels: GamePixels::new(),
            })
    }
    pub fn clear_display(&mut self) {}
    /*
     * Render the binary representation of `bytes` starting at start_coord.
     * If the bits to render expand outside of the display, wrap.
     *
     * Returns True if collision was detected, else False
     */
    pub fn draw_at(&mut self, bytes: &[u8], start_coord: (usize, usize)) -> bool {
        self.draw_background();
        // convert each byte to [u8;8] - these should be "stacked"
        // update `pixels` with these bits starting at start_coord -- track if there is collision
        let collided = self.pixels.fill_in_bytes(bytes, start_coord);
        // display pixels
        self.draw_pixels().unwrap();

        // return whether collision was detected or not as bool.
        collided
    }

    pub fn clear(&mut self) {
        self.draw_background()
    }

    pub fn draw_background(&mut self) {
        self.canv.set_draw_color(Color::RGB(30, 30, 30));
        self.canv.clear()
    }
    fn draw_pixels(&mut self) -> Result<(), String> {
        self.canv.set_draw_color(Color::WHITE);
        self.canv.draw_rects(&self.pixels.to_rects())?;
        self.canv.present();

        Ok(())
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
    fn destruct_byte() {
        let bits = GamePixels::byte_to_binary(255);
        assert!(bits.into_iter().all(|x| x == 1))
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
