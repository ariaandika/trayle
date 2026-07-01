use wl_data_source::{Offer, Destroy, SetActions};

use crate::compositor::prelude::*;

impl RequestHandler<Offer<'_>> for Compositor {
    fn handle(&mut self, req: Operation<Offer>, client: &mut ClientMut) -> Result<(), WlError> {
        Err(self.todo(req, client))
    }
}

impl RequestHandler<Destroy> for Compositor {
    fn handle(&mut self, req: Operation<Destroy>, client: &mut ClientMut) -> Result<(), WlError> {
        Err(self.todo(req, client))
    }
}

impl RequestHandler<SetActions> for Compositor {
    fn handle(
        &mut self,
        req: Operation<SetActions>,
        client: &mut ClientMut,
    ) -> Result<(), WlError> {
        Err(self.todo(req, client))
    }
}
