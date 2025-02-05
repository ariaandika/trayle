use anyhow::Result;
use smithay::backend::{allocator::Fourcc, renderer::Color32F};


pub const SUPPORTED_FORMATS_8BIT_ONLY: &[Fourcc] = &[Fourcc::Abgr8888, Fourcc::Argb8888];
pub const SUPPORTED_FORMATS: &[Fourcc] = &[
    Fourcc::Abgr2101010,
    Fourcc::Argb2101010,
    Fourcc::Abgr8888,
    Fourcc::Argb8888,
];

pub struct Config {
    pub clear_color: Color32F,
    pub kb_repeat_delay: i32,
    pub kb_repeat_rate: i32,
    pub disable_direct_10bit: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            clear_color: Color32F::new(0.8, 0.8, 0.9, 1.0),
            kb_repeat_delay: 160,
            kb_repeat_rate: 50,
            disable_direct_10bit: env("TRAYLE_DISABLE_DIRECT_10BIT"),
        }
    }
}

impl Config {
    pub fn setup() -> Result<Config> {
        Ok(Config::default())
    }
}

fn env(key: &str) -> bool {
    matches!(std::env::var(key).as_deref(),Ok("1"))
}

