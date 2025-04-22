// main_11
// Not a good idea...
// Use an Arc and a mutex to globally protect the access to the image
// Performances are worst than in single-threaded
// Not a surprise. 20 people : There are 20 people around the sink washing dishes, but only one has access to the sponge at a time.

// TODO : issues with scaling
// TODO : add a zoom and be able to move the centre of the view_rectangle in complex space

extern crate num_complex;
extern crate png;

use num_complex::Complex;
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

// ----------------------------------------------------------------------------
fn main() {
    let (width, height) = (640, 480);
    let from = Complex::new(-2.5, -1.315);
    let to = Complex::new(1.0, 1.315);

    let mut image = vec![0u8; (width * height * 3) as usize].into_boxed_slice();
    let start = Instant::now();
    render_zone(&from, &to, width, height, &mut image);
    let duration = start.elapsed();
    println!("Single-threaded : {} ms.", duration.as_millis());
    save_image("./assets/image_rgb_11.png", &image, width, height).expect("Failed to save image");

    let image_mt = vec![0u8; (width * height * 3) as usize].into_boxed_slice();
    let start = Instant::now();
    let image_mt = mt_build_mandelbrot(&from, &to, width, height, image_mt);
    let duration = start.elapsed();
    println!("Multithreaded   : {} ms.", duration.as_millis());
    save_image("./assets/image_rgb_mt_11.png", &image_mt, width, height)
        .expect("Failed to save image");
}

// ----------------------------------------------------------------------------
// Arc lets us share immutable data between threads, but for mutable data (such as the image to be modified), we need to associate it with a Mutex or RwLock.
// We use Arc<Mutex<&mut [u8]>> to share the buffer between threads.
// Each thread will work on a stripe of the image.
// It's not ultra-performant, as the Mutex is global, but it's a good pedagogical test.
// The buffer is locked for the duration of the stripes processed in each thread.
// TO KEEP IN MIND : Arc is cloned, but not the entire buffer. The buffer is shared in memory, not duplicated.
fn mt_build_mandelbrot(
    from: &Complex<f64>,
    to: &Complex<f64>,
    width: u32,
    height: u32,
    image: Box<[u8]>,
) -> Box<[u8]> {
    let nthreads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1) as u32;
    println!("# of threads    : {nthreads}");

    let rows_per_thread = height / nthreads;
    let buffer = Arc::new(Mutex::new(image));

    let mut handles = vec![];

    for i in 0..nthreads {
        let y_start = i * rows_per_thread;
        let y_end = if i == nthreads - 1 {
            height
        } else {
            (i + 1) * rows_per_thread
        };

        let buffer_clone = Arc::clone(&buffer);
        let from = *from;
        let to = *to;

        let handle = thread::spawn(move || {
            let mut guard = buffer_clone.lock().unwrap();
            let slice = &mut guard[..];
            let size = to - from;

            for y in y_start..y_end {
                for x in 0..width {
                    let c = from
                        + Complex::new(
                            x as f64 * size.re / width as f64,
                            y as f64 * size.im / height as f64,
                        );
                    let (r, g, b) = mandelbrot_color(&c);
                    let idx = (y * width + x) as usize * 3;
                    slice[idx] = r;
                    slice[idx + 1] = g;
                    slice[idx + 2] = b;
                }
            }
        });

        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    // Return the content of Arc<Mutex<Box<[u8]>>>
    Arc::try_unwrap(buffer)
        .expect("Multiple references exist, cannot unwrap Arc")
        .into_inner()
        .expect("Mutex poisoned")
}

// ----------------------------------------------------------------------------
// target is supposed to be allocated and filled
fn render_zone(from: &Complex<f64>, to: &Complex<f64>, width: u32, height: u32, target: &mut [u8]) {
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
            target[idx + 0] = r;
            target[idx + 1] = g;
            target[idx + 2] = b;
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
