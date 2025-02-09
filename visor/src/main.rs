use std::time::Duration;

use anyhow::Result;
use smithay_client_toolkit::{
    self as smithay,
    compositor::{CompositorHandler, CompositorState},
    output::{OutputHandler, OutputState},
    reexports::{
        calloop::{EventLoop, LoopHandle},
        calloop_wayland_source::WaylandSource,
    },
    registry::{ProvidesRegistryState, RegistryState},
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers, RawModifiers},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        xdg::{
            window::{Window, WindowConfigure, WindowDecorations, WindowHandler},
            XdgShell,
        },
        WaylandSurface,
    },
    shm::{
        slot::{Buffer, SlotPool},
        Shm, ShmHandler,
    },
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{
        wl_keyboard::WlKeyboard, wl_output::WlOutput, wl_seat::WlSeat, wl_shm, wl_surface::WlSurface
    },
    Connection, QueueHandle,
};

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    app().inspect_err(|err|tracing::error!("{err:?}"))
}

fn app() -> Result<()> {
    let conn = Connection::connect_to_env()?;

    // enumerate the list of globals protocols that server implements
    let (globals, event_queue) = registry_queue_init::<App>(&conn)?;
    let qh = event_queue.handle();
    let mut event_loop = EventLoop::try_new()?;
    let lh = event_loop.handle();

    tracing::debug!(?globals);

    WaylandSource::new(conn.clone(), event_queue).insert(event_loop.handle())?;

    // compositor (the protocol) allow configuring surfaces to be presented
    let compositor = CompositorState::bind(&globals, &qh)?;
    // xdg shell allow creating desktop window
    let xdg_shell = XdgShell::bind(&globals, &qh)?;

    // shared memory for client to rendered to
    let shm_state = Shm::bind(&globals, &qh)?;
    let pool = SlotPool::new(256 * 256 * 4, &shm_state)?;

    let registry_state = RegistryState::new(&globals);
    let seat_state = SeatState::new(&globals, &qh);
    let output_state = OutputState::new(&globals, &qh);

    let surface = compositor.create_surface(&qh);
    let window = xdg_shell.create_window(surface, WindowDecorations::RequestServer, &qh);
    window.set_title("deez");
    window.set_app_id("nutz");
    window.commit();

    let mut app = App {
        exit: false,
        keyboard: None,
        lh,
        width: 640,
        height: 480,
        shift: 0,
        buffer: None,
        first_configured: false,

        window,
        pool,

        registry_state,
        output_state,
        seat_state,
        shm_state,
    };

    tracing::info!("setup finished");

    loop {
        event_loop.dispatch(Duration::from_millis(16), &mut app)?;

        if app.exit {
            break;
        }
    }

    Ok(())
}

struct App {
    exit: bool,
    keyboard: Option<WlKeyboard>,
    lh: LoopHandle<'static, App>,
    width: u32,
    height: u32,
    buffer: Option<Buffer>,
    shift: u8,
    first_configured: bool,

    window: Window,
    pool: SlotPool,

    registry_state: RegistryState,
    output_state: OutputState,
    seat_state: SeatState,
    shm_state: Shm,
}

impl App {
    fn swap(&mut self) {
        self.shift = (self.shift + 1) % 4;
    }

    fn draw(&mut self, _conn: &Connection, qh: &QueueHandle<App>) {
        let width = self.width;
        let height = self.height;
        let stride = self.width as i32 * 4;

        let buffer = self.buffer.get_or_insert_with(||{
            self.pool
                .create_buffer(width as i32, height as i32, stride, wl_shm::Format::Argb8888)
                .unwrap()
                .0
        });

        let canvas = match self.pool.canvas(buffer) {
            Some(canvas) => canvas,
            None => {
                let (second_buffer, canvas) = self
                    .pool
                    .create_buffer(
                        self.width as i32,
                        self.height as i32,
                        stride,
                        wl_shm::Format::Argb8888,
                    )
                    .unwrap();
                *buffer = second_buffer;
                canvas
            }
        };

        // Draw to the window:
        {
            canvas.chunks_exact_mut(4).enumerate().for_each(|(_index, chunk)| {
                for (i,byte) in chunk[0..3].iter_mut().enumerate() {
                    *byte = if i % 4 == self.shift as usize { 48 } else { 0 };
                }
            });
        }

        self.window.wl_surface().damage_buffer(0, 0, self.width as i32/2, self.height as i32/2);
        self.window.wl_surface().frame(qh, self.window.wl_surface().clone());
        buffer.attach_to(self.window.wl_surface()).unwrap();
        self.window.wl_surface().commit();
    }
}

smithay::delegate_compositor!(App);

impl CompositorHandler for App {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _new_transform: wayland_client::protocol::wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _time: u32,
    ) {
        self.draw(conn,qh);
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _output: &wayland_client::protocol::wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _output: &wayland_client::protocol::wl_output::WlOutput,
    ) {
    }
}

smithay::delegate_registry!(App);

impl ProvidesRegistryState for App {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    smithay::registry_handlers![OutputState, SeatState];
}

smithay::delegate_output!(App);

impl OutputHandler for App {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: WlOutput,
    ) {
    }
}

smithay::delegate_seat!(App);

impl SeatHandler for App {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: WlSeat) {
    }

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: WlSeat,
        capability: Capability,
    ) {
        if matches!(capability, Capability::Keyboard) {
            let keyboard = self.seat_state.get_keyboard_with_repeat(
                qh,
                &seat,
                None,
                self.lh.clone(),
                Box::new(|_, _, _| {}),
            ).unwrap();
            self.keyboard.replace(keyboard);
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: WlSeat,
        capability: Capability,
    ) {
        if matches!(capability, Capability::Keyboard) {
            self.keyboard.take().as_ref().map(WlKeyboard::release);
        }
    }

    fn remove_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: WlSeat) {
    }
}

smithay::delegate_keyboard!(App);

impl KeyboardHandler for App {
    fn enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &WlKeyboard,
        _surface: &WlSurface,
        _serial: u32,
        _raw: &[u32],
        _keysyms: &[Keysym],
    ) {
        tracing::info!("keyboard focus");
    }

    fn leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &WlKeyboard,
        _surface: &WlSurface,
        _serial: u32,
    ) {
        tracing::info!("keyboard unfocus");
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &WlKeyboard,
        _serial: u32,
        event: KeyEvent,
    ) {
        tracing::info!(?event.keysym,"keyboard press");
        self.swap();
    }

    fn release_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &WlKeyboard,
        _serial: u32,
        event: KeyEvent,
    ) {
        tracing::info!(?event,"keyboard release");
    }

    fn update_modifiers(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &WlKeyboard,
        _serial: u32,
        modifiers: Modifiers,
        _raw_modifiers: RawModifiers,
        _layout: u32,
    ) {
        tracing::info!(?modifiers,"keyboard mods");
    }
}

smithay::delegate_xdg_shell!(App);
smithay::delegate_xdg_window!(App);

impl WindowHandler for App {
    fn request_close(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _window: &Window) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        _window: &Window,
        configure: WindowConfigure,
        _serial: u32,
    ) {
        tracing::info!("window configured ({},{})",self.width,self.height);

        self.buffer = None;

        if let Some(w) = configure.new_size.0 {
            self.width = w.get();
        }

        if let Some(h) = configure.new_size.1 {
            self.height = h.get();
        }

        if !self.first_configured {
            self.first_configured = true;
            self.draw(conn, qh);
        }
    }
}


smithay::delegate_shm!(App);

impl ShmHandler for App {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm_state
    }
}

