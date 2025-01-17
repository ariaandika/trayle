use std::{collections::HashMap, io, iter::{Chain, Map}, vec};

use drm::control::{connector, crtc, Device as ControlDevice};

use crate::{ConnectorScanner, CrtcMapper, SimpleCrtcMapper};


#[derive(Debug, Default)]
pub struct DrmScanner<Mapper = SimpleCrtcMapper>
where
    Mapper: CrtcMapper,
{
    connectors: ConnectorScanner,
    crtc_mapper: Mapper,
}

impl<Mapper> DrmScanner<Mapper>
where Mapper: CrtcMapper
{
    pub fn new_with_mapper(mapper: Mapper) -> Self {
        Self {
            connectors: Default::default(),
            crtc_mapper: mapper,
        }
    }

    pub fn crtc_mapper(&self) -> &Mapper {
        &self.crtc_mapper
    }

    pub fn crtc_mapper_mut(&mut self) -> &Mapper {
        &mut self.crtc_mapper
    }

    pub fn scan_connectors(&mut self, drm: &impl ControlDevice) -> io::Result<DrmScanResult> {
        let scan = self.connectors.scan(drm)?;

        let removed = scan
            .disconnected
            .into_iter()
            .map(|info|{
                let crtc = self.crtc_mapper.crtc_for_connector(&info.handle());
                (info, crtc)
            })
            .collect();

        self.crtc_mapper
            .map(drm, self.connectors.connectors().iter().map(|(_, info)|info));

        let added = scan
            .connected
            .into_iter()
            .map(|info|{
                let crtc = self.crtc_mapper.crtc_for_connector(&info.handle());
                (info, crtc)
            })
            .collect();

        Ok(DrmScanResult {
            connected: added,
            disconnected: removed,
        })
    }

    pub fn connectors(&self) -> &HashMap<connector::Handle, connector::Info> {
        self.connectors.connectors()
    }

    pub fn crtc_for_connector(&self, connector: &connector::Handle) -> Option<crtc::Handle> {
        self.crtc_mapper.crtc_for_connector(connector)
    }

    pub fn crtcs(&self) -> impl Iterator<Item = (&connector::Info, crtc::Handle)> {
        self.connectors()
            .iter()
            .filter_map(|(handle, info)|Some((info, self.crtc_for_connector(handle)?)))
    }
}

type DrmScanItem = (connector::Info, Option<crtc::Handle>);

#[derive(Debug, Clone)]
pub struct DrmScanResult {
    pub connected: Vec<DrmScanItem>,
    pub disconnected: Vec<DrmScanItem>,
}

impl DrmScanResult {
    pub fn iter(&self) -> impl Iterator<Item = DrmScanEvent> {
        self.clone().into_iter()
    }
}

#[derive(Debug)]
pub enum DrmScanEvent {
    Connected {
        connector: connector::Info,
        crtc: Option<crtc::Handle>,
    },
    Disconnected {
        connector: connector::Info,
        crtc: Option<crtc::Handle>,
    }
}

impl DrmScanEvent {
    fn connected((connector, crtc): DrmScanItem) -> Self {
        Self::Connected { connector, crtc }
    }
    fn disconnected((connector, crtc): DrmScanItem) -> Self {
        Self::Disconnected { connector, crtc }
    }
}

type DrmScanItemToEvent = fn(DrmScanItem) -> DrmScanEvent;

impl IntoIterator for DrmScanResult {
    type Item = DrmScanEvent;
    type IntoIter = Chain<
        Map<vec::IntoIter<DrmScanItem>, DrmScanItemToEvent>,
        Map<vec::IntoIter<DrmScanItem>, DrmScanItemToEvent>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.disconnected
            .into_iter()
            .map(DrmScanEvent::disconnected as DrmScanItemToEvent)
            .chain(
                self.connected
                    .into_iter()
                    .map(DrmScanEvent::connected as DrmScanItemToEvent),
            )
    }
}



