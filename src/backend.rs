use anyhow::{bail, Context, Result};
use smithay::{
    backend::{
        allocator::dmabuf::Dmabuf,
        drm::{DrmDeviceFd, DrmNode, NodeType},
        egl::context::ContextPriority,
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        renderer::{
            gles::GlesRenderer,
            multigpu::{gbm::GbmGlesBackend, GpuManager, MultiRenderer, MultiTexture},
            DebugFlags, ImportDma,
        },
        session::{
            libseat::{LibSeatSession, LibSeatSessionNotifier},
            Session,
        },
        udev::{self, UdevBackend},
    },
    reexports::{
        input::{Device as LibinputDevice, Libinput},
        wayland_server::{protocol::wl_surface::WlSurface, DisplayHandle},
    },
    wayland::{
        dmabuf::{DmabufFeedbackBuilder, DmabufGlobal, DmabufState},
        drm_syncobj::DrmSyncobjState,
    },
};
use std::{collections::HashMap, ops::{Deref, DerefMut}};

use crate::{trayle::DeviceData, Trayle};

pub type UdevRenderer<'a> = MultiRenderer<
    'a,'a,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
>;

pub enum LazyDmabufState {
    Uninit,
    Init((DmabufState, DmabufGlobal))
}

impl LazyDmabufState {
    pub fn write(&mut self, val: (DmabufState, DmabufGlobal)) {
        *self = Self::Init(val);
    }
}

impl Deref for LazyDmabufState {
    type Target = (DmabufState, DmabufGlobal);

    fn deref(&self) -> &Self::Target {
        match self {
            LazyDmabufState::Init(e) => e,
            LazyDmabufState::Uninit => panic!("uninitialized"),
        }
    }
}

impl DerefMut for LazyDmabufState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            LazyDmabufState::Init(e) => e,
            LazyDmabufState::Uninit => panic!("uninitialized"),
        }
    }
}

pub struct Backend {
    pub seat: String,
    pub keyboards: Vec<LibinputDevice>,
    pub devices: HashMap<DrmNode, DeviceData>,

    pub primary_gpu: DrmNode,
    pub gpus: GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>,
    pub debug_flags: DebugFlags,

    pub session: LibSeatSession,
    pub input: Libinput,
    pub dmabuf_state: LazyDmabufState,
    pub syncobj_state: Option<DrmSyncobjState>,
}

impl Backend {
    pub fn setup(dh: &DisplayHandle) -> Result<(Backend, BackendSources)> {
        // libseat
        let (session, session_source) = LibSeatSession::new().context("failed to setup libseat")?;
        let seat = session.seat();
        tracing::info!("using seat {seat:?}");


        // gpu devices
        let primary_gpu = match udev::primary_gpu(&seat)
            .context("failed to query gpu")?
            .and_then(|gpu|DrmNode::from_path(gpu).ok()?.node_with_type(NodeType::Render)?.ok())
        {
            Some(ok) => ok,
            None => udev::all_gpus(&seat)
                .context("failed to query gpu")?
                .into_iter()
                .find_map(|gpu|DrmNode::from_path(gpu).ok())
                .context("no gpu found")?,
        };
        let graphics_api = GbmGlesBackend::with_context_priority(ContextPriority::High);
        let mut gpus = GpuManager::new(graphics_api).context("failed to setup gbm gles renderer")?;
        tracing::info!("using {primary_gpu:?} as primary gpu");

        let udev = UdevBackend::new(&seat).context("failed to setup udev")?;

        // libinput
        type Libseat = LibinputSessionInterface<LibSeatSession>;
        let mut input = Libinput::new_with_udev::<Libseat>(session.clone().into());
        input.udev_assign_seat(&seat).or_else(|()|bail!("failed to assign a seat to current libinput"))?;
        let input_source = LibinputInputBackend::new(input.clone());


        let backend = Backend {
            seat,
            keyboards: vec![],
            devices: HashMap::new(),

            primary_gpu,
            gpus,
            debug_flags: DebugFlags::empty(),

            session,
            input,
            dmabuf_state: LazyDmabufState::Uninit,
            syncobj_state: None,
        };

        let sources = BackendSources {
            session: session_source,
            input: input_source,
            udev,
        };

        Ok((backend, sources))
    }
}

/// mostly delegation function
impl Backend {
    /// delegate function from [`GpuManager::single_renderer`] with [`Tty::primary_gpu`]
    pub fn primary_renderer(&mut self) -> UdevRenderer {
        self.gpus.single_renderer(&self.primary_gpu).expect("failed to get primary renderer")
    }

    /// see [`ImportDma::import_dmabuf`]
    pub fn import_dmabuf(&mut self, dmabuf: &Dmabuf) -> Result<MultiTexture> {
        Ok(self.primary_renderer().import_dmabuf(dmabuf, None)?)
    }

    /// optimizing buffer imports across multiple gpus
    ///
    /// can call be called on commit to start necessary copy processes early
    ///
    /// required to use with smithay's [`utils::on_commit_buffer_handler`]
    ///
    /// see [`GpuManager::early_import`]
    ///
    /// [`utils::on_commit_buffer_handler`]: smithay::backend::renderer::utils::on_commit_buffer_handler
    pub fn early_import(&mut self, surface: &WlSurface) -> Result<()> {
        self.gpus.early_import(self.primary_gpu, surface).map_err(Into::into)
    }

}

pub struct BackendSources {
    pub session: LibSeatSessionNotifier,
    pub input: LibinputInputBackend,
    pub udev: UdevBackend,
}

