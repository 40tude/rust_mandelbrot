// main_05
// Success!
// I really like the Austin Powers pallette

extern crate num_complex;
extern crate png;

use num_complex::Complex;
use std::fs::File;
use std::io::BufWriter;

// ----------------------------------------------------------------------------
fn main() {
    let from = Complex::new(-1.75, -1.0);
    let to = Complex::new(0.75, 1.0);

    let (width, height) = (640, 480);
    let image = build_mandelbrot(&from, &to, width, height);
    save_image("./assets/image_rgb_05.png", &image, width, height).expect("Failed to save image");
}

// ----------------------------------------------------------------------------
fn build_mandelbrot(from: &Complex<f64>, to: &Complex<f64>, width: u32, height: u32) -> Vec<u8> {
    let mut image: Vec<u8> = Vec::with_capacity((width * height * 3) as usize);
    dbg!(image.len(), image.capacity());

    let size = to - from;

    for y in 0..height {
        for x in 0..width {
            let c = from
                + Complex::new(
                    x as f64 * size.re / width as f64,
                    y as f64 * size.im / height as f64,
                );
            let (r, g, b) = mandelbrot_color(&c);
            image.push(r);
            image.push(g);
            image.push(b);
        }
    }
    dbg!(image.len(), image.capacity());
    image
}

// ----------------------------------------------------------------------------
fn mandelbrot_color(c: &Complex<f64>) -> (u8, u8, u8) {
    const ITERATIONS: u32 = 1_000;
    let mut z = Complex::new(0.0, 0.0);
    let mut i = 0;

    for t in 0..ITERATIONS {
        z = z * z + c;
        if z.norm_sqr() > 4.0 {
            i = t;
            break;
        }
    }

    if i == 0 {
        return (0, 0, 0); // in the set 
    }

    // Austin Powers is back in town
    // Soft log scale
    let zn = z.norm_sqr().sqrt().ln() / 2.0;
    let smooth_i = (i as f64) + 1.0 - zn.ln() / std::f64::consts::LN_2;

    // Modulate with HSV palette -> RGB simplified
    let hue = smooth_i * 0.1;
    let r = (0.5 + 0.5 * (6.2831 * (hue + 0.0)).cos()) * 255.0;
    let g = (0.5 + 0.5 * (6.2831 * (hue + 0.33)).cos()) * 255.0;
    let b = (0.5 + 0.5 * (6.2831 * (hue + 0.66)).cos()) * 255.0;

    (r as u8, g as u8, b as u8)
}

// ----------------------------------------------------------------------------
fn save_image(
    filename: &str,
    data: &Vec<u8>,
    width: u32,
    height: u32,
) -> Result<(), png::EncodingError> {
    let file = File::create(filename).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(data.as_slice())
}
