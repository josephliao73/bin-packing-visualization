/*
 *
 * TODO: 
 * have it play animation in order
 * move the rectangles
 * implement rest of the list
 * compile to binary
 *
 */ 

use crate::config_parser::{create_input};
use iced::widget::{button, checkbox, column, container, row, text, text_input, text_editor, scrollable, slider};
use iced::{Element, Theme, Alignment, Length, Color, Font, time, Subscription};
use std::collections::{HashSet};
use iced::widget::canvas::{Canvas};
use crate::types::{Rectangle, Input, PackingApp, AlgorithmOutput, ParseOutput, BinCanvas };
use std::time::Duration;

impl Default for PackingApp {
    fn default() -> Self {
        Self {
            w_input: String::new(),
            n_input: String::new(),
            k_input: String::new(),
            autofile: false,
            rectangle_data: text_editor::Content::new(),
            error_message: None,
            algorithm_output: None,
            zoom: 1.0,
            visible_rects: 0,
            animating: false,
            animation_speed: 80.0,
            pan_x: 0.0,
            pan_y: 0.0,
            is_panning: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
            hovered_rect: None,
            dragged_rect: None,
            dragged_rect_offset_x: 0.0,
            dragged_rect_offset_y: 0.0,
        }
    }
}

impl PackingApp {
    pub fn update(&mut self, input: Input) {
        match input {
            Input::WChanged(w_input) => {
                self.w_input = w_input;
            }
            Input::NChanged(n_input) => {
                self.n_input = n_input;
            }
            Input::KChanged(k_input) => {
                self.k_input = k_input;
            }
            Input::AutofillChanged(autofile) => {
                self.autofile = autofile;
            }
            Input::ImportPressed => {
                if let Some(file_path) = rfd::FileDialog::new()
                    .add_filter("Supported files", &["txt", "in", "csv"])
                    .pick_file()
                {
                    if let Ok(contents) = std::fs::read_to_string(&file_path) {
                        println!("Loaded file: {:?}", file_path);
                        self.rectangle_data = text_editor::Content::with_text(&contents);
                        self.error_message = None;
                    } else {
                        self.error_message = Some(format!("Error reading file: {:?}", file_path));
                    }
                }
            }
            Input::ImportOutputJsonPressed => {
                if let Some(file_path) = rfd::FileDialog::new()
                    .add_filter("JSON files", &["json"])
                    .pick_file()
                {
                    match std::fs::read_to_string(&file_path) {
                        Ok(contents) => {
                            match serde_json::from_str::<AlgorithmOutput>(&contents) {
                                Ok(output) => {
                                    self.algorithm_output = Some(output);
                                    self.visible_rects = 0;
                                    self.animating = true;
                                    self.error_message = Some("✓ Successfully imported algorithm output".to_string());
                                }
                                Err(e) => {
                                    self.error_message = Some(format!("Error parsing JSON: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Error reading file: {}", e));
                        }
                    }
                }
            }
            Input::RectangleDataAction(action) => {
                self.rectangle_data.perform(action);
            }

            Input::ExportAlgorithmInput => {
                match self.parse_rectangles() {
                    Ok(output) => {
                        let json = match serde_json::to_string_pretty(&create_input(&output)) {
                            Ok(j) => j,
                            Err(e) => {
                                self.error_message = Some(format!("Failed to serialize JSON: {e}"));
                                return;
                            }
                        };

                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("JSON file", &["json"])
                            .set_file_name("algorithm_input.json")
                            .save_file()
                        {
                            match std::fs::write(&path, json) {
                                Ok(_) => {
                                    self.error_message = Some(format!(
                                        "✓ Successfully parsed {} rectangles and saved to {}",
                                        output.rects.len(),
                                        path.display()
                                    ));
                                }
                                Err(e) => {
                                    self.error_message = Some(format!("Failed to save file: {e}"));
                                }
                            }
                        } else {
                            self.error_message = Some("Export cancelled.".to_string());
                        }
                    }
                    Err(errors) => {
                        self.error_message = Some(errors.join("\n"));
                    }
                }
            }
           Input::ZoomChanged(factor) => {
               let new_zoom = (self.zoom * factor).clamp(0.1, 10.0);
               self.zoom = new_zoom;
            }
            Input::AnimationSpeedChanged(speed) => {
                self.animation_speed = speed.clamp(10.0, 500.0);
            }
            Input::PanStart(x, y) => {
                self.is_panning = true;
                self.last_mouse_x = x;
                self.last_mouse_y = y;
            }
            Input::PanMove(x, y) => {
                if self.is_panning {
                    let dx = x - self.last_mouse_x;
                    let dy = y - self.last_mouse_y;
                    self.pan_x += dx;
                    self.pan_y += dy;
                    self.last_mouse_x = x;
                    self.last_mouse_y = y;
                }
                self.hovered_rect = None;
            }
            Input::PanEnd => {
                self.is_panning = false;
            }
            Input::RectangleHovered(rect_idx) => {
                self.hovered_rect = rect_idx;
            }
            Input::RectangleDragStart(rect_idx, x, y) => {
                self.dragged_rect = Some(rect_idx);
                self.last_mouse_x = x;
                self.last_mouse_y = y;
                self.dragged_rect_offset_x = 0.0;
                self.dragged_rect_offset_y = 0.0;
            }
            Input::RectangleDragMove(x, y) => {
                if self.dragged_rect.is_some() {
                    let dx = x - self.last_mouse_x;
                    let dy = y - self.last_mouse_y;
                    self.dragged_rect_offset_x += dx;
                    self.dragged_rect_offset_y += dy;
                    self.last_mouse_x = x;
                    self.last_mouse_y = y;
                }
            }
            Input::RectangleDragEnd => {
                self.dragged_rect = None;
                self.dragged_rect_offset_x = 0.0;
                self.dragged_rect_offset_y = 0.0;
            }
            Input::Tick => {
                if let Some(output) = &self.algorithm_output {
                    let total = output.placements.len();
                    if self.visible_rects < total {
                        self.visible_rects += 1;
                    } else {
                        self.animating = false;
                    }
                } else {
                    self.animating = false;
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Input> {
        if self.animating {
            time::every(Duration::from_millis(self.animation_speed as u64)).map(|_| Input::Tick)
        } else {
            Subscription::none()
        }
    }
    
    fn parse_rectangles(&self) -> Result<ParseOutput, Vec<String>> {
        let text = self.rectangle_data.text();
        let mut rectangles = Vec::new();
        let mut errors = Vec::new();
        let mut w_val: i32 = -1;
        let mut total: i32 = 0;
        let mut Ntemp: i32 = -1;
        let mut Ktemp: i32 = -1;
        let mut set: HashSet<(i32, i32)> = HashSet::new();
        let mut min_height: i32 = i32::MAX;
        let mut max_height: i32 = i32::MIN;

        if self.w_input.is_empty() {
            errors.push("Enter a value for the width of the bin".to_string());
            return Err(errors);
        }

        if let Ok(w) = self.w_input.parse::<i32>() {
            if w < 0 {
                errors.push("Emter a positive value for the width of the bin".to_string());
                return Err(errors);
            }
            w_val = w;
        } else {
            errors.push("Enter an integer value for the width of the bin".to_string());
            return Err(errors);           
        }

        if !self.n_input.is_empty() {
            if let Ok(n) = self.n_input.parse::<i32>() {
                if n < 0 {
                    errors.push("Enter an integer value for the quantity of rectangles".to_string());
                    return Err(errors);
                }
                Ntemp = n;
            } else {
                errors.push("Enter an integer value for the quantity of rectangles".to_string());
            }
        }


        if !self.k_input.is_empty() {
            if let Ok(k) = self.k_input.parse::<i32>() {
                if k < 0 {
                    errors.push("Enter an integer value for the types f rectangles".to_string());
                    return Err(errors);
                }
                Ktemp = k;
            } else {
                errors.push("Enter an integer value for the types of rectangles".to_string());
            }
        }


        if !errors.is_empty() {
            return Err(errors)
        }

        for (line_num, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() { continue; }
            
            let parts: Vec<&str> = line.split_whitespace().collect();


            if parts.len() != 3 {
                errors.push(format!("Line {}: Expected exactly 3 space-separated values (X Y Q), found {}", 
                    line_num + 1, parts.len()));
                continue;
            }

            let x_result = parts[0].parse::<i32>();
            let y_result = parts[1].parse::<i32>();
            let q_result = parts[2].parse::<i32>();

            match (x_result, y_result, q_result) {
                (Ok(x), Ok(y), Ok(q)) => {
                    if x <= 0 || y <= 0 || q <= 0 {
                        errors.push(format!("Line {}: X, Y, and Q must be positive", line_num + 1));
                        continue;
                    }

                    min_height = i32::min(y, min_height);
                    max_height = i32::max(y, max_height);

                    total += q;
                    if x > w_val {
                        errors.push(format!("Line {}: '{}' is greater than the width {}", line_num+1, parts[0], w_val));
                    } else {
                        rectangles.push(Rectangle { 
                            width: x, 
                            height: y, 
                            quantity: q 
                        });
                        set.insert((x, y));
                    }
                }
                (Err(_), _, _) => {
                    errors.push(format!("Line {}: '{}' is not a valid integer for X", 
                        line_num + 1, parts[0]));
                }
                (_, Err(_), _) => {
                    errors.push(format!("Line {}: '{}' is not a valid integer for Y", 
                        line_num + 1, parts[1]));
                }
                (_, _, Err(_)) => {
                    errors.push(format!("Line {}: '{}' is not a valid integer for Q", 
                        line_num + 1, parts[2]));
                }
            }
        }

        if !self.autofile && Ntemp != -1 && Ntemp != total {
            errors.push(format!("The quantity of rectangles is NOT the same as the input. {} rectangles found, {} expected.", total, Ntemp));
        }

        if !self.autofile && Ktemp != -1 && Ktemp != set.len() as i32 {
            errors.push(format!("The number of types of rectangles is NOT the same as the input. {} types found, {} expected.", set.len(), Ktemp));
        }

        if self.autofile {
            let actual_n = total;
            let actual_k = set.len() as i32;
            
            if Ktemp != -1 && actual_k > Ktemp {
                errors.push(format!("The number of types of rectangles is greater than the input. {} types found, {} expected.", actual_k, Ktemp));
            }
            
            if Ntemp != -1 && Ktemp != -1 {
                let n_difference = Ntemp - actual_n;
                let k_difference = Ktemp - actual_k;
                
                if n_difference > 0 && k_difference > n_difference {
                    errors.push(format!(
                        "Autofill impossible: Need to add {} rectangles but only {} type slots available. \
                        (Input N={}, Actual N={}, Input K={}, Actual K={})",
                        n_difference, k_difference, Ntemp, actual_n, Ktemp, actual_k
                    ));
                }
                
                if n_difference < 0 {
                    errors.push(format!(
                        "Autofill impossible: Already have {} rectangles but input N={} (cannot remove rectangles)",
                        actual_n, Ntemp
                    ));
                }
            }
        }
        
        if errors.is_empty() {
            Ok(ParseOutput {width: w_val, quantity: Ntemp, types: Ktemp, autofill: self.autofile, rects: rectangles, input_types: set.len() as i32,min_height, max_height})
        } else {
            Err(errors)
        }
    }

    pub fn view(&self) -> Element<'_, Input> {
        let nerd_font = Font::with_name("JetBrainsMono Nerd Font");
        
        let title = text("Rectangle Packing Configuration")
            .size(22)
            .font(nerd_font);
        
        let header = column![
            title,
        ]
        .spacing(4);
        
        let w_label = text("Bin Width")
            .size(12)
            .font(nerd_font)
            .style(|_theme: &Theme| {
                text::Style {
                    color: Some(Color::from_rgb(0.75, 0.75, 0.8)),
                }
            });
        
        let w_input = text_input("e.g., 100", &self.w_input)
            .on_input(Input::WChanged)
            .size(13)
            .padding(10)
            .width(Length::Fill)
            .font(nerd_font);
        
        let w_input_container = container(w_input)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.08, 0.08, 0.1).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.25, 0.25, 0.3),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }
            });
        
        let n_label = text("Number of Rectangles")
            .size(12)
            .font(nerd_font)
            .style(|_theme: &Theme| {
                text::Style {
                    color: Some(Color::from_rgb(0.75, 0.75, 0.8)),
                }
            });
        
        let n_input = text_input("Optional", &self.n_input)
            .on_input(Input::NChanged)
            .size(13)
            .padding(10)
            .width(Length::Fill)
            .font(nerd_font);
        
        let n_input_container = container(n_input)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.08, 0.08, 0.1).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.25, 0.25, 0.3),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }
            });
        
        let k_label = text("Number of Rectangle Types")
            .size(12)
            .font(nerd_font)
            .style(|_theme: &Theme| {
                text::Style {
                    color: Some(Color::from_rgb(0.75, 0.75, 0.8)),
                }
            });
        
        let k_input = text_input("Optional", &self.k_input)
            .on_input(Input::KChanged)
            .size(13)
            .padding(10)
            .width(Length::Fill)
            .font(nerd_font);
        
        let k_input_container = container(k_input)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.08, 0.08, 0.1).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.25, 0.25, 0.3),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }
            });
        
        let autofill_checkbox = if self.n_input.is_empty() {
            checkbox("Autofill remaining values", self.autofile)
                .size(10)
                .font(nerd_font)
        } else {
            checkbox("Autofill remaining values", self.autofile)
                .on_toggle(Input::AutofillChanged)
                .size(10)
                .font(nerd_font)
        };
        
        let autofill_container = container(autofill_checkbox)
            .padding([8, 0]);
        
        let divider = container(
            container(text(""))
                .width(Length::Fill)
                .height(1)
                .style(|_theme: &Theme| {
                    container::Style {
                        background: Some(Color::from_rgb(0.2, 0.2, 0.25).into()),
                        ..Default::default()
                    }
                })
        );
        
        let import_button = button(
            container(
                text("Import Configuration")
                    .size(13)
                    .font(nerd_font)
            )
            .center_x(Length::Fill)
        )
        .on_press(Input::ImportPressed)
        .padding(10)
        .width(Length::Fill)
        .style(|_theme: &Theme, status| {
            let base_bg = Color::from_rgb(0.18, 0.2, 0.24);
            let hover_bg = Color::from_rgb(0.22, 0.24, 0.28);
            
            button::Style {
                background: Some(match status {
                    button::Status::Hovered => hover_bg.into(),
                    _ => base_bg.into(),
                }),
                border: iced::Border {
                    color: Color::from_rgb(0.3, 0.35, 0.4),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                text_color: Color::from_rgb(0.85, 0.85, 0.9),
                ..Default::default()
            }
        });
        
        
        let rectangle_label = text("Rectangle Values")
            .size(12)
            .font(nerd_font)
            .style(|_theme: &Theme| {
                text::Style {
                    color: Some(Color::from_rgb(0.75, 0.75, 0.8)),
                }
            });
        
        let rectangle_hint = text("Format: X Y Q (space-separated)")
            .size(10)
            .font(nerd_font)
            .style(|_theme: &Theme| {
                text::Style {
                    color: Some(Color::from_rgb(0.5, 0.5, 0.55)),
                }
            });
        
        let editor_header = column![
            rectangle_label,
            rectangle_hint,
        ]
        .spacing(2);
        
        let rectangle_editor = text_editor(&self.rectangle_data)
            .on_action(Input::RectangleDataAction)
            .height(225)
            .padding(12)
            .size(13)
            .font(nerd_font);
        
        let editor_container = container(rectangle_editor)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.06, 0.06, 0.08).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.25, 0.25, 0.3),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }
            });
        
        let text_content = self.rectangle_data.text();
        let total_lines = if text_content.is_empty() || text_content == "\n" { 
            1 
        } else { 
            text_content.chars().filter(|&c| c == '\n').count() + 1
        };

        let cursor = self.rectangle_data.cursor_position();
        let cursor_pos = cursor.0.min(text_content.len());
        let lines_before_cursor = if cursor_pos == 0 || text_content.is_empty() {
            1
        } else {
            text_content[..cursor_pos]
                .chars()
                .filter(|&c| c == '\n')
                .count() + 1
        };

        let line_info = text(format!("Line {} of {}", lines_before_cursor, total_lines))
            .size(10)
            .font(nerd_font)
            .style(|_theme: &Theme| {
                text::Style {
                    color: Some(Color::from_rgb(0.5, 0.5, 0.55)),
                }
            });
        
        let editor_with_info = column![
            editor_container,
            line_info,
        ]
        .spacing(6);
        
        let export_button = button(
            container(
                text("Export Algorithm Input")
                    .size(13)
                    .font(nerd_font)
            )
            .center_x(Length::Fill)
        )
        .on_press(Input::ExportAlgorithmInput)
        .padding(12)
        .width(Length::Fill)
        .style(|_theme: &Theme, status| {
            let base_bg = Color::from_rgb(0.2, 0.4, 0.65);
            let hover_bg = Color::from_rgb(0.25, 0.45, 0.7);
            
            button::Style {
                background: Some(match status {
                    button::Status::Hovered => hover_bg.into(),
                    _ => base_bg.into(),
                }),
                border: iced::Border {
                    color: Color::from_rgb(0.3, 0.5, 0.75),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                text_color: Color::from_rgb(1.0, 1.0, 1.0),
                ..Default::default()
            }
        });
        
        let message_display = if let Some(msg) = &self.error_message {
            let is_success = msg.starts_with("✓");
            let (bg_color, border_color, text_color) = if is_success {
                (
                    Color::from_rgb(0.1, 0.25, 0.15),
                    Color::from_rgb(0.2, 0.6, 0.3),
                    Color::from_rgb(0.4, 0.9, 0.5)
                )
            } else {
                (
                    Color::from_rgb(0.25, 0.1, 0.1),
                    Color::from_rgb(0.7, 0.2, 0.2),
                    Color::from_rgb(1.0, 0.5, 0.5)
                )
            };
            
            container(
                scrollable(
                    text(msg)
                        .size(11)
                        .font(nerd_font)
                        .style(move |_theme: &Theme| {
                            text::Style {
                                color: Some(text_color),
                            }
                        })
                )
                .height(Length::Fixed(50.0))
            )
            .padding(12)
            .width(Length::Fill)
            .style(move |_theme: &Theme| {
                container::Style {
                    background: Some(bg_color.into()),
                    border: iced::Border {
                        color: border_color,
                        width: 1.5,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }
            })
        } else {
            container(text("").size(1))
        };

        let import_output_json_button = button(
            container(
                text("Import Output JSON")
                    .size(13)
                    .font(nerd_font)
            )
            .center_x(Length::Fill)
        )
        .on_press(Input::ImportOutputJsonPressed)
        .padding(12)
        .width(Length::Fill)
        .style(|_theme: &Theme, status| {
            let base_bg = Color::from_rgb(0.65, 0.4, 0.2);
            let hover_bg = Color::from_rgb(0.7, 0.45, 0.25);
            
            button::Style {
                background: Some(match status {
                    button::Status::Hovered => hover_bg.into(),
                    _ => base_bg.into(),
                }),
                border: iced::Border {
                    color: Color::from_rgb(0.75, 0.5, 0.3),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                text_color: Color::from_rgb(1.0, 1.0, 1.0),
                ..Default::default()
            }
        });

        let animation_speed_label = text("Animation Speed (ms)")
            .size(12)
            .font(nerd_font)
            .style(|_theme: &Theme| {
                text::Style {
                    color: Some(Color::from_rgb(0.75, 0.75, 0.8)),
                }
            });

        let animation_speed_slider = slider(10.0..=500.0, self.animation_speed, Input::AnimationSpeedChanged)
            .width(Length::Fill)
            .step(10.0);

        let animation_speed_value = text(format!("{:.0}ms", self.animation_speed))
            .size(11)
            .font(nerd_font)
            .style(|_theme: &Theme| {
                text::Style {
                    color: Some(Color::from_rgb(0.65, 0.85, 0.95)),
                }
            });

        let animation_speed_container = container(
            column![
                animation_speed_label,
                column![].height(4),
                row![
                    animation_speed_slider,
                    column![].width(8),
                    animation_speed_value,
                ].spacing(0).align_y(Alignment::Center),
            ].spacing(4)
        );

        let import_output_json_container = container(import_output_json_button)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.1, 0.1, 0.12).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.2, 0.2, 0.25),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }
            });

let visualization_content = if let Some(output) = &self.algorithm_output {
    let canvas = Canvas::new(BinCanvas {
            output,
            zoom: self.zoom,
            visible_count: self.visible_rects,
            pan_x: self.pan_x,
            pan_y: self.pan_y,
            hovered_rect: self.hovered_rect,
            is_panning: self.is_panning,
            dragged_rect: self.dragged_rect,
            dragged_rect_offset_x: self.dragged_rect_offset_x,
            dragged_rect_offset_y: self.dragged_rect_offset_y,
            animating: self.animating,
        })
        .width(Length::Fill)
        .height(Length::Fill);
    let height_display = container(
        text(format!("Total Height: {}", output.total_height))
            .size(14)
            .font(nerd_font)
            .style(|_theme: &Theme| {
                text::Style {
                    color: Some(Color::from_rgb(0.9, 0.9, 0.95)),
                }
            })
    )
    .padding(8)
    .style(|_theme: &Theme| {
        container::Style {
            background: Some(Color::from_rgb(0.2, 0.2, 0.3).into()),
            border: iced::Border {
                color: Color::from_rgb(0.4, 0.4, 0.6),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    });

    let dimensions_display = if let Some(hovered_idx) = self.hovered_rect {
        if hovered_idx < output.placements.len() {
            let placement = &output.placements[hovered_idx];
            container(
                text(format!("Width: {} | Height: {}", placement.width, placement.height))
                    .size(14)
                    .font(nerd_font)
                    .style(|_theme: &Theme| {
                        text::Style {
                            color: Some(Color::from_rgb(0.9, 0.9, 0.95)),
                        }
                    })
            )
            .padding(8)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.2, 0.2, 0.3).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.4, 0.8, 1.0),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }
            })
        } else {
            container(text("").size(14))
        }
    } else {
        container(text("Hover over a rectangle to see dimensions").size(14).style(|_theme: &Theme| {
            text::Style {
                color: Some(Color::from_rgb(0.6, 0.6, 0.7)),
            }
        }))
        .padding(8)
        .style(|_theme: &Theme| {
            container::Style {
                background: Some(Color::from_rgb(0.15, 0.15, 0.2).into()),
                border: iced::Border {
                    color: Color::from_rgb(0.3, 0.3, 0.4),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        })
    };

    column![
        container(canvas)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
        column![].height(16),
        row![
            dimensions_display.width(Length::Fill),
            height_display.width(Length::Fill),
        ]
        .spacing(8)
        .width(Length::Fill),
    ]
    .align_x(Alignment::Center)
    .spacing(8)
        } else {
            column![
                text("Visualization Area")
                    .size(16)
                    .font(nerd_font)
                    .style(|_theme: &Theme| {
                        text::Style {
                            color: Some(Color::from_rgb(0.5, 0.5, 0.55)),
                        }
                    }),
                column![].height(8),
                text("Import Output JSON to see the packing result")
                    .size(12)
                    .font(nerd_font)
                    .style(|_theme: &Theme| {
                        text::Style {
                            color: Some(Color::from_rgb(0.4, 0.4, 0.45)),
                        }
                    }),
            ]
            .align_x(Alignment::Center)
            .width(Length::Fill)
        };
        
        let stats_display = if let Some(output) = &self.algorithm_output {
            let rect_count_text = text(format!("Rectangles: {}/{}", self.visible_rects, output.placements.len()))
                .size(11)
                .font(nerd_font)
                .style(|_theme: &Theme| {
                    text::Style {
                        color: Some(Color::from_rgb(0.65, 0.85, 0.95)),
                    }
                });

            let zoom_text = text(format!("Zoom: {:.0}%", self.zoom * 100.0))
                .size(11)
                .font(nerd_font)
                .style(|_theme: &Theme| {
                    text::Style {
                        color: Some(Color::from_rgb(0.65, 0.85, 0.95)),
                    }
                });

            container(
                row![
                    rect_count_text,
                    column![].width(Length::Fill),
                    zoom_text,
                ].spacing(8).width(Length::Fill)
            )
            .padding(8)
            .width(Length::Fill)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.12, 0.12, 0.15).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.25, 0.25, 0.3),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }
            })
        } else {
            container(text("").size(1))
        };

        let input_section = column![
            header,
            column![].height(20),
            column![
                w_label,
                column![].height(4),
                w_input_container,
            ].spacing(0),
            column![].height(14),
            column![
                n_label,
                column![].height(4),
                n_input_container,
            ].spacing(0),
            column![].height(14),
            column![
                k_label,
                column![].height(4),
                k_input_container,
            ].spacing(0),
            column![].height(10),
            autofill_container,
            column![].height(16),
            divider,
            column![].height(16),
            row![
                import_button,
            ].spacing(8),
            column![].height(20),
            editor_header,
            column![].height(6),
            editor_with_info,
            column![].height(14),
            export_button,
            column![].height(12),
            message_display,
        ]
        .spacing(0)
        .padding(24)
        .align_x(Alignment::Start);

        let input_container = container(input_section)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.1, 0.1, 0.12).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.2, 0.2, 0.25),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }
            });

        let output_section = column![
            import_output_json_container,
            column![].height(12),
            animation_speed_container,
            column![].height(12),
            stats_display,
        ]
        .spacing(0)
        .padding(24)
        .align_x(Alignment::Start);

        let output_container = container(output_section)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.1, 0.1, 0.12).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.2, 0.2, 0.25),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }
            });

        let left_panel = column![
            input_container,
            column![].height(16),
            output_container,
        ]
        .spacing(0)
        .width(Length::Fixed(440.0));

        let left_panel_container = container(left_panel)
            .width(Length::Fixed(440.0));
        
        let visualization = container(visualization_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(30)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.08, 0.08, 0.1).into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.2, 0.2, 0.25),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }
            });
        
        let main_content = row![
            left_panel_container,
            visualization,
        ]
        .spacing(16);
        
        container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(16)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb(0.06, 0.06, 0.08).into()),
                    text_color: Some(Color::from_rgb(0.9, 0.9, 0.92)),
                    ..Default::default()
                }
            })
            .into()
    }
}
