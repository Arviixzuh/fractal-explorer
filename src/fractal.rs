use crate::camera::{Camera, ComplexPoint, Precise};
use crate::palette::Palette;
use rayon::prelude::*;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FractalMode {
    Mandelbrot,
    Julia,
}

impl FractalMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Mandelbrot => "Mandelbrot",
            Self::Julia => "Julia",
        }
    }
}

pub struct RenderParams {
    pub width: u32,
    pub height: u32,
    pub camera: Camera,
    pub iterations: u32,
    pub palette: Palette,
    pub mode: FractalMode,
}

pub struct RenderedFrame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub elapsed_ms: f32,
    pub camera: Camera,
}

pub fn adaptive_iterations(base_iterations: u32, zoom: f64, maximum: u32) -> u32 {
    let zoom_levels = zoom.max(1.0).log2().ceil().max(0.0) as u32;
    base_iterations
        .saturating_add(zoom_levels.saturating_mul(32))
        .clamp(1, maximum.max(1))
}

pub fn render(params: RenderParams) -> Result<RenderedFrame, String> {
    let pixels_len = (params.width as usize)
        .checked_mul(params.height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| "image dimensions overflow the address space".to_owned())?;
    if params.width == 0 || params.height == 0 {
        return Err("image dimensions must be non-zero".to_owned());
    }

    let started = Instant::now();
    let row_bytes = params.width as usize * 4;
    let interior = params.palette.interior();
    let units_per_pixel = params.camera.units_per_pixel(params.height as f64);
    let use_perturbation = params.camera.zoom >= 1.0e12;
    let reference_orbit = use_perturbation
        .then(|| build_reference_orbit(params.camera.center, params.mode, params.iterations));
    let mut pixels = vec![0_u8; pixels_len];
    pixels
        .par_chunks_mut(row_bytes)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, pixel) in row.chunks_exact_mut(4).enumerate() {
                let offset_x = (x as f64 - params.width as f64 * 0.5) * units_per_pixel;
                let offset_y = (params.height as f64 * 0.5 - y as f64) * units_per_pixel;
                let (iteration, escaped) = if let Some(reference) = &reference_orbit {
                    escape_perturbed(
                        params.camera.center,
                        params.mode,
                        offset_x,
                        offset_y,
                        reference,
                        params.iterations,
                    )
                } else {
                    let point = params.camera.complex_at(
                        x as f64,
                        y as f64,
                        params.width as f64,
                        params.height as f64,
                    );
                    match params.mode {
                        FractalMode::Mandelbrot => {
                            escape_mandelbrot(point.x.to_f64(), point.y.to_f64(), params.iterations)
                        }
                        FractalMode::Julia => escape_julia(
                            point.x.to_f64(),
                            point.y.to_f64(),
                            -0.8,
                            0.156,
                            params.iterations,
                        ),
                    }
                };
                let color = if escaped {
                    params.palette.color(iteration)
                } else {
                    [interior.r, interior.g, interior.b]
                };
                pixel.copy_from_slice(&[color[0], color[1], color[2], 255]);
            }
        });

    Ok(RenderedFrame {
        width: params.width,
        height: params.height,
        pixels,
        elapsed_ms: started.elapsed().as_secs_f32() * 1000.0,
        camera: params.camera,
    })
}

#[derive(Clone, Copy)]
struct ReferencePoint {
    real: Precise,
    imaginary: Precise,
}

fn build_reference_orbit(
    center: ComplexPoint,
    mode: FractalMode,
    max_iterations: u32,
) -> Vec<ReferencePoint> {
    let (mut real, mut imaginary) = match mode {
        FractalMode::Mandelbrot => (Precise::from_f64(0.0), Precise::from_f64(0.0)),
        FractalMode::Julia => (center.x, center.y),
    };
    let mut orbit = Vec::with_capacity(max_iterations as usize);
    for _ in 0..max_iterations {
        orbit.push(ReferencePoint { real, imaginary });
        let real_f64 = real.to_f64();
        let imaginary_f64 = imaginary.to_f64();
        if real_f64.hypot(imaginary_f64) > 1.0e100 {
            break;
        }
        let real_squared = real.mul(real);
        let imaginary_squared = imaginary.mul(imaginary);
        let next_imaginary = real
            .mul(imaginary)
            .mul(Precise::from_f64(2.0))
            .add(match mode {
                FractalMode::Mandelbrot => center.y,
                FractalMode::Julia => Precise::from_f64(0.156),
            });
        real = real_squared.sub(imaginary_squared).add(match mode {
            FractalMode::Mandelbrot => center.x,
            FractalMode::Julia => Precise::from_f64(-0.8),
        });
        imaginary = next_imaginary;
    }
    orbit
}

fn escape_perturbed(
    center: ComplexPoint,
    mode: FractalMode,
    offset_x: f64,
    offset_y: f64,
    reference: &[ReferencePoint],
    max_iterations: u32,
) -> (f64, bool) {
    let (mut delta_real, mut delta_imaginary) = match mode {
        FractalMode::Mandelbrot => (0.0, 0.0),
        FractalMode::Julia => (offset_x, offset_y),
    };
    for iteration in 0..max_iterations {
        let Some(reference) = reference.get(iteration as usize) else {
            return escape_precise(center, mode, offset_x, offset_y, max_iterations);
        };
        let real = reference.real.add_f64(delta_real);
        let imaginary = reference.imaginary.add_f64(delta_imaginary);
        let real_f64 = real.to_f64();
        let imaginary_f64 = imaginary.to_f64();
        let magnitude_squared = real_f64 * real_f64 + imaginary_f64 * imaginary_f64;
        if magnitude_squared > 256.0 {
            return (smooth_iteration(iteration, magnitude_squared), true);
        }

        let reference_real = reference.real.to_f64();
        let reference_imaginary = reference.imaginary.to_f64();
        let next_delta_real = 2.0
            * (reference_real * delta_real - reference_imaginary * delta_imaginary)
            + delta_real * delta_real
            - delta_imaginary * delta_imaginary
            + match mode {
                FractalMode::Mandelbrot => offset_x,
                FractalMode::Julia => 0.0,
            };
        let next_delta_imaginary = 2.0
            * (reference_real * delta_imaginary + reference_imaginary * delta_real)
            + 2.0 * delta_real * delta_imaginary
            + match mode {
                FractalMode::Mandelbrot => offset_y,
                FractalMode::Julia => 0.0,
            };
        delta_real = next_delta_real;
        delta_imaginary = next_delta_imaginary;
    }
    (max_iterations as f64, false)
}

fn escape_precise(
    center: ComplexPoint,
    mode: FractalMode,
    offset_x: f64,
    offset_y: f64,
    max_iterations: u32,
) -> (f64, bool) {
    let (mut real, mut imaginary, constant_real, constant_imaginary) = match mode {
        FractalMode::Mandelbrot => (
            Precise::from_f64(0.0),
            Precise::from_f64(0.0),
            center.x.add_f64(offset_x),
            center.y.add_f64(offset_y),
        ),
        FractalMode::Julia => (
            center.x.add_f64(offset_x),
            center.y.add_f64(offset_y),
            Precise::from_f64(-0.8),
            Precise::from_f64(0.156),
        ),
    };
    for iteration in 0..max_iterations {
        let real_f64 = real.to_f64();
        let imaginary_f64 = imaginary.to_f64();
        let magnitude_squared = real_f64 * real_f64 + imaginary_f64 * imaginary_f64;
        if magnitude_squared > 256.0 {
            return (smooth_iteration(iteration, magnitude_squared), true);
        }
        let real_squared = real.mul(real);
        let imaginary_squared = imaginary.mul(imaginary);
        let next_imaginary = real
            .mul(imaginary)
            .mul(Precise::from_f64(2.0))
            .add(constant_imaginary);
        real = real_squared.sub(imaginary_squared).add(constant_real);
        imaginary = next_imaginary;
    }
    (max_iterations as f64, false)
}

fn escape_mandelbrot(real: f64, imaginary: f64, max_iterations: u32) -> (f64, bool) {
    let (mut zr, mut zi): (f64, f64) = (0.0, 0.0);
    for iteration in 0..max_iterations {
        let zr2 = zr * zr;
        let zi2 = zi * zi;
        if zr2 + zi2 > 256.0 {
            return (smooth_iteration(iteration, zr2 + zi2), true);
        }
        zi = 2.0 * zr * zi + imaginary;
        zr = zr2 - zi2 + real;
    }
    (max_iterations as f64, false)
}

fn escape_julia(
    mut zr: f64,
    mut zi: f64,
    real_constant: f64,
    imaginary_constant: f64,
    max_iterations: u32,
) -> (f64, bool) {
    for iteration in 0..max_iterations {
        let zr2 = zr * zr;
        let zi2 = zi * zi;
        if zr2 + zi2 > 256.0 {
            return (smooth_iteration(iteration, zr2 + zi2), true);
        }
        zi = 2.0 * zr * zi + imaginary_constant;
        zr = zr2 - zi2 + real_constant;
    }
    (max_iterations as f64, false)
}

fn smooth_iteration(iteration: u32, magnitude_squared: f64) -> f64 {
    let magnitude = magnitude_squared.sqrt();
    iteration as f64 + 1.0 - magnitude.ln().ln() / std::f64::consts::LN_2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_inside_mandelbrot_do_not_escape() {
        assert!(!escape_mandelbrot(0.0, 0.0, 100).1);
        assert!(!escape_mandelbrot(-1.0, 0.0, 100).1);
    }

    #[test]
    fn points_outside_mandelbrot_escape() {
        assert!(escape_mandelbrot(2.0, 2.0, 100).1);
    }

    #[test]
    fn render_returns_an_opaque_rgba_frame() {
        let frame = render(RenderParams {
            width: 4,
            height: 3,
            camera: Camera::default(),
            iterations: 32,
            palette: Palette::CyberpunkNeon,
            mode: FractalMode::Mandelbrot,
        })
        .expect("small frame should render successfully");
        assert_eq!((frame.width, frame.height), (4, 3));
        assert_eq!(frame.pixels.len(), 4 * 3 * 4);
        assert!(frame.pixels.chunks_exact(4).all(|pixel| pixel[3] == 255));
    }

    #[test]
    fn iteration_depth_increases_with_zoom_and_respects_the_cap() {
        assert_eq!(adaptive_iterations(320, 1.0, 4096), 320);
        assert_eq!(adaptive_iterations(320, 2.0, 4096), 352);
        assert_eq!(adaptive_iterations(320, 4.0, 4096), 384);
        assert_eq!(adaptive_iterations(4090, 100.0, 4096), 4096);
    }

    #[test]
    fn perturbation_matches_extended_precision_escape_classification() {
        let center = ComplexPoint {
            x: Precise::from_f64(-0.75),
            y: Precise::from_f64(0.1),
        };
        let iterations = 600;
        for zoom in [1.0e14, 1.0e18, 1.0e22] {
            let units = 4.0 / zoom / 600.0;
            let reference = build_reference_orbit(center, FractalMode::Mandelbrot, iterations);
            for x in -2..=2 {
                for y in -2..=2 {
                    let offset_x = x as f64 * units;
                    let offset_y = y as f64 * units;
                    let perturbed = escape_perturbed(
                        center,
                        FractalMode::Mandelbrot,
                        offset_x,
                        offset_y,
                        &reference,
                        iterations,
                    );
                    let precise = escape_precise(
                        center,
                        FractalMode::Mandelbrot,
                        offset_x,
                        offset_y,
                        iterations,
                    );
                    assert_eq!(
                        perturbed.1, precise.1,
                        "classification diverged at zoom {zoom}, pixel ({x}, {y})"
                    );
                    if perturbed.1 {
                        assert!(
                            (perturbed.0 - precise.0).abs() < 1.0,
                            "escape iteration diverged at zoom {zoom}, pixel ({x}, {y})"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn deep_zoom_render_uses_perturbation_and_returns_a_complete_frame() {
        let frame = render(RenderParams {
            width: 8,
            height: 6,
            camera: Camera {
                center: ComplexPoint {
                    x: Precise::from_f64(-0.75),
                    y: Precise::from_f64(0.1),
                },
                zoom: 1.0e18,
            },
            iterations: 256,
            palette: Palette::CyberpunkNeon,
            mode: FractalMode::Mandelbrot,
        })
        .expect("deep zoom frame should render successfully");
        assert_eq!(frame.pixels.len(), 8 * 6 * 4);
        assert!(frame.pixels.chunks_exact(4).all(|pixel| pixel[3] == 255));
    }
}
