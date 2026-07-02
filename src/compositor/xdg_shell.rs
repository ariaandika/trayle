use xdg_wm_base::{CreatePositioner, GetXdgSurface, Pong};

use crate::compositor::prelude::*;

macro_rules! ignore {
    ($req:ty) => {
        impl MessageHandler<$req> for Compositor {
            fn handle(&mut self, _: Msg<$req>, _: &mut ClientMut) -> Result<(), WlError> {
                Ok(())
            }
        }
    };
}

macro_rules! insert {
    ($req:ty) => {
        impl MessageHandler<$req> for Compositor {
            fn handle(&mut self, req: Msg<$req>, client: &mut ClientMut) -> Result<(), WlError> {
                let _ = client.objects.create(req)?;
                Ok(())
            }
        }
    };
}

ignore!(xdg_wm_base::Destroy);
insert!(CreatePositioner);
insert!(GetXdgSurface);
ignore!(Pong);

ignore!(xdg_surface::Destroy);
insert!(xdg_surface::GetToplevel);
insert!(xdg_surface::GetPopup);
ignore!(xdg_surface::SetWindowGeometry);
ignore!(xdg_surface::AckConfigure);

ignore!(xdg_toplevel::SetTitle<'_>);
ignore!(xdg_toplevel::SetAppId<'_>);
