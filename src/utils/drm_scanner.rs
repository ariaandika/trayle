pub use connector_scanner::{ConnectorScanEvent, ConnectorScanResult, ConnectorScanner};
use smithay::reexports::drm::control::{connector, crtc, Device as ControlDevice};

mod connector_scanner;
use std::{collections::HashMap, iter::{Chain, Map}};

mod crtc_mapper;
pub use crtc_mapper::{CrtcMapper, SimpleCrtcMapper};

#[derive(Default)]
pub struct DrmScanner<Mapper = SimpleCrtcMapper>
where
    Mapper: CrtcMapper,
{
    connectors: ConnectorScanner,
    crtc_mapper: Mapper,
}

impl<Mapper> DrmScanner<Mapper>
where
    Mapper: CrtcMapper + Default,
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<Mapper> DrmScanner<Mapper>
where
    Mapper: CrtcMapper,
{
    pub fn new_with_mapper(crtc_mapper: Mapper) -> Self {
        Self { crtc_mapper, connectors: Default::default(), }
    }

    pub fn crtcs(&self) -> impl Iterator<Item = (&connector::Info, crtc::Handle)> {
        self.connectors()
            .iter()
            .filter_map(|(handle,info)|Some((info,self.crtc_for_connector(handle)?)))
    }

    pub fn crtc_mapper(&self) -> &Mapper {
        &self.crtc_mapper
    }

    pub fn crtc_mapper_mut(&mut self) -> &mut Mapper {
        &mut self.crtc_mapper
    }

    pub fn connectors(&self) -> &HashMap<connector::Handle, connector::Info> {
        self.connectors.connectors()
    }

    pub fn crtc_for_connector(&self, connector: &connector::Handle) -> Option<crtc::Handle> {
        self.crtc_mapper.crtc_for_connector(connector)
    }

    pub fn scan_connectors(&mut self, drm: &impl ControlDevice) -> std::io::Result<DrmScanResult> {
        let scan = self.connectors.scan(drm)?;
        let removed = scan.disconnected
            .into_iter()
            .map(|conn|{
                let crtc = self.crtc_mapper.crtc_for_connector(&conn.handle());
                (conn,crtc)
            })
            .collect();

        self.crtc_mapper
            .map(drm, self.connectors.connectors().iter().map(|(_,conn)|conn));

        let added = scan.connected
            .into_iter()
            .map(|conn|{
                let crtc = self.crtc_mapper.crtc_for_connector(&conn.handle());
                (conn,crtc)
            })
            .collect();

        Ok(DrmScanResult { connected: added, disconnected: removed })
    }
}

type DrmScanItem = (connector::Info,Option<crtc::Handle>);

#[derive(Default,Clone)]
pub struct DrmScanResult {
    pub connected: Vec<DrmScanItem>,
    pub disconnected: Vec<DrmScanItem>,
}

impl DrmScanResult {
    pub fn iter(&self) -> impl Iterator<Item = DrmScanEvent> {
        self.clone().into_iter()
    }
}

#[derive(Clone)]
pub enum DrmScanEvent {
    Connected {
        connector: connector::Info,
        crtc: Option<crtc::Handle>
    },
    Disconnected {
        connector: connector::Info,
        crtc: Option<crtc::Handle>
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
        Map<std::vec::IntoIter<DrmScanItem>, DrmScanItemToEvent>,
        Map<std::vec::IntoIter<DrmScanItem>, DrmScanItemToEvent>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.disconnected
            .into_iter()
            .map(DrmScanEvent::disconnected as DrmScanItemToEvent)
            .chain(
                self.connected
                    .into_iter()
                    .map(DrmScanEvent::connected as DrmScanItemToEvent)
            )
    }
}

