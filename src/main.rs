extern crate num;
extern crate crossbeam;
#[macro_use] extern crate conrod;
extern crate piston_window;

use num::Complex;
use piston_window::*;

fn escape_time(c: Complex<f64>, max_iter: u32) -> Option<u32> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..max_iter {
        z = z * z + c;
        if z.norm_sqr() > 4.0 {
            return Some(i)
        }
    }
    None
}

fn pixel_to_point(bounds: (usize, usize),
                  pixel: (usize, usize),
                  upper_left: Complex<f64>,
                  lower_right: Complex<f64>)
    -> Complex<f64>
{
    let (width, height) = (lower_right.re - upper_left.re,
                           upper_left.im - lower_right.im);
    Complex {
        re: upper_left.re + pixel.0 as f64 * width  / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64
    }
}

fn render(pixels: &mut [u32],
          max_iter: u32,
          bounds: (usize, usize),
          upper_left: Complex<f64>,
          lower_right: Complex<f64>)
{
    assert!(pixels.len() == bounds.0 * bounds.1);

    for row in 0 .. bounds.1 {
        for col in 0 .. bounds.0 {
            let point = pixel_to_point(bounds, (col, row),
                                       upper_left, lower_right);
            pixels[row * bounds.0 + col] = 
                match escape_time(point, max_iter) {
                    None => 0,
                    Some(count) => max_iter - count as u32,
                };
        }
    }
}


fn main() {
    let bounds: (usize, usize) = (500, 500);
    let pixel_size: f64 = 1.0;
    
    let upper_left: Complex<f64> = Complex { re: -1.5, im: 1.0 };
    let lower_right: Complex<f64> = Complex { re: 0.5, im: -1.0 };

    let max_iter: u32 = 50;
        
    let mut pixels = vec![0; bounds.0 * bounds.1];

    // render(&mut pixels, bounds, upper_left, lower_right);
    let threads = 8;
    let rows_per_band = bounds.1 / threads + 1;

    {
        let bands: Vec<&mut [u32]> =
            pixels.chunks_mut(rows_per_band * bounds.0)
                  .collect();
        crossbeam::scope(|spawner| {
            for (i, band) in bands.into_iter().enumerate() {
                let top = rows_per_band * i;
                let height = band.len() / bounds.0;
                let band_bounds = (bounds.0, height);
                let band_upper_left = pixel_to_point(bounds, (0, top),
                                                     upper_left, lower_right);
                let band_lower_right = pixel_to_point(bounds, (bounds.0, top + height),
                                                      upper_left, lower_right);

                spawner.spawn(move || {
                    render(band, max_iter, band_bounds, 
                           band_upper_left, band_lower_right);
                });
            }
        });
    }

    let canvas_width = bounds.0 as u32 * pixel_size as u32;
    let canvas_height = bounds.1 as u32 * pixel_size as u32;
    let mut window: PistonWindow = WindowSettings::new(
        "Mandelbrot Application",
        [canvas_width, canvas_height]
        )
        .exit_on_esc(true)
        .build()
        .unwrap();

    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {
            clear([0.0, 0.0, 0.0, 0.0], g);

            for i in 0..pixels.len() {
                let x_pos = i % bounds.0;
                let y_pos = i / bounds.1;
                let intensity = pixels[i] as f32 / max_iter as f32;
                rectangle([intensity, intensity, intensity, 1.0],
                          [x_pos as f64 * pixel_size,
                           y_pos as f64 * pixel_size,
                           pixel_size, pixel_size],
                          c.transform, g);
            }
        });
    }
}
