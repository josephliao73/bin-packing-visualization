use serde::{Serialize, Deserialize};
use iced::widget::{text_editor};

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
    RectangleDragEnd(bool, bool, f32, f32),
    SnapAndAdjustHeight,
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
    pub total_height: f32,
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
