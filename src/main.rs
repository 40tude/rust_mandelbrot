// main_10
// the code has been refactored
// the image is now pre-allocated and the processing is done in place
// the processing is done by one function render_zone() (code factorization)
// render_zone() is called in both single-threaded and multithreaded portion of the code
// this requires crossbeam::thread

// TODO : issues with scaling
// TODO : add a zoom and be able to move the centre of the view_rectangle in complex space

extern crate num_complex;
extern crate png;

use num_complex::Complex;
use std::fs::File;
use std::io::BufWriter;
use std::time::Instant;

// std::thread is replaced by crossbeam::thread BUT I want to make sure where crossbeam::thread is used in the code
// use std::thread;
// use crossbeam::thread; // => uncomment crossbeam = "0.8.4" in cargo.toml

// ----------------------------------------------------------------------------
// image is Box<[u8]> and not Vec<u8> because the image size is invariant
fn main() {
    let (width, height) = (640, 480);

    let from = Complex::new(-2.5, -1.315);
    let to = Complex::new(1.0, 1.315);

    let mut image = vec![0u8; (width * height * 3) as usize].into_boxed_slice();

    let start = Instant::now();
    render_zone(&from, &to, width, height, &mut image);
    let duration = start.elapsed();
    println!("Single-threaded : {} ms.", duration.as_millis());
    save_image("./assets/image_rgb_10.png", &image, width, height).expect("Failed to save image");

    let start = Instant::now();
    mt_build_mandelbrot(&from, &to, width, height, &mut image);
    let duration = start.elapsed();
    println!("Multithreaded   : {} ms.", duration.as_millis());
    save_image("./assets/image_rgb_mt_10.png", &image, width, height)
        .expect("Failed to save image");
}

// ----------------------------------------------------------------------------
// does not return an image
// no longer joins the stripes to rebuild the image
// try to be smarter if the number of cores does not divide the height of the image evenly
// .split_at_mut() is used to help the compiler "understand" (trust in me, just in me...) that each thread work on non-overlapping parts of the image
fn mt_build_mandelbrot(
    from: &Complex<f64>,
    to: &Complex<f64>,
    width: u32,
    height: u32,
    image: &mut [u8],
) {
    let nthreads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    println!("# of threads    : {nthreads}");

    let stripe_width = width;
    // let stripe_height = height / nthreads;
    // the value height / nthreads, nthreads times
    let mut stripe_heights = vec![height / nthreads as u32; nthreads];
    // handles cases where height is not divisible by nthreads (480 and 7 for example)
    // 480 % 7 = 4 - the first 4 stripes receive 1 additional line each
    for i in 0..(height % nthreads as u32) {
        stripe_heights[i as usize] += 1;
    }

    let size = to - from;
    let delta_y = size.im / height as f64;

    let mut y_start = 0;
    // the scope guarantees that all threads are joined before the end of the block.
    // so the compiler "knows" that threads won't escape and that local references will live long enough.
    crossbeam::thread::scope(|my_scope| {
        let mut remaining = image;

        for stripe_height in stripe_heights {
            let stripe_byte_count = (stripe_height * stripe_width * 3) as usize;

            // .split_at_mut() returns 2 disjoint mutables slices
            let (stripe, rest) = remaining.split_at_mut(stripe_byte_count);
            remaining = rest;

            let y_end = y_start + stripe_height;
            let stripe_from = from + Complex::new(0.0, y_start as f64 * delta_y);
            let stripe_to = from + Complex::new(size.re, y_end as f64 * delta_y);

            my_scope.spawn(move |_| {
                render_zone(
                    &stripe_from,
                    &stripe_to,
                    stripe_width,
                    stripe_height as u32,
                    stripe,
                );
            });

            y_start = y_end;
        }
    })
    .unwrap(); // .expect("A thread panicked during Mandelbrot computation");
}

// ----------------------------------------------------------------------------
// does not return an image
// image is pre-allocated and the processing is done in place
// fn render_stripe(from: &Complex<f64>, to: &Complex<f64>, width: u32, height: u32) -> Box<[u8]>
fn render_zone(from: &Complex<f64>, to: &Complex<f64>, width: u32, height: u32, image: &mut [u8]) {
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
            image[idx + 0] = r;
            image[idx + 1] = g;
            image[idx + 2] = b;
        }
    }
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
// data is now &[u8] (so far it used to be &Vec<u8>)
fn save_image(
    filename: &str,
    data: &[u8],
    width: u32,
    height: u32,
) -> Result<(), png::EncodingError> {
    let file = File::create(filename).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(data)
}
