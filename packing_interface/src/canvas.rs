use iced::widget::canvas::{self};
use iced::widget::canvas::event::Event;
use iced::mouse;
use iced::{Color};
use crate::types::{Input, BinCanvas};

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
        let bin_h_units = self.output.total_height;

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

                let bin_rect = iced::Rectangle {
                    x: origin_x,
                    y: origin_y,
                    width: bin_w_units * scale,
                    height: bin_h_units * scale,
                };

                let is_inside = bin_rect.contains(Point {x: x_px, y: y_px})
                    && bin_rect.contains(Point {x: x_px + w, y: y_px})
                    && bin_rect.contains(Point {x: x_px, y: y_px + h})
                    && bin_rect.contains(Point {x: x_px + w, y: y_px + h});


                let mut intersects = false;


                for (idx, other) in self.output.placements.iter().enumerate() {
                    if idx == dragged_idx {
                        continue;
                    }

                    let other_width = other.width as f32 * scale;
                    let other_height = other.height as f32 * scale;
                    let other_x = origin_x + other.x * scale;
                    let other_y = origin_y + (bin_h_units - (other.y + other.height as f32)) * scale;

                    intersects = !(x_px + w <= other_x ||
                                     x_px >= other_x + other_width ||
                                     y_px + h <= other_y ||
                                     y_px >= other_y + other_height);

                    if intersects {
                        break;
                    }
                }

                println!("INSIDE BIN: {}", is_inside);
                println!("INTERSECTS: {}", intersects);

                let stroke_color = if is_inside && !intersects {
                    Color::from_rgb(0.0, 1.0, 0.0)
                } else {
                    Color::from_rgb(1.0, 0.0, 0.0)
                };
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
        let bin_h_units = self.output.total_height;

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
                    let draw_h = self.output.total_height * (base_scale * self.zoom);
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
                } else if let Some(dragged_idx) = self.dragged_rect {
                    if dragged_idx < self.output.placements.len() {
                        let p = &self.output.placements[dragged_idx];
                        let w = p.width as f32 * scale;
                        let h = p.height as f32 * scale;
                        let x_px = origin_x + p.x * scale + self.dragged_rect_offset_x;
                        let y_px = origin_y + (bin_h_units - (p.y + p.height as f32)) * scale + self.dragged_rect_offset_y;

                        let bin_rect = iced::Rectangle {
                            x: origin_x,
                            y: origin_y,
                            width: bin_w_units * scale,
                            height: bin_h_units * scale,
                        };

                        let is_inside = bin_rect.contains(iced::Point {x: x_px, y: y_px})
                            && bin_rect.contains(iced::Point {x: x_px + w, y: y_px})
                            && bin_rect.contains(iced::Point {x: x_px, y: y_px + h})
                            && bin_rect.contains(iced::Point {x: x_px + w, y: y_px + h});

                        let mut intersects = false;
                        for (idx, other) in self.output.placements.iter().enumerate() {
                            if idx == dragged_idx {
                                continue;
                            }

                            let other_width = other.width as f32 * scale;
                            let other_height = other.height as f32 * scale;
                            let other_x = origin_x + other.x * scale;
                            let other_y = origin_y + (bin_h_units - (other.y + other.height as f32)) * scale;

                            intersects = !(x_px + w <= other_x ||
                                         x_px >= other_x + other_width ||
                                         y_px + h <= other_y ||
                                         y_px >= other_y + other_height);

                            if intersects {
                                break;
                            }
                        }

                        let new_x = p.x + (self.dragged_rect_offset_x / scale);
                        let new_y = p.y - (self.dragged_rect_offset_y / scale);

                        (canvas::event::Status::Captured, Some(Input::RectangleDragEnd(is_inside, intersects, new_x, new_y)))
                    } else {
                        (canvas::event::Status::Ignored, None)
                    }
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

fn color_from_dimensions(x: i32, y: i32) -> Color {
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
