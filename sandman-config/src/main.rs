mod app;

use crate::app::SandmanConfigApp;
use iced::{Sandbox, Settings};

// https://leafheap.com/articles/iced-tutorial-version-0-12

pub fn main() -> iced::Result {
    SandmanConfigApp::run(Settings::default())
}
