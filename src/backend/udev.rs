// use smithay::backend::{drm::{DrmDeviceFd, DrmNode, NodeType}, egl::context::ContextPriority, renderer::{gles::GlesRenderer, multigpu::{gbm::GbmGlesBackend, GpuManager}}, session::{libseat::LibSeatSession, Session}, udev};

pub fn setup() -> Result<(), Box<dyn std::error::Error>> {
    // let (seat, notifier) = LibSeatSession::new()?;
    //
    // let primary_gpu = udev::primary_gpu(seat.seat())?
    //     .and_then(|path| DrmNode::from_path(path).ok()?.node_with_type(NodeType::Render)?.ok())
    //     .expect("no gpu available");
    //
    // println!("using {primary_gpu} as primary gpu");
    //
    // let seat_name = "";
    //
    // let gpus = {
    //     let gles = GbmGlesBackend::<GlesRenderer, DrmDeviceFd>::with_context_priority(ContextPriority::High);
    //     GpuManager::new(gles)?
    // };

    // GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>;

    todo!()
}

