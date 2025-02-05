use smithay::{
    backend::input::{Event, InputBackend, InputEvent, KeyboardKeyEvent},
    input::keyboard::{FilterResult, KeysymHandle, ModifiersState},
    utils::SERIAL_COUNTER,
};
use xkbcommon::xkb::Keysym;

use crate::{state::BackendState, Trayle};


impl<B> Trayle<B> where B: BackendState + 'static {
    fn on_keyboard(&mut self, mods: &ModifiersState, handle: KeysymHandle) -> FilterResult<()> {
        let keysym = handle.modified_sym();
        tracing::debug!(?mods, keysym = ::xkbcommon::xkb::keysym_get_name(keysym), "keysym");

        match keysym {
            Keysym::Q if mods.logo => {
                tracing::info!("shutting down");
                self.running.store(false, std::sync::atomic::Ordering::SeqCst);
                FilterResult::Intercept(())
            }
            _ => FilterResult::Forward
        }
    }
    pub fn process_input_event<I>(&mut self, event: InputEvent<I>) where I: InputBackend {
        if let InputEvent::Keyboard { event } = event {
            let serial = SERIAL_COUNTER.next_serial();
            let time = event.time_msec();
            self.seat.get_keyboard().unwrap().input::<(), _>(
                self,
                event.key_code(),
                event.state(),
                serial,
                time,
                Self::on_keyboard,
            );
            return;
        }

        // match event {
        //     InputEvent::DeviceAdded { device } => todo!(),
        //     InputEvent::DeviceRemoved { device } => todo!(),
        //     InputEvent::Keyboard { event } => todo!(),
        //     InputEvent::PointerMotion { event } => todo!(),
        //     InputEvent::PointerMotionAbsolute { event } => todo!(),
        //     InputEvent::PointerButton { event } => todo!(),
        //     InputEvent::PointerAxis { event } => todo!(),
        //     InputEvent::GestureSwipeBegin { event } => todo!(),
        //     InputEvent::GestureSwipeUpdate { event } => todo!(),
        //     InputEvent::GestureSwipeEnd { event } => todo!(),
        //     InputEvent::GesturePinchBegin { event } => todo!(),
        //     InputEvent::GesturePinchUpdate { event } => todo!(),
        //     InputEvent::GesturePinchEnd { event } => todo!(),
        //     InputEvent::GestureHoldBegin { event } => todo!(),
        //     InputEvent::GestureHoldEnd { event } => todo!(),
        //     InputEvent::TouchDown { event } => todo!(),
        //     InputEvent::TouchMotion { event } => todo!(),
        //     InputEvent::TouchUp { event } => todo!(),
        //     InputEvent::TouchCancel { event } => todo!(),
        //     InputEvent::TouchFrame { event } => todo!(),
        //     InputEvent::TabletToolAxis { event } => todo!(),
        //     InputEvent::TabletToolProximity { event } => todo!(),
        //     InputEvent::TabletToolTip { event } => todo!(),
        //     InputEvent::TabletToolButton { event } => todo!(),
        //     InputEvent::SwitchToggle { event } => todo!(),
        //     InputEvent::Special(_) => todo!(),
        // }
    }
}

