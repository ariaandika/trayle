use wl_data_device_manager::{CreateDataSource, GetDataDevice};

use crate::compositor::prelude::*;

pub fn create_source(req: Msg<CreateDataSource>, client: &mut ClientMut) {
    client.objects.create(req);
}

pub fn get_data_device(
    req: Msg<GetDataDevice>,
    client: &mut ClientMut,
    seat: &mut crate::seat::Seat,
) -> Result<(), UnknownId> {
    let _ = client.objects.get_with(req.seat)?;
    client.objects.create(req);
    seat.set_data_device(client.id.to_raw());
    Ok(())
}
