use wl_data_source::{Destroy, Offer, SetActions};

use crate::compositor::prelude::*;

impl MessageHandler<Offer<'_>> for Compositor {
    fn handle(&mut self, _msg: Msg<Offer>, _client: &mut ClientMut) -> Result<(), WlError> {
        Err(WlError::NotYetImplemented)
    }
}

impl MessageHandler<Destroy> for Compositor {
    fn handle(&mut self, _msg: Msg<Destroy>, _client: &mut ClientMut) -> Result<(), WlError> {
        Err(WlError::NotYetImplemented)
    }
}

impl MessageHandler<SetActions> for Compositor {
    fn handle(&mut self, _msg: Msg<SetActions>, _client: &mut ClientMut) -> Result<(), WlError> {
        Err(WlError::NotYetImplemented)
    }
}
