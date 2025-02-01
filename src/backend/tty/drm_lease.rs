use smithay::{
    backend::drm::DrmNode,
    wayland::drm_lease::{
        DrmLease, DrmLeaseBuilder, DrmLeaseHandler, DrmLeaseRequest, DrmLeaseState, LeaseRejected,
    },
};

use crate::Trayle;

use super::Tty;

smithay::delegate_drm_lease!(Trayle<Tty>);

impl DrmLeaseHandler for Trayle<Tty> {
    fn drm_lease_state(&mut self, node: DrmNode) -> &mut DrmLeaseState {
        self.backend
            .backends
            .get_mut(&node)
            .unwrap()
            .leasing_global
            .as_mut()
            .unwrap()
    }

    fn lease_request(
        &mut self,
        node: DrmNode,
        request: DrmLeaseRequest,
    ) -> Result<DrmLeaseBuilder, LeaseRejected> {
        let backend = self.backend
            .backends
            .get(&node)
            .ok_or_else(LeaseRejected::default)?;

        let drm_device = backend.drm_output_manager.device();
        let mut builder = DrmLeaseBuilder::new(drm_device);
        for conn in request.connectors {
            let Some((_,crtc)) = backend.non_desktop_connectors.iter().find(|(handle,_)|*handle == conn) else {
                tracing::warn!(?conn, "lease request for desktop connector denied");
                return Err(LeaseRejected::default());
            };
            builder.add_connector(conn);
            builder.add_crtc(*crtc);
            let planes = drm_device.planes(crtc).map_err(LeaseRejected::with_cause)?;
            let (primary_plane,primary_plane_claim) = planes
                .primary
                .iter()
                .find_map(|plane|{
                    drm_device
                        .claim_plane(plane.handle, *crtc)
                        .map(|claim|(plane,claim))
                })
                .ok_or_else(LeaseRejected::default)?;
            builder.add_plane(primary_plane.handle, primary_plane_claim);

            if let Some((cursor, claim)) = planes.cursor.iter().find_map(|plane| {
                drm_device
                    .claim_plane(plane.handle, *crtc)
                    .map(|claim| (plane, claim))
            }) {
                builder.add_plane(cursor.handle, claim)
            }
        }

        Ok(builder)
    }

    fn new_active_lease(&mut self, node: DrmNode, lease: DrmLease) {
        let backend = self.backend.backends.get_mut(&node).unwrap();
        backend.active_leases.push(lease);
    }

    fn lease_destroyed(&mut self, node: DrmNode, lease_id: u32) {
        let backend = self.backend.backends.get_mut(&node).unwrap();
        backend.active_leases.retain(|l|l.id() != lease_id);
    }
}

