#![allow(unused_variables)]
use std::borrow::Cow;

use crate::Trayle;
use smithay::{
    backend::input::KeyState,
    input::{
        keyboard::{KeyboardTarget, KeysymHandle, ModifiersState},
        pointer::{
            AxisFrame, ButtonEvent, GestureHoldBeginEvent, GestureHoldEndEvent,
            GesturePinchBeginEvent, GesturePinchEndEvent, GesturePinchUpdateEvent,
            GestureSwipeBeginEvent, GestureSwipeEndEvent, GestureSwipeUpdateEvent, MotionEvent,
            PointerTarget, RelativeMotionEvent,
        },
        touch::{
            DownEvent, MotionEvent as TouchMotionEvent, OrientationEvent, ShapeEvent, TouchTarget,
            UpEvent,
        },
        Seat, SeatHandler, SeatState,
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{IsAlive, Serial},
    wayland::seat::WaylandFocus,
};

#[derive(Debug,Clone,PartialEq)]
pub enum KeyboardFocusTarget {
    
}

impl IsAlive for KeyboardFocusTarget {
    fn alive(&self) -> bool {
        true
        // todo!()
    }
}

impl WaylandFocus for KeyboardFocusTarget {
    fn wl_surface(&self) -> Option<Cow<'_, WlSurface>> {
        None
        // todo!()
    }
}

impl<B> KeyboardTarget<Trayle<B>> for KeyboardFocusTarget {
    fn enter(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, keys: Vec<KeysymHandle<'_>>, serial: Serial) {
        // todo!()
    }

    fn leave(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, serial: Serial) {
        // todo!()
    }

    fn key(
        &self,
        seat: &Seat<Trayle<B>>,
        data: &mut Trayle<B>,
        key: KeysymHandle<'_>,
        state: KeyState,
        serial: Serial,
        time: u32,
    ) {
        // todo!()
    }

    fn modifiers(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, modifiers: ModifiersState, serial: Serial) {
        // todo!()
    }
}

#[derive(Debug,Clone,PartialEq)]
pub enum PointerFocusTarget {
    
}

impl IsAlive for PointerFocusTarget {
    fn alive(&self) -> bool {
        true
        // todo!()
    }
}

impl WaylandFocus for PointerFocusTarget {
    fn wl_surface(&self) -> Option<Cow<'_, WlSurface>> {
        None
        // todo!()
    }
}

impl<B> PointerTarget<Trayle<B>> for PointerFocusTarget {
    fn enter(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, event: &MotionEvent) {
        // todo!()
    }

    fn motion(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, event: &MotionEvent) {
        // todo!()
    }

    fn relative_motion(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, event: &RelativeMotionEvent) {
        // todo!()
    }

    fn button(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, event: &ButtonEvent) {
        // todo!()
    }

    fn axis(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, frame: AxisFrame) {
        // todo!()
    }

    fn frame(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>) {
        // todo!()
    }

    fn gesture_swipe_begin(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, event: &GestureSwipeBeginEvent) {
        // todo!()
    }

    fn gesture_swipe_update(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, event: &GestureSwipeUpdateEvent) {
        // todo!()
    }

    fn gesture_swipe_end(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, event: &GestureSwipeEndEvent) {
        // todo!()
    }

    fn gesture_pinch_begin(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, event: &GesturePinchBeginEvent) {
        // todo!()
    }

    fn gesture_pinch_update(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, event: &GesturePinchUpdateEvent) {
        // todo!()
    }

    fn gesture_pinch_end(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, event: &GesturePinchEndEvent) {
        // todo!()
    }

    fn gesture_hold_begin(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, event: &GestureHoldBeginEvent) {
        // todo!()
    }

    fn gesture_hold_end(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, event: &GestureHoldEndEvent) {
        // todo!()
    }

    fn leave(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, serial: Serial, time: u32) {
        // todo!()
    }
}

#[derive(Debug,Clone,PartialEq)]
pub enum TouchFocusTarget {
    
}

impl IsAlive for TouchFocusTarget {
    fn alive(&self) -> bool {
        true
        // todo!()
    }
}

impl WaylandFocus for TouchFocusTarget {
    fn wl_surface(&self) -> Option<Cow<'_, WlSurface>> {
        None
        // todo!()
    }
}

impl<B> TouchTarget<Trayle<B>> for TouchFocusTarget {
    fn down(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, event: &DownEvent, seq: Serial) {
        // todo!()
    }

    fn up(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, event: &UpEvent, seq: Serial) {
        // todo!()
    }

    fn motion(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, event: &TouchMotionEvent, seq: Serial) {
        // todo!()
    }

    fn frame(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, seq: Serial) {
        // todo!()
    }

    fn cancel(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, seq: Serial) {
        // todo!()
    }

    fn shape(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, event: &ShapeEvent, seq: Serial) {
        // todo!()
    }

    fn orientation(&self, seat: &Seat<Trayle<B>>, data: &mut Trayle<B>, event: &OrientationEvent, seq: Serial) {
        // todo!()
    }
}

smithay::delegate_seat!(@<B: 'static> Trayle<B>);

impl<B> SeatHandler for Trayle<B> {
    type KeyboardFocus = KeyboardFocusTarget;
    type PointerFocus = PointerFocusTarget;
    type TouchFocus = TouchFocusTarget;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }
}

