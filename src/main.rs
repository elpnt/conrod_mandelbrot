extern crate num;
extern crate image;
extern crate crossbeam;

use num::Complex;
use image::ColorType;
use image::png::PNGEncoder;
use std::fs::File;

fn escape_time(c: Complex<f64>, max_iter: u16) -> Option<u16> {
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

#[test]
fn test_pixel_to_point() {
    assert_eq!(pixel_to_point((100, 100), (25, 75),
                              Complex { re: -1.0, im: 1.0 },
                              Complex { re: 1.0, im: -1.0 }),
               Complex { re: -0.5, im: -0.5 });
}

fn render(pixels: &mut [u8],
          max_iter: u16,
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
                    Some(count) =>  {
                        let val = max_iter - count as u16;
                        (val as f32 * std::u8::MAX as f32 / max_iter as f32) as u8
                    }
                };
        }
    }
}

fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize))
    -> Result<(), std::io::Error>
{
    let output = File::create(filename)?;

    let encoder = PNGEncoder::new(output);
    encoder.encode(&pixels,
                   bounds.0 as u32, bounds.1 as u32,
                   ColorType::Gray(8))?;
    Ok(())
}


fn main() {
    let bounds: (usize, usize) = (1000, 1000);
    
    let upper_left: Complex<f64> = Complex { re: -1.5, im: 1.0 };
    let lower_right: Complex<f64> = Complex { re: 0.5, im: -1.0 };

    let max_iter: u16 = 55;
        
    let mut pixels = vec![0; bounds.0 * bounds.1];

    // render(&mut pixels, bounds, upper_left, lower_right);
    let threads = 8;
    let rows_per_band = bounds.1 / threads + 1;

    {
        let bands: Vec<&mut [u8]> =
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

    let filename = String::from("mandel.png");
    write_image(&filename, &mut pixels, bounds)
        .expect("Error writing PNG file");
}
