use std::collections::HashMap;
use smithay::reexports::drm::control::{connector, crtc, Device as ControlDevice};

pub trait CrtcMapper {
    fn map<'a>(&mut self, drm: &impl ControlDevice, connectors: impl Iterator<Item = &'a connector::Info> + Clone);
    fn crtc_for_connector(&self, connector: &connector::Handle) -> Option<crtc::Handle>;
}

#[derive(Default)]
pub struct SimpleCrtcMapper {
    crtcs: HashMap<connector::Handle, crtc::Handle>,
}

impl SimpleCrtcMapper {
    pub fn new() -> Self {
        Self::default()
    }

    fn is_taken(&self, crtc: &crtc::Handle) -> bool {
        self.crtcs.values().any(|v|v==crtc)
    }

    fn is_available(&self, crtc: &crtc::Handle) -> bool {
        !self.is_taken(crtc)
    }

    fn restored_for_connector(
        &self,
        drm: &impl ControlDevice,
        connector: &connector::Info,
    ) -> Option<crtc::Handle> {
        let encoder = drm.get_encoder(connector.current_encoder()?).ok()?;
        let crtc = encoder.crtc()?;
        self.is_available(&crtc).then_some(crtc)
    }

    fn next_available_for_connector(
        &self,
        drm: &impl ControlDevice,
        connector: &connector::Info,
    ) -> Option<crtc::Handle> {
        let res_handle = drm.resource_handles().ok()?;
        connector
            .encoders()
            .iter()
            .flat_map(|encoder| drm.get_encoder(*encoder))
            .find_map(|encoder| {
                res_handle
                    .filter_crtcs(encoder.possible_crtcs())
                    .into_iter()
                    .find(|crtc| self.is_available(crtc))
            })
    }
}

impl CrtcMapper for SimpleCrtcMapper {
    fn map<'a>(
        &mut self,
        drm: &impl ControlDevice,
        connectors: impl Iterator<Item = &'a connector::Info> + Clone,
    ) {
        for connector in connectors
            .clone()
            .filter(|conn| conn.state() != connector::State::Connected)
        {
            self.crtcs.remove(&connector.handle());
        }

        let mut needs_crtc = connectors
            .filter(|conn|conn.state()==connector::State::Connected)
            .filter(|conn|!self.crtcs.contains_key(&conn.handle()))
            .collect::<Vec<_>>();

        needs_crtc.retain(|conn|{
            let Some(crtc) = self.restored_for_connector(drm, conn) else {
                return true;
            };
            self.crtcs.insert(conn.handle(), crtc);
            return false;
        });

        for conn in needs_crtc {
            let Some(crtc) = self.next_available_for_connector(drm, conn) else {
                continue;
            };
            self.crtcs.insert(conn.handle(), crtc);
        }

    }

    fn crtc_for_connector(&self, connector: &connector::Handle) -> Option<crtc::Handle> {
        self.crtcs.get(connector).copied()
    }
}

