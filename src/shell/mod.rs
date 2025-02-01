use std::cell::RefCell;

use elements::WindowElement;
use smithay::utils::IsAlive;

pub mod elements;
pub mod utils;
pub mod ssd;

#[derive(Default)]
pub struct FullscreenSurface(RefCell<Option<WindowElement>>);

impl FullscreenSurface {
    pub fn get(&self) -> Option<WindowElement> {
        let mut window = self.0.borrow_mut();
        if window.as_ref().map(|w|!w.alive()).unwrap_or(false) {
            *window = None;
        }
        window.clone()
    }
}


