use wayland::wl_data_device_manager::{CreateDataSource, GetDataDevice, Release};

use crate::compositor::prelude::*;

impl RequestHandler<CreateDataSource> for Compositor {
    fn handle(
        &mut self,
        req: Operation<CreateDataSource>,
        client: &mut ClientMut,
    ) -> Result<(), WlError> {
        let _ = client.objects.create(req.data_source)?;
        Ok(())
    }
}

impl RequestHandler<GetDataDevice> for Compositor {
    fn handle(
        &mut self,
        req: Operation<GetDataDevice>,
        client: &mut ClientMut,
    ) -> Result<(), WlError> {
        let _ = client.objects.get_mut(&req.seat)?;
        let _ = client.objects.create(req.data_device)?;
        self.seat.set_data_device(client.id.to_raw());
        Ok(())
    }
}

impl RequestHandler<Release> for Compositor {
    fn handle(&mut self, _: Operation<Release>, _: &mut ClientMut) -> Result<(), WlError> {
        self.seat.clear_data_device();
        Ok(())
    }
}
