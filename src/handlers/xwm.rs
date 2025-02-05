#![allow(unused_variables)]
use smithay::{
    utils::{Logical, Rectangle},
    xwayland::{
        xwm::{Reorder, ResizeEdge, XwmId},
        X11Surface, X11Wm, XwmHandler,
    },
};

use crate::{state::BackendState, Trayle};

impl<B> XwmHandler for Trayle<B> where B: BackendState + 'static {
    fn xwm_state(&mut self, xwm: XwmId) -> &mut X11Wm {
        self.xwm.as_mut().unwrap()
    }

    fn new_window(&mut self, xwm: XwmId, window: X11Surface) {
        // todo!()
    }

    fn new_override_redirect_window(&mut self, xwm: XwmId, window: X11Surface) {
        // todo!()
    }

    fn map_window_request(&mut self, xwm: XwmId, window: X11Surface) {
        // todo!()
    }

    fn mapped_override_redirect_window(&mut self, xwm: XwmId, window: X11Surface) {
        // todo!()
    }

    fn unmapped_window(&mut self, xwm: XwmId, window: X11Surface) {
        // todo!()
    }

    fn destroyed_window(&mut self, xwm: XwmId, window: X11Surface) {
        // todo!()
    }

    fn configure_request(
        &mut self,
        xwm: XwmId,
        window: X11Surface,
        x: Option<i32>,
        y: Option<i32>,
        w: Option<u32>,
        h: Option<u32>,
        reorder: Option<Reorder>,
    ) {
        // todo!()
    }

    fn configure_notify(
        &mut self,
        xwm: XwmId,
        window: X11Surface,
        geometry: Rectangle<i32, Logical>,
        above: Option<u32>,
    ) {
        // todo!()
    }

    fn resize_request(&mut self, xwm: XwmId, window: X11Surface, button: u32, resize_edge: ResizeEdge) {
        // todo!()
    }

    fn move_request(&mut self, xwm: XwmId, window: X11Surface, button: u32) {
        // todo!()
    }
}

