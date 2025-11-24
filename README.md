# Synth-ID Watermark Removal Tool

A Rust-based image processing tool designed to remove watermarks from AI-generated images by applying various destructive transformations including Gaussian noise, quantization, dithering, and thresholding.

## Features

- **Multiple Removal Modes**: Choose from basic noise, aggressive, destructive, or threshold modes
- **Tunable Parameters**: Fine-tune each mode for optimal results
- **Safety Checks**: Automatic grayscale detection for threshold mode
- **CLI Interface**: Easy-to-use command-line interface with clap

## Installation

```bash
cd main
cargo build --release
```

## Usage

### Basic Mode (Gaussian Noise)
Adds Gaussian noise to the image. Default sigma is 30.

```bash
cargo run -- --input input/image.png --output output/result.png
```

Adjust noise intensity:
```bash
cargo run -- --input input/image.png --output output/result.png --sigma 50
```

### Aggressive Mode
Applies resize, blur, and noise transformations.

```bash
cargo run -- --input input/image.png --output output/result.png --aggressive
```

Fine-tune parameters:
```bash
cargo run -- --input input/image.png --output output/result.png --aggressive --blur-sigma 0.5 --resize-scale 0.95
```

**Parameters:**
- `--blur-sigma`: Blur amount (default: 1.0, lower = less blur)
- `--resize-scale`: Resize factor (default: 0.9, closer to 1.0 = less distortion)

### Destructive Mode
Uses color quantization and Floyd-Steinberg dithering for maximum watermark removal.

```bash
cargo run -- --input input/image.png --output output/result.png --destructive
```

Adjust quantization:
```bash
cargo run -- --input input/image.png --output output/result.png --destructive --levels 4
```

**Parameters:**
- `--levels`: Colors per channel (default: 4, lower = more dithering)

### Threshold Mode (B&W Only)
Converts images to pure black and white. **Only works on grayscale images.**

```bash
cargo run -- --input input/image.png --output output/result.png --bw
```

**Parameters:**
- `--threshold`: Luminance threshold 0-255 (default: 128)
- `--force-bw`: Override grayscale detection (use with caution)

## Project Structure

```
main/
├── src/
│   └── main.rs          # Main application code
├── input/               # Place your images here
├── output/              # Processed images saved here
├── Cargo.toml           # Dependencies
└── Cargo.lock           # Dependency lock file
```

## Dependencies

- `image` - Image processing
- `rand` - Random number generation
- `rand_distr` - Gaussian distribution
- `clap` - Command-line argument parsing

## How It Works

### Gaussian Noise
Adds random noise to each pixel channel, disrupting subtle watermark patterns.

### Aggressive Mode
1. **Resize**: Downscales and upscales the image, destroying high-frequency information
2. **Blur**: Applies Gaussian blur to smooth details
3. **Noise**: Adds Gaussian noise

### Destructive Mode
1. **Quantization**: Reduces color palette to a limited set
2. **Dithering**: Uses Floyd-Steinberg algorithm to approximate original colors

### Threshold Mode
Converts pixels to pure black (0,0,0) or white (255,255,255) based on luminance, eliminating all gray values where watermarks hide.

## Warning

This tool is designed for removing watermarks from AI-generated images. Be aware that:
- Aggressive and destructive modes will degrade image quality
- Results may vary depending on watermark implementation
- Some modes may not work on all types of watermarks

## License

This project is open source and available under the MIT License.