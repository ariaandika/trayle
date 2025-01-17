use std::{collections::HashMap, io, iter::{Chain, Map}};

use drm::control::{connector, Device as ControlDevice};


#[derive(Debug, Default)]
pub struct ConnectorScanner {
    connectors: HashMap<connector::Handle, connector::Info>
}

impl ConnectorScanner {
    pub fn new() -> Self {
        Self::default()
    }

    /// should be called every device changed event
    pub fn scan(&mut self, drm: &impl ControlDevice) -> io::Result<ConnectorScanResult> {
        let res_handle = drm.resource_handles()?;
        let connector_handles = res_handle.connectors();

        let mut added = vec![];
        let mut removed = vec![];

        for conn in connector_handles.iter().filter_map(|conn|drm.get_connector(*conn, true).ok()) {
            let curr_state = conn.state();

            use connector::State;
            if let Some(old) = self.connectors.insert(conn.handle(), conn.clone()) {
                match (old.state(),curr_state) {
                    (State::Connected, State::Disconnected) => removed.push(conn),
                    (State::Disconnected|State::Unknown, State::Connected) => added.push(conn),
                    _ => {}
                }
            } else if curr_state == State::Connected {
                added.push(conn);
            }
        }

        Ok(ConnectorScanResult {
            connected: added,
            disconnected: removed,
        })
    }

    pub fn connectors(&self) -> &HashMap<connector::Handle, connector::Info> {
        &self.connectors
    }
}

#[derive(Debug, Clone)]
pub enum ConnectorScanEvent {
    Connected(connector::Info),
    Disconnected(connector::Info),
}

#[derive(Debug, Default, Clone)]
pub struct ConnectorScanResult {
    pub connected: Vec<connector::Info>,
    pub disconnected: Vec<connector::Info>,
}

impl ConnectorScanResult {
    pub fn iter(&self) -> impl Iterator<Item = ConnectorScanEvent> {
        self.clone().into_iter()
    }
}

type ConnectorScanItemToEvent = fn(connector::Info) -> ConnectorScanEvent;

impl IntoIterator for ConnectorScanResult {
    type Item = ConnectorScanEvent;
    type IntoIter = Chain<
        Map<std::vec::IntoIter<connector::Info>, ConnectorScanItemToEvent>,
        Map<std::vec::IntoIter<connector::Info>, ConnectorScanItemToEvent>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.disconnected
            .into_iter()
            .map(ConnectorScanEvent::Disconnected as ConnectorScanItemToEvent)
            .chain(
                self.connected
                .into_iter()
                .map(ConnectorScanEvent::Connected as ConnectorScanItemToEvent)
            )
    }
}

