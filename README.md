# Mandelbrot Fractal Viewer
An immersive Bevy-powered application that lets you explore the beautiful intricacies of the Mandelbrot fractal. Dive deep into the fractal's infinite intricacies, panning and zooming to witness the awe-inspiring patterns that emerge.

![screenshot 1](https://raw.githubusercontent.com/Lowband21/bevy_mandelbrot/master/screenshot_1.png)



## Table of Contents
- Features
- Installation
- Usage
- Controls
- Acknowledgements
- License

## Features
- Panning: Freely move around the fractal landscape.
- Animated Zooming: Smoothly dive deep or zoom out to appreciate the fractal's vastness.
- High-Resolution Rendering: Witness the Mandelbrot fractal in all its detailed glory.
- Dynamic Coloring: The color of the fractal changes dynamically, creating mesmerizing effects.
- User Configurable Coloring: The color of the fractal is determined by sampling a gradient png that can be swapped out for unique color pallets.
- Boundaries: Set limits to your exploration, ensuring you don't lose yourself in infinity!

## Installation
To run the Mandelbrot Fractal Viewer, you'll need Rust and Cargo installed.

Clone this repository:

```bash

git clone https://github.com/your-username/mandelbrot-viewer.git
cd mandelbrot-viewer
```

Then, run the application:

```bash
cargo run --release
```

## Usage

Once you've launched the Mandelbrot Fractal Viewer, you'll be presented with the fractal's visualization. Use the provided controls to navigate and explore!
## Controls

    Pan: Click and drag using the left or middle mouse button.
    Zoom: Scroll up to zoom in, and scroll down to zoom out.
    Inspector: Press Escape to toggle the WorldInspectorPlugin.

## Acknowledgements

Huge thanks to the bevy_pancam crate for providing the foundational camera controls. The PanCam functionality in this application has been inspired by and adapted from their work.
License

This project is under the MIT License. See the LICENSE file for more details.
