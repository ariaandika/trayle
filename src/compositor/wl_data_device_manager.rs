use wl_data_device_manager::{CreateDataSource, GetDataDevice, Release};

use crate::compositor::prelude::*;

impl MessageHandler<CreateDataSource> for Compositor {
    fn handle(&mut self, req: Msg<CreateDataSource>, client: &mut ClientMut) {
        client.objects.create(req);
    }
}

impl MessageHandler<GetDataDevice> for Compositor {
    fn handle(&mut self, req: Msg<GetDataDevice>, client: &mut ClientMut) -> Result<(), ObjectError> {
        let _ = client.objects.get_with(req.seat)?;
        client.objects.create(req);
        self.seat.set_data_device(client.id.to_raw());
        Ok(())
    }
}

impl MessageHandler<Release> for Compositor {
    fn handle(&mut self, _: Msg<Release>, _: &mut ClientMut) {
        self.seat.clear_data_device();
    }
}
