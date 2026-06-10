use wayland::wl_data_device_manager::{CreateDataSource, GetDataDevice};

use crate::prelude::*;

impl RequestHandler<CreateDataSource> for Compositor {
    fn handle(&mut self, req: CreateDataSource, client: &mut ClientMut) -> Result<(), WlError> {
        let data_source = req.data_source.create();
        client.objects.insert(&data_source)
    }
}

impl RequestHandler<GetDataDevice> for Compositor {
    fn handle(&mut self, req: GetDataDevice, client: &mut ClientMut) -> Result<(), WlError> {
        let data_device = req.data_device.create();
        let Some(object) = client.objects.get_mut(req.seat) else {
            return Err(WlError::UnknownObject);
        };
        let Interface::WlSeat = object.interface() else {
            return Err(WlError::UnknownObject);
        };
        client.objects.insert(&data_device)
    }
}

