use crate::wayland::{AsObjectId, FromObjectId, interface::AsInterface};

// ===== traits =====

pub trait WlObject: FromObjectId + AsObjectId + AsInterface {}

impl<O: FromObjectId + AsObjectId + AsInterface> WlObject for O {}
