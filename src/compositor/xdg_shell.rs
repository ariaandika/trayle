use wayland::xdg_wm_base::{self, CreatePositioner, GetXdgSurface, Pong};
use wayland::xdg_surface;
use wayland::xdg_toplevel;

use crate::compositor::prelude::*;

macro_rules! ignore {
    ($req:ty) => {
        impl RequestHandler<$req> for Compositor {
            fn handle(&mut self, _: $req, _: &mut ClientMut) -> Result<(), WlError> {
                Ok(())
            }
        }
    };
}

macro_rules! insert {
    ($req:ty,$field:ident) => {
        impl RequestHandler<$req> for Compositor {
            fn handle(&mut self, req: $req, client: &mut ClientMut) -> Result<(), WlError> {
                let _ = client.objects.create(req.$field)?;
                Ok(())
            }
        }
    };
}

ignore!(xdg_wm_base::Destroy);
insert!(CreatePositioner, positioner);
insert!(GetXdgSurface, xdg_surface);
ignore!(Pong);

ignore!(xdg_surface::Destroy);
insert!(xdg_surface::GetToplevel, toplevel);
insert!(xdg_surface::GetPopup, popup);
ignore!(xdg_surface::SetWindowGeometry);
ignore!(xdg_surface::AckConfigure);

ignore!(xdg_toplevel::SetTitle<'_>);
ignore!(xdg_toplevel::SetAppId<'_>);
