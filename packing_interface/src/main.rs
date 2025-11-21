mod config_parser;
mod ui;
mod types;
mod canvas;

use crate::types::{PackingApp};

fn main() -> iced::Result {

    iced::application("Packing App", PackingApp::update, PackingApp::view)
        .theme(|_| iced::Theme::TokyoNight)
        .subscription(PackingApp::subscription)
        .run_with(|| (PackingApp::default(), iced::Task::none()))
}
