use wayland::wl_data_device_manager::{CreateDataSource, GetDataDevice, Release};

use crate::compositor::prelude::*;

impl RequestHandler<CreateDataSource> for Compositor {
    fn handle(&mut self, req: CreateDataSource, client: &mut ClientMut) -> Result<(), WlError> {
        let _ = client.objects.create(req.data_source)?;
        Ok(())
    }
}

impl RequestHandler<GetDataDevice> for Compositor {
    fn handle(&mut self, req: GetDataDevice, client: &mut ClientMut) -> Result<(), WlError> {
        let _ = client.objects.get_mut(req.seat)?;
        let _ = client.objects.create(req.data_device)?;
        self.seat.set_data_device(client.id);
        Ok(())
    }
}

impl RequestHandler<Release> for Compositor {
    fn handle(&mut self, _: Release, _: &mut ClientMut) -> Result<(), WlError> {
        self.seat.clear_data_device();
        // TODO: blocker: destructor trait
        // client.delete_id(object);
        Ok(())
    }
}
