use super::diagnostic_row;
use crate::fractal::adaptive_iterations;
use crate::MAX_ITERATIONS;
use creamui::core::layout::FlexDirection;
use creamui::core::BoxedWidget;
use creamui::{jsx, Color, Symbol};
use rayon::current_num_threads;

pub(super) fn build(
    fps: f32,
    base_iterations: u32,
    render_ms: f32,
    center: crate::camera::ComplexPoint,
    zoom: f64,
) -> BoxedWidget {
    let iterations = adaptive_iterations(base_iterations, zoom, MAX_ITERATIONS);
    let accent = Color::rgb(174, 195, 184);
    let primary = Color::rgb(232, 230, 224);

    Box::new(jsx! {
        <Flex direction={FlexDirection::Column} gap={4.0}>
            {diagnostic_row::build(Symbol::Display, "SYS.FPS", format!("{:>6.1}", fps), accent, primary)}
            {diagnostic_row::build(Symbol::Grid, "PLANE.RE", format!("{:+.12e}", center.x.to_f64()), accent, primary)}
            {diagnostic_row::build(Symbol::Grid, "PLANE.IM", format!("{:+.12e}", center.y.to_f64()), accent, primary)}
            {diagnostic_row::build(Symbol::Search, "ZOOM_MAG", format!("{:.3e}x", zoom), accent, primary)}
            {diagnostic_row::build(Symbol::Sliders, "ITERATIONS", format!("{iterations}"), accent, primary)}
            {diagnostic_row::build(Symbol::Controls, "CPU_THREADS", format!("{}", current_num_threads()), accent, primary)}
            {diagnostic_row::build(Symbol::Appearance, "RENDER_MS", format!("{render_ms:.1} ms"), accent, primary)}
        </Flex>
    })
}
