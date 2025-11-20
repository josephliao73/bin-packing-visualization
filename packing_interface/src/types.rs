use serde::{Serialize, Deserialize};
use iced::widget::{text_editor};
use iced::widget::canvas::{self};
use iced::widget::canvas::event::Event;
use iced::mouse;
use iced::{Color};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct Rectangle {
    pub width: i32,
    pub height: i32,
    pub quantity: i32,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct JsonInput {
    pub width_of_bin: i32,
    pub number_of_rectangles: usize,
    pub number_of_types_of_rectangles: usize,
    pub autofill_option: bool,
    pub rectangle_list: Vec<Rectangle>
}

#[derive(Debug, Clone)]
pub enum Input {
    WChanged(String),
    NChanged(String),
    KChanged(String),
    AutofillChanged(bool),
    ImportPressed,
    ImportOutputJsonPressed,
    RectangleDataAction(text_editor::Action),
    ExportAlgorithmInput,
    ZoomChanged(f32),
    Tick,
    AnimationSpeedChanged(f32),
    PanStart(f32, f32),
    PanMove(f32, f32),
    PanEnd,
    RectangleHovered(Option<usize>),
    RectangleDragStart(usize, f32, f32),
    RectangleDragMove(f32, f32),
    RectangleDragEnd,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Placement {
    pub x: f32,
    pub y: f32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlgorithmOutput {
    pub bin_width: i32,
    pub total_height: i32,
    pub placements: Vec<Placement>,
}

pub struct PackingApp {
    pub w_input: String,
    pub n_input: String,
    pub k_input: String,
    pub autofile: bool,
    pub rectangle_data: text_editor::Content,
    pub error_message: Option<String>,
    pub algorithm_output: Option<AlgorithmOutput>,
    pub zoom: f32,
    pub visible_rects: usize,
    pub animating: bool,
    pub animation_speed: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub is_panning: bool,
    pub last_mouse_x: f32,
    pub last_mouse_y: f32,
    pub hovered_rect: Option<usize>,
    pub dragged_rect: Option<usize>,
    pub dragged_rect_offset_x: f32,
    pub dragged_rect_offset_y: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParseOutput {
    pub width: i32,
    pub quantity: i32,
    pub types: i32,
    pub autofill: bool,
    pub rects: Vec<Rectangle>,
    pub input_types: i32,
    pub min_height: i32,
    pub max_height: i32,
}

pub struct BinCanvas<'a>  {
    pub output: &'a AlgorithmOutput,
    pub zoom: f32,
    pub visible_count: usize,
    pub pan_x: f32,
    pub pan_y: f32,
    pub hovered_rect: Option<usize>,
    pub is_panning: bool,
    pub dragged_rect: Option<usize>,
    pub dragged_rect_offset_x: f32,
    pub dragged_rect_offset_y: f32,
    pub animating: bool,
}

impl<'a> BinCanvas<'a> {
    fn find_rectangle_at_point(&self, x: f32, y: f32, bounds: &iced::Rectangle, scale: f32, origin_x: f32, origin_y: f32, bin_h_units: f32) -> Option<usize> {
        let total = self.output.placements.len();
        let count = self.visible_count.min(total);

        let local_x = x - bounds.x;
        let local_y = y - bounds.y;

        for (idx, p) in self.output.placements.iter().enumerate().take(count).rev() {
            let w = p.width as f32 * scale;
            let h = p.height as f32 * scale;
            let rect_x = origin_x + p.x * scale;
            let rect_y = origin_y + (bin_h_units - (p.y + p.height as f32)) * scale;

            if local_x >= rect_x && local_x <= rect_x + w && local_y >= rect_y && local_y <= rect_y + h {
                return Some(idx);
            }
        }
        None
    }
}

impl<'a> iced::widget::canvas::Program<Input> for BinCanvas<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        use iced::widget::canvas::{Frame, Path, Stroke, Fill};
        use iced::{Point, Size};

        let mut frame = Frame::new(renderer, bounds.size());

        let bin_w_units = self.output.bin_width as f32;
        let bin_h_units = self.output.total_height as f32;

        if bin_w_units <= 0.0 || bin_h_units <= 0.0 {
            return vec![frame.into_geometry()];
        }

        let fit_x = bounds.width / bin_w_units;
        let fit_y = bounds.height / bin_h_units;
        let base_scale = fit_x.min(fit_y);
        let scale = base_scale * self.zoom;

        let draw_w = bin_w_units * scale;
        let draw_h = bin_h_units * scale;

        let origin_x = (bounds.width - draw_w) / 2.0 + self.pan_x;
        let origin_y = (bounds.height - draw_h) / 2.0 + self.pan_y;

        let bin_path = Path::rectangle(
            Point::new(origin_x, origin_y),
            Size::new(draw_w, draw_h),
        );
        frame.stroke(&bin_path, Stroke::default().with_color(Color::from_rgb(1.0, 0.65, 0.0)).with_width(2.0));

        let total = self.output.placements.len();
        let count = self.visible_count.min(total);

        for (idx, p) in self.output.placements.iter().enumerate().take(count) {
            if self.dragged_rect == Some(idx) {
                continue;
            }

            let w = p.width as f32 * scale;
            let h = p.height as f32 * scale;
            let x_px = origin_x + p.x * scale;
            let y_px = origin_y
                + (bin_h_units - (p.y + p.height as f32)) * scale;

            let rect_path = Path::rectangle(Point::new(x_px, y_px), Size::new(w, h));
            let color = color_from_dimensions(p.width, p.height);
            frame.fill(&rect_path, Fill::from(color));
            frame.stroke(&rect_path, Stroke::default());
        }

        if let Some(hovered_idx) = self.hovered_rect && hovered_idx < count && self.dragged_rect != Some(hovered_idx) {
                let p = &self.output.placements[hovered_idx];
                let w = p.width as f32 * scale;
                let h = p.height as f32 * scale;
                let x_px = origin_x + p.x * scale;
                let y_px = origin_y
                    + (bin_h_units - (p.y + p.height as f32)) * scale;

                let rect_path = Path::rectangle(Point::new(x_px, y_px), Size::new(w, h));
                let stroke_color = Color::from_rgb(0.4, 0.8, 1.0);
                frame.stroke(&rect_path, Stroke::default().with_color(stroke_color).with_width(2.0));
            }

        if let Some(dragged_idx) = self.dragged_rect && dragged_idx < count {
                let p = &self.output.placements[dragged_idx];
                let w = p.width as f32 * scale;
                let h = p.height as f32 * scale;
                let x_px = origin_x + p.x * scale + self.dragged_rect_offset_x;
                let y_px = origin_y
                    + (bin_h_units - (p.y + p.height as f32)) * scale + self.dragged_rect_offset_y;

                println!("Dragging Rectangle #{}: Original({:.1}, {:.1}) + Offset({:.1}, {:.1}) = Screen({:.1}, {:.1}) | Bin Coords({:.1}, {:.1})",
                    dragged_idx,
                    p.x, p.y,
                    self.dragged_rect_offset_x, self.dragged_rect_offset_y,
                    x_px, y_px,
                    p.x + (self.dragged_rect_offset_x / scale),
                    p.y + (self.dragged_rect_offset_y / scale)
                );

                let rect_path = Path::rectangle(Point::new(x_px, y_px), Size::new(w, h));
                let color = color_from_dimensions(p.width, p.height);
                frame.fill(&rect_path, Fill::from(color));
                let stroke_color = Color::from_rgb(1.0, 0.0, 0.0);
                frame.stroke(&rect_path, Stroke::default().with_color(stroke_color).with_width(2.0));

            }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: Event,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Input>) {
        let bin_w_units = self.output.bin_width as f32;
        let bin_h_units = self.output.total_height as f32;

        if bin_w_units <= 0.0 || bin_h_units <= 0.0 {
            return (canvas::event::Status::Ignored, None);
        }

        let fit_x = bounds.width / bin_w_units;
        let fit_y = bounds.height / bin_h_units;
        let base_scale = fit_x.min(fit_y);
        let scale = base_scale * self.zoom;

        let draw_w = bin_w_units * scale;
        let draw_h = bin_h_units * scale;

        let origin_x = (bounds.width - draw_w) / 2.0 + self.pan_x;
        let origin_y = (bounds.height - draw_h) / 2.0 + self.pan_y;

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let dy = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => y,
                    mouse::ScrollDelta::Pixels { y, .. } => y / 100.0,
                };

                let factor = if dy > 0.0 { 1.1 } else { 0.9 };
                (canvas::event::Status::Captured, Some(Input::ZoomChanged(factor)))
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(position) = cursor.position() {
                    let canvas_rect = iced::Rectangle {
                        x: bounds.x,
                        y: bounds.y,
                        width: bounds.width,
                        height: bounds.height,
                    };

                    if !canvas_rect.contains(position) {
                        return (canvas::event::Status::Ignored, None);
                    }

                    let draw_w = self.output.bin_width as f32 * (base_scale * self.zoom);
                    let draw_h = self.output.total_height as f32 * (base_scale * self.zoom);
                    let bin_origin_x = bounds.x + (bounds.width - draw_w) / 2.0 + self.pan_x;
                    let bin_origin_y = bounds.y + (bounds.height - draw_h) / 2.0 + self.pan_y;

                    let bin_rect = iced::Rectangle {
                        x: bin_origin_x,
                        y: bin_origin_y,
                        width: draw_w,
                        height: draw_h,
                    };

                    if !bin_rect.contains(position) {
                        (canvas::event::Status::Captured, Some(Input::PanStart(position.x, position.y)))
                    } else {
                        if !self.animating && let Some(rect_idx) = self.find_rectangle_at_point(position.x, position.y, &bounds, scale, origin_x, origin_y, bin_h_units) {
                                return (canvas::event::Status::Captured, Some(Input::RectangleDragStart(rect_idx, position.x, position.y)));
                            }
                        
                        (canvas::event::Status::Ignored, None)
                    }
                } else {
                    (canvas::event::Status::Ignored, None)
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if self.is_panning {
                    (canvas::event::Status::Captured, Some(Input::PanEnd))
                } else if self.dragged_rect.is_some() {
                    (canvas::event::Status::Captured, Some(Input::RectangleDragEnd))
                } else {
                    (canvas::event::Status::Ignored, None)
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                let hovered = self.find_rectangle_at_point(position.x, position.y, &bounds, scale, origin_x, origin_y, bin_h_units);

                if self.is_panning {
                    (canvas::event::Status::Captured, Some(Input::PanMove(position.x, position.y)))
                } else if self.dragged_rect.is_some() {
                    (canvas::event::Status::Captured, Some(Input::RectangleDragMove(position.x, position.y)))
                } else {
                    (canvas::event::Status::Captured, Some(Input::RectangleHovered(hovered)))
                }
            }
            _ => (canvas::event::Status::Ignored, None)
        }
    }
}

     fn color_from_dimensions(x: i32,y: i32) -> Color {
         let mut h = 14695981039346656037u64;
         for v in [x as u32, y as u32] {
            h ^= v as u64;
            h = h.wrapping_mul(1099511628211);
        }

        let r = ((h & 0xFF) as f32) / 255.0;
        let g = (((h >> 8) & 0xFF) as f32) / 255.0;
        let b = (((h >> 16) & 0xFF) as f32) / 255.0;

        Color::from_rgb(r, g, b)   
     }
