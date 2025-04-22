// main_02
// image weird but in color (./assets/image_rgb_02.png)
// RGB support impacts :
//      - the size of the image
//      - how colors are saved in the image build_mandelbrot()
//      - the signature of mandelbrot_color()

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
    save_image("./assets/image_rgb_02.png", &image, width, height).expect("Failed to save image");
}

// ----------------------------------------------------------------------------
fn build_mandelbrot(from: &Complex<f64>, to: &Complex<f64>, width: u32, height: u32) -> Vec<u8> {
    // let mut image: Vec<u8> = Vec::with_capacity((width * height) as usize);
    let mut image: Vec<u8> = Vec::with_capacity((width * height * 3) as usize); // 3 bytes per pixel (RGB)
    dbg!(image.len(), image.capacity());

    let size = to - from;

    for y in 0..height {
        for x in 0..width {
            let c = from
                + Complex::new(
                    x as f64 * size.re / width as f64,
                    y as f64 * size.im / height as f64,
                );
            // let color = mandelbrot_color(&c);
            // image.push(color);
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

    if z.norm_sqr() > 4.0 {
        // Convert iteration in color (shades of blue)
        let b = (255.0 * (i as f64 / ITERATIONS as f64)) as u8;
        (0, 0, b) // blue
    } else {
        (0, 0, 0) // black
    }
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
    // encoder.set_color(png::ColorType::Grayscale);
    encoder.set_color(png::ColorType::Rgb); // RGB rather than Grayscale
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(data.as_slice())
}
