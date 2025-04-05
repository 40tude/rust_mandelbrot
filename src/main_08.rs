// main_08
// The image has invariant dimensions
// render_stripe() allocates a fixed size region on the heap

// TODO : issues with scaling
// TODO : add a zoom and be able to move the centre of the view_rectangle in complex space

extern crate num_complex;
extern crate png;

use num_complex::Complex;
use std::fs::File;
use std::io::BufWriter;
use std::thread;
use std::time::Instant;

// ----------------------------------------------------------------------------
fn main() {
    let (width, height) = (640, 480);
    let from = Complex::new(-2.5, -1.315);
    let to = Complex::new(1.0, 1.315);

    let start = Instant::now();
    let image = build_mandelbrot(&from, &to, width, height);
    let duration = start.elapsed();
    println!("Single-threaded : {} ms.", duration.as_millis());
    save_image("./assets/image_rgb_08.png", &image, width, height).expect("Failed to save image");

    let start = Instant::now();
    let image = mt_build_mandelbrot(&from, &to, width, height);
    let duration = start.elapsed();
    println!("Multithreaded   : {} ms.", duration.as_millis());
    save_image("./assets/image_rgb_mt_08.png", &image, width, height)
        .expect("Failed to save image");
}

// ----------------------------------------------------------------------------
fn mt_build_mandelbrot(from: &Complex<f64>, to: &Complex<f64>, width: u32, height: u32) -> Vec<u8> {
    let nthreads = 4;
    println!("# of threads    : {nthreads}");

    let mut handles = vec![];

    let stripe_width = width;
    let stripe_height = height / nthreads;

    let size = to - from;
    let delta_y = (to.im - from.im) / nthreads as f64;

    for i in 0..nthreads {
        let stripe_from = from + Complex::new(0.0, i as f64 * delta_y);
        let stripe_to = stripe_from + Complex::new(size.re, delta_y);

        let handle = thread::spawn(move || {
            // ! no ";" at EOL => the thread return a stripe
            render_stripe(&stripe_from, &stripe_to, stripe_width, stripe_height)
        });
        handles.push(handle);
    }

    let mut stripes = vec![];
    for handle in handles {
        let stripe = handle.join().unwrap();
        stripes.push(stripe);
    }

    let mut final_img = vec![0u8; (width * height * 3) as usize];

    for (i, stripe) in stripes.into_iter().enumerate() {
        let start = i * stripe_height as usize * width as usize * 3;
        final_img[start..start + stripe.len()].copy_from_slice(&stripe);
    }

    final_img
}

// ----------------------------------------------------------------------------
// render_stripe() now returns an array of u8 (see -> Box<[u8]>)
// fixed size area (array) are allocated on the stack (not applicable here)
// Here a Vec<u8> is first created then converted into Box<[u8]>
// capacity and len are lost, only the ptr of the data is kept
// Box <[u8]> helps to express the fact that the allocated memory is invariant
fn render_stripe(from: &Complex<f64>, to: &Complex<f64>, width: u32, height: u32) -> Box<[u8]> {
    // let mut image = vec![0u8; (width * height * 3) as usize];
    let mut image = vec![0u8; (width * height * 3) as usize].into_boxed_slice();

    let size = to - from;

    for y in 0..height {
        for x in 0..width {
            let c = from
                + Complex::new(
                    x as f64 * size.re / width as f64,
                    y as f64 * size.im / height as f64,
                );
            let (r, g, b) = mandelbrot_color(&c);

            let idx = (y * width + x) as usize * 3;
            image[idx] = r;
            image[idx + 1] = g;
            image[idx + 2] = b;
        }
    }
    image
}

// ----------------------------------------------------------------------------
fn build_mandelbrot(from: &Complex<f64>, to: &Complex<f64>, width: u32, height: u32) -> Vec<u8> {
    let mut image: Vec<u8> = Vec::with_capacity((width * height * 3) as usize);
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
    image
}

// ----------------------------------------------------------------------------
fn mandelbrot_color(c: &Complex<f64>) -> (u8, u8, u8) {
    const ITERATIONS: u32 = 250; //1_000;
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
        return (0, 0, 0);
    }

    let zn = z.norm_sqr().sqrt().ln() / 2.0;
    let smooth_i = (i as f64) + 1.0 - zn.ln() / std::f64::consts::LN_2;

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
