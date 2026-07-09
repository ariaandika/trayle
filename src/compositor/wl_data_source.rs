use wl_data_source::{Destroy, Offer, SetActions};

use crate::compositor::prelude::*;

impl<'a> MessageHandler<Offer<'a>> for Compositor {
    fn handle(&mut self, _msg: Msg<Offer>, _client: &mut ClientMut) -> Todo<Offer<'a>> {
        Todo::new()
    }
}

impl MessageHandler<Destroy> for Compositor {
    fn handle(&mut self, _msg: Msg<Destroy>, _client: &mut ClientMut) -> Todo<Destroy> {
        Todo::new()
    }
}

impl MessageHandler<SetActions> for Compositor {
    fn handle(&mut self, _msg: Msg<SetActions>, _client: &mut ClientMut) -> Todo<SetActions> {
        Todo::new()
    }
}
