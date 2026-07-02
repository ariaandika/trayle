use wl_data_device_manager::{CreateDataSource, GetDataDevice, Release};

use crate::compositor::prelude::*;

impl MessageHandler<CreateDataSource> for Compositor {
    fn handle(
        &mut self,
        req: Msg<CreateDataSource>,
        client: &mut ClientMut,
    ) -> Result<(), WlError> {
        let _ = client.objects.create(req)?;
        Ok(())
    }
}

impl MessageHandler<GetDataDevice> for Compositor {
    fn handle(&mut self, req: Msg<GetDataDevice>, client: &mut ClientMut) -> Result<(), WlError> {
        let _ = client.objects.get_mut(req.seat)?;
        let _ = client.objects.create(req)?;
        self.seat.set_data_device(client.id.to_raw());
        Ok(())
    }
}

impl MessageHandler<Release> for Compositor {
    fn handle(&mut self, _: Msg<Release>, _: &mut ClientMut) -> Result<(), WlError> {
        self.seat.clear_data_device();
        Ok(())
    }
}
