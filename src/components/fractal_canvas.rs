use crate::camera::Camera;
use crate::DisplayFrame;
use creamui::core::{BoxedWidget, Painter, Point, Rect, Style, TextAlign, Widget};
use creamui::Color;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Instant;

pub(crate) struct Props {
    pub(crate) style: Style,
    pub(crate) image: Option<DisplayFrame>,
    pub(crate) visual_camera: Rc<Cell<Camera>>,
    pub(crate) frame_counter: Rc<RefCell<(Instant, u32, f32)>>,
    pub(crate) on_drag_start: Rc<dyn Fn(Point, Rect)>,
    pub(crate) on_drag: Rc<dyn Fn(Point, Rect)>,
    pub(crate) on_drag_end: Rc<dyn Fn()>,
    pub(crate) on_scroll: Rc<dyn Fn(f32)>,
}

pub(crate) fn build(props: Props) -> BoxedWidget {
    Box::new(FractalCanvas {
        style: props.style,
        image: props.image,
        visual_camera: props.visual_camera,
        frame_counter: props.frame_counter,
        on_drag_start: props.on_drag_start,
        on_drag: props.on_drag,
        on_drag_end: props.on_drag_end,
        on_scroll: props.on_scroll,
    })
}

struct FractalCanvas {
    style: Style,
    image: Option<DisplayFrame>,
    visual_camera: Rc<Cell<Camera>>,
    frame_counter: Rc<RefCell<(Instant, u32, f32)>>,
    on_drag_start: Rc<dyn Fn(Point, Rect)>,
    on_drag: Rc<dyn Fn(Point, Rect)>,
    on_drag_end: Rc<dyn Fn()>,
    on_scroll: Rc<dyn Fn(f32)>,
}

impl Widget for FractalCanvas {
    fn style(&self) -> Style {
        self.style.clone()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        update_fps(&self.frame_counter);
        painter.fill_rect(rect, Color::rgb(4, 5, 5), 0.0);
        if let Some(frame) = &self.image {
            let transformed = transformed_frame_rect(rect, frame.camera, self.visual_camera.get());
            painter.push_clip(rect);
            painter.draw_image(transformed, frame.image.image(), None);
            painter.pop_clip();
        } else {
            painter.fill_text(
                rect,
                "INITIALIZING FRACTAL ENGINE...",
                Color::rgb(174, 195, 184),
                14.0,
                TextAlign::Center,
            );
        }
    }

    fn on_drag_start(&self) -> Option<Rc<dyn Fn(Point, Rect)>> {
        Some(self.on_drag_start.clone())
    }

    fn on_drag(&self) -> Option<Rc<dyn Fn(Point, Rect)>> {
        Some(self.on_drag.clone())
    }

    fn on_drag_end(&self) -> Option<Rc<dyn Fn()>> {
        Some(self.on_drag_end.clone())
    }

    fn on_scroll(&self) -> Option<Rc<dyn Fn(f32)>> {
        Some(self.on_scroll.clone())
    }
}

fn update_fps(frame_counter: &Rc<RefCell<(Instant, u32, f32)>>) {
    let now = Instant::now();
    let mut counter = frame_counter.borrow_mut();
    counter.1 += 1;
    let elapsed = now.duration_since(counter.0).as_secs_f32();
    if elapsed >= 0.5 {
        counter.2 = counter.1 as f32 / elapsed;
        counter.0 = now;
        counter.1 = 0;
    }
}

pub(crate) fn transformed_frame_rect(
    rect: Rect,
    frame_camera: Camera,
    current_camera: Camera,
) -> Rect {
    let scale = current_camera.zoom / frame_camera.zoom;
    let half_height = rect.height as f64 * 0.5;
    Rect {
        x: (rect.x as f64
            + rect.width as f64 * 0.5
            + frame_camera.center.x.sub(current_camera.center.x).to_f64()
                * current_camera.zoom
                * half_height
                / 2.0
            - rect.width as f64 * scale * 0.5) as f32,
        y: (rect.y as f64
            + rect.height as f64 * 0.5
            + current_camera.center.y.sub(frame_camera.center.y).to_f64()
                * current_camera.zoom
                * half_height
                / 2.0
            - rect.height as f64 * scale * 0.5) as f32,
        width: (rect.width as f64 * scale) as f32,
        height: (rect.height as f64 * scale) as f32,
    }
}
