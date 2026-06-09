use wayland::wl_data_source::Offer;

use crate::prelude::*;

impl RequestHandler<Offer<'_>> for Compositor {
    fn handle(&mut self, offer: Offer, client: &mut ClientMut) -> Result<(), WlError> {
        let _ = (offer, client);
        Ok(())
    }
}

