use clap::Parser;
use image::{imageops, ImageBuffer, Pixel, Rgb};
use rand::thread_rng;
use rand_distr::{Distribution, Normal};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the input image
    #[arg(short, long)]
    input: PathBuf,

    /// Path to the output image
    #[arg(short, long)]
    output: PathBuf,

    /// Standard deviation of the Gaussian noise
    #[arg(short, long, default_value_t = 30.0)]
    sigma: f64,

    /// Enable aggressive removal (Resize + Blur + Noise)
    #[arg(short, long)]
    aggressive: bool,

    /// Gaussian blur sigma for aggressive mode
    #[arg(long, default_value_t = 1.0)]
    blur_sigma: f32,

    /// Resize scale for aggressive mode (e.g., 0.9 for 90%)
    #[arg(long, default_value_t = 0.9)]
    resize_scale: f64,

    /// Enable destructive removal (Quantization + Dithering)
    #[arg(short, long)]
    destructive: bool,

    /// Number of quantization levels per channel (for destructive mode)
    #[arg(long, default_value_t = 4)]
    levels: u8,

    /// Enable Threshold Mode (Binary Black & White)
    #[arg(long = "bw")]
    threshold_mode: bool,

    /// Threshold value for BW mode (0-255)
    #[arg(long, default_value_t = 128)]
    threshold: u8,

    /// Force BW mode even if image looks colored
    #[arg(long)]
    force_bw: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Open the image
    let mut img = image::open(&args.input)?.to_rgb8();
    let (width, height) = img.dimensions();

    if args.threshold_mode {
        println!("Applying Threshold Mode (Binary B&W)...");
        
        // Robust Grayscale Detection
        if !args.force_bw {
            let mut is_grayscale = true;
            let tolerance = 30; // Tolerance for noise
            for pixel in img.pixels() {
                let r = pixel[0] as i16;
                let g = pixel[1] as i16;
                let b = pixel[2] as i16;
                if (r - g).abs() > tolerance || (g - b).abs() > tolerance || (b - r).abs() > tolerance {
                    is_grayscale = false;
                    break;
                }
            }

            if !is_grayscale {
                return Err("Image appears to be color. Use --force-bw to proceed.".into());
            }
        }

        let threshold = args.threshold;
        for pixel in img.pixels_mut() {
            let r = pixel[0] as u32;
            let g = pixel[1] as u32;
            let b = pixel[2] as u32;
            // Calculate luminance (standard Rec. 601)
            let luma = (r * 299 + g * 587 + b * 114) / 1000;
            
            if luma > threshold as u32 {
                *pixel = Rgb([255, 255, 255]);
            } else {
                *pixel = Rgb([0, 0, 0]);
            }
        }
        
        // Skip other processing
        img.save(&args.output)?;
        println!("Saved binary B&W image to {:?}", args.output);
        return Ok(());
    }

    if args.destructive {
        println!("Applying destructive removal (Quantization + Dithering)...");
        // Floyd-Steinberg Dithering
        let levels = args.levels as f32;
        let step = 255.0 / (levels - 1.0);

        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, y);
                let old_r = pixel[0] as f32;
                let old_g = pixel[1] as f32;
                let old_b = pixel[2] as f32;

                let new_r = (old_r / step).round() * step;
                let new_g = (old_g / step).round() * step;
                let new_b = (old_b / step).round() * step;

                img.put_pixel(x, y, Rgb([
                    new_r.clamp(0.0, 255.0) as u8,
                    new_g.clamp(0.0, 255.0) as u8,
                    new_b.clamp(0.0, 255.0) as u8,
                ]));

                let err_r = old_r - new_r;
                let err_g = old_g - new_g;
                let err_b = old_b - new_b;

                // Distribute error to neighbors
                let distribute = |img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, x: u32, y: u32, factor: f32| {
                    if x < width && y < height {
                        let p = img.get_pixel(x, y);
                        let r = (p[0] as f32 + err_r * factor).clamp(0.0, 255.0) as u8;
                        let g = (p[1] as f32 + err_g * factor).clamp(0.0, 255.0) as u8;
                        let b = (p[2] as f32 + err_b * factor).clamp(0.0, 255.0) as u8;
                        img.put_pixel(x, y, Rgb([r, g, b]));
                    }
                };

                distribute(&mut img, x + 1, y, 7.0 / 16.0);
                distribute(&mut img, x.wrapping_sub(1), y + 1, 3.0 / 16.0);
                distribute(&mut img, x, y + 1, 5.0 / 16.0);
                distribute(&mut img, x + 1, y + 1, 1.0 / 16.0);
            }
        }
    } else if args.aggressive {
        println!("Applying aggressive removal...");
        // 1. Resize (Resample)
        let new_w = (width as f64 * args.resize_scale) as u32;
        let new_h = (height as f64 * args.resize_scale) as u32;
        let mut dynamic_img = image::DynamicImage::ImageRgb8(img);
        dynamic_img = dynamic_img.resize(new_w, new_h, imageops::FilterType::Triangle);
        dynamic_img = dynamic_img.resize(width, height, imageops::FilterType::Triangle);

        // 2. Blur
        dynamic_img = dynamic_img.blur(args.blur_sigma);
        img = dynamic_img.to_rgb8();
    }

    let mut output_img = ImageBuffer::new(width, height);

    // Create a normal distribution
    let normal = Normal::new(0.0, args.sigma)?;
    let mut rng = thread_rng();

    for (x, y, pixel) in img.enumerate_pixels() {
        let rgb = pixel.to_rgb();
        let r = rgb[0] as f64;
        let g = rgb[1] as f64;
        let b = rgb[2] as f64;

        // Add noise
        let noise_r = normal.sample(&mut rng);
        let noise_g = normal.sample(&mut rng);
        let noise_b = normal.sample(&mut rng);

        let new_r = (r + noise_r).clamp(0.0, 255.0) as u8;
        let new_g = (g + noise_g).clamp(0.0, 255.0) as u8;
        let new_b = (b + noise_b).clamp(0.0, 255.0) as u8;

        output_img.put_pixel(x, y, Rgb([new_r, new_g, new_b]));
    }

    // Save the image
    output_img.save(&args.output)?;
    println!("Saved noisy image to {:?}", args.output);

    Ok(())
}
