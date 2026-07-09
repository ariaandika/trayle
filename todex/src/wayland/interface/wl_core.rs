use super::*;

interface! {
    pub struct WlRegistry;

    impl Request {
        pub fn bind(name: uint, id_name: string, id_version: version, new_id: object_id);
    }

    impl Event {
        pub fn global(name: uint, interface: string, version: uint);
        pub fn global_remove(name: uint);
    }
}

interface! {
    pub struct WlCallback;

    impl Event {
        #[destructor]
        pub fn done(callback_data: uint);
    }
}

interface! {
    pub struct WlBuffer;

    impl Request {
        #[destructor]
        pub fn destroy();
    }

    impl Event {
        pub fn release();
    }
}

// ===== Compositor =====

interface! {
    #[global = 7]
    pub struct WlCompositor;

    impl Request {
        pub fn create_surface(new_id: new_id<wl_surface>);
        pub fn create_region(new_id: new_id<wl_region>);
        #[since = 7, destructor]
        pub fn release();
    }
}

interface! {
    pub struct WlSurface;

    impl Request {
        #[destructor]
        pub fn destroy();
        pub fn attach(buffer: object<wl_buffer>?, x: int, y: int);
        pub fn damage(x: int, y: int, width: int, height: int);
        pub fn frame(callback_id: new_id<wl_callback>);
        pub fn set_opaque_region(region: object<wl_region>?);
        pub fn set_input_region(region: object<wl_region>?);
        pub fn commit();
        #[since = 2]
        pub fn set_buffer_transform(transform: int<wl_output.transform>);
        #[since = 3]
        pub fn set_buffer_scale(scale: int);
        #[since = 4]
        pub fn damage_buffer(x: int, y: int, width: int, height: int);
        #[since = 5]
        pub fn offset(x: int, y: int);
        #[since = 7]
        pub fn get_release(callback_id: new_id<wl_callback>);
    }

    impl Event {
        pub fn enter(output: object<wl_output>);
        pub fn leave(output: object<wl_output>);
        #[since = 6]
        pub fn preferred_buffer_scale(factor: int);
        pub fn preferred_buffer_transform(transform: uint<wl_output.transform>);
    }

    #[error]
    pub enum Error {
        /// Invalid buffer scale.
        invalid_scale = 0,
        /// Invalid buffer transform.
        invalid_transform = 1,
        /// Invalid buffer size.
        invalid_size = 2,
        /// Invalid buffer offset.
        invalid_offset = 3,
        /// Surface destroyed before its role object.
        defunct_role_object = 4,
        /// No buffer attached.
        no_buffer = 5,
    }
}

interface! {
    pub struct WlRegion;

    impl Request {
        #[destructor]
        pub fn destroy();
        pub fn add(x: int, y: int, width: int, height: int);
        pub fn subtract(x: int, y: int, width: int, height: int);
    }
}

// ===== Output =====

interface! {
    #[global = 4]
    pub struct WlOutput;

    impl Request {
        #[since = 3, destructor]
        pub fn release();
    }

    impl Event {
        pub fn geometry(
            x: int, y: int,
            physical_width: int, physical_height: int,
            subpixel: int<wl_output.subpixel>,
            make: string, model: string,
            transform: int<wl_output.transform>
        );
    }

    pub enum Subpixel {
        /// Unknown geometry.
        unknown = 0,
        /// No geometry.
        none = 1,
        /// Horizontal RGB.
        horizontal_rgb = 2,
        /// Horizontal BGR.
        horizontal_bgr = 3,
        /// Vertical RGB.
        vertical_rgb = 4,
        /// Vertical BGR.
        vertical_bgr = 5,
    }

    pub enum Transform {
        /// No transform.
        normal = 0,
        /// 90 degrees counter-clockwise.
        d90 = 1,
        /// 180 degrees counter-clockwise.
        d180 = 2,
        /// 270 degrees counter-clockwise.
        d270 = 3,
        /// 180 degree flip around a vertical axis.
        flipped = 4,
        /// Flip and rotate 90 degrees counter-clockwise.
        flipped_90 = 5,
        /// flip and rotate 180 degrees counter-clockwise.
        flipped_180 = 6,
        /// flip and rotate 270 degrees counter-clockwise.
        flipped_270 = 7,
    }

    #[bitfield]
    pub enum Mode {
        current = 0x1,
        preferred = 0x2,
    }
}

// ===== Shm =====

interface! {
    #[global = 2]
    pub struct WlShm;

    impl Request {
        pub fn create_pool(new_id: new_id<wl_shm_pool>, fd: fd, size: int);
        #[since = 2, destructor]
        pub fn release();
    }

    impl Event {
        #[since = 2, destructor]
        pub fn format(format: uint<wl_shm.format_enum>);
    }

    #[error]
    pub enum Error {
        /// Unknown buffer format.
        invalid_format = 0,
        /// Invalid size or stride during pool or buffer creation.
        invalid_stride = 1,
        /// `mmap`-ing the file descriptor failed.
        invalid_fd = 2,
    }

    pub enum FormatEnum {
        /// 32-bit ARGB format, [31:0] A:R:G:B 8:8:8:8 little endian.
        argb8888 = 0,
        /// 32-bit RGB format, [31:0] x:R:G:B 8:8:8:8 little endian.
        xrgb8888 = 1,
    }
}

interface! {
    pub struct WlShmPool;

    impl Request {
        pub fn create_buffer(
            new_id: new_id<wl_buffer>,
            offset: int,
            width: int, height: int,
            stride: int,
            format: uint<wl_shm.format_enum>,
        );
        #[destructor]
        pub fn destroy();
        pub fn resize(size: int);
    }
}

// ===== Seat =====

interface! {
    #[global = 10]
    pub struct WlSeat;

    impl Request {
        pub fn get_pointer(new_id: new_id<wl_pointer>);
        pub fn get_keyboard(new_id: new_id<wl_keyboard>);
        pub fn get_touch(new_id: new_id<wl_touch>);
        #[since = 5, destructor]
        pub fn release();
    }

    impl Event {
        pub fn capabilities(capabilities: uint<wl_seat.capability>);
        #[since = 2]
        pub fn name(name: string);
    }

    #[bitfield]
    pub enum Capability {
        /// The seat has pointer devices.
        pointer = 1,
        /// The seat has keyboard devices.
        keyboard = 2,
        /// The seat has touch devices.
        touch = 4,
    }

    #[error]
    pub enum Error {
        /// Missing seat capability.
        missing_capability
    }
}

interface! {
    pub struct WlPointer;

    impl Request {
        pub fn set_cursor(serial: uint, surface: object<wl_surface>, hotspot_x: int, hotspot_y: int);
        #[since = 3, destructor]
        pub fn release();
    }

    impl Event {
        pub fn enter(serial: uint, surface: object<wl_surface>, surface_x: fixed, surface_y: fixed);
        pub fn leave(serial: uint, surface: object<wl_surface>);
        pub fn motion(time: uint, surface_x: fixed, surface_y: fixed);
        pub fn button(serial: uint, time: uint, button: uint, state: uint<wl_pointer.button_state>);
        pub fn axis(time: uint, axis: uint<wl_pointer.axis_enum>, value: fixed);
        #[since = 5]
        pub fn frame();
        pub fn axis_source(axis_source: uint<wl_pointer.axis_source_enum>);
        pub fn axis_stop(time: uint, axis: uint<wl_pointer.axis_enum>);
        // deprecated-since 8
        pub fn axis_discrete(axis: uint<wl_pointer.axis_enum>, discrete: int);
        #[since = 8]
        pub fn axis_value120(axis: uint<wl_pointer.axis_enum>, value120: int);
        #[since = 9]
        pub fn axis_relative_direction(
            axis: uint<wl_pointer.axis_enum>,
            direction: uint<wl_pointer.axis_relative_direction_enum>,
        );
    }

    #[error]
    pub enum Error {
        /// Given wl_surface has another role.
        role = 0,
    }

    pub enum ButtonState {
        released = 0,
        pressed = 1,
    }

    pub enum AxisEnum {
        vertical_scroll = 0,
        horizontal_scroll = 1,
    }

    pub enum AxisSourceEnum {
        wheel = 0,
        finger = 1,
        continuous = 2,
        // since 6
        wheel_tilt = 3,
    }

    pub enum AxisRelativeDirectionEnum {
        identical = 0,
        inverted = 1,
    }
}

interface! {
    pub struct WlKeyboard;

    impl Request {
        #[since = 3, destructor]
        pub fn release();
    }

    impl Event {
        pub fn keymap(format: uint<wl_keyboard.keymap_format>, fd: fd, size: uint);
        pub fn enter(serial: uint, surface: object<wl_surface>, keys: array);
        pub fn leave(serial: uint, surface: object<wl_surface>);
        pub fn key(serial: uint, time: uint, key: uint, state: uint<wl_keyboard.key_state>);
        pub fn modifiers(serial: uint, mods_depressed: uint, mods_latched: uint, mods_locked: uint, group: uint);
        #[since = 4]
        pub fn repeat_info(rate: int, delay: int);
    }

    pub enum KeymapFormat {
        /// No keymap; client must understand how to interpret the raw keycode.
        no_keymap = 0,
        /// `libxkbcommon` compatible, null-terminated string; to determine the xkb keycode, clients must add 8 to the key event keycode.
        xkb_v1 = 1,
    }

    pub enum KeyState {
        /// Key is not pressed.
        released = 0,
        /// Key is pressed.
        pressed = 1,
        // since 10
        /// Key was repeated.
        repeated = 2,
    }
}

interface! {
    pub struct WlTouch;

    impl Request {
        #[since = 3, destructor]
        pub fn release();
    }

    impl Event {
        pub fn down(serial: uint, time: uint, surface: object<wl_surface>, id: int, x: fixed, y: fixed);
        pub fn up(serial: uint, time: uint, id: int);
        pub fn motion(time: uint, id: int, x: fixed, y: fixed);
        pub fn frame();
        pub fn cancel();
        #[since = 6]
        pub fn shape(id: int, major: fixed, minor: fixed);
        pub fn orientation(id: int, orientation: fixed);
    }
}

// ===== Shm =====

interface! {
    #[global = 4]
    pub struct WlDataDeviceManager;

    impl Request {
        pub fn create_data_source(new_id: new_id<wl_data_source>);
        pub fn get_data_device(new_id: new_id<wl_data_device>, seat: object<wl_seat>);
        #[since = 4, destructor]
        pub fn release();
    }

    // #[since = 3]
    #[bitfield]
    pub enum DndAction {
        /// No action.
        none = 0,
        /// Copy action.
        copy = 1,
        /// Move action.
        move = 2,
        /// Ask action.
        ask = 4,
    }
}

interface! {
    pub struct WlDataSource;

    impl Request {
        pub fn offer(mime_type: string);
        #[destructor]
        pub fn destroy();
        #[since = 3]
        pub fn set_actions(dnd_actions: uint<wl_data_device_manager.dnd_action>);
    }

    impl Event {
        pub fn target(mime_type: string?);
        pub fn send(mime_type: string, fd: fd);
        pub fn cancelled();
        #[since = 3]
        pub fn dnd_drop_performed();
        pub fn dnd_finished();
        pub fn action(dnd_action: uint<wl_data_device_manager.dnd_action>);
    }

    #[error]
    pub enum Error {
        /// Action mask contains invalid values.
        invalid_action_mask = 0,
        /// Source does not accept this request.
        invalid_source = 1,
    }
}

interface! {
    pub struct WlDataDevice;

    impl Request {
        pub fn start_drag(
            source: object<wl_data_source>?,
            origin: object<wl_surface>,
            icon: object<wl_surface>?,
            serial: uint,
        );
        pub fn set_selection(source: object<wl_data_source>?, serial: uint);
        #[since = 2, destructor]
        pub fn release();
    }

    impl Event {
        pub fn data_offer(new_id: new_id<wl_data_offer>);
        pub fn enter(serial: uint, surface: object<wl_surface>, x: fixed, y: fixed, id: object<wl_data_offer>?);
        pub fn leave();
        pub fn motion(time: uint, x: fixed, y: fixed);
        pub fn drop();
        pub fn selection(id: object<wl_data_offer>?);
    }

    #[error]
    pub enum Error {
        /// Given wl_surface has another role.
        role = 0,
        /// Source has already been used.
        used_source = 1,
    }
}

interface! {
    pub struct WlDataOffer;

    impl Request {
        pub fn accept(serial: uint, mime_type: string?);
        pub fn receive(mime_type: string, fd: fd);
        #[destructor]
        pub fn destroy();
        #[since = 3]
        pub fn finish();
        pub fn set_actions(
            dnd_actions: uint<wl_data_device_manager.dnd_action>,
            preferred_action: uint<wl_data_device_manager.dnd_action>,
        );
    }

    impl Event {
        pub fn offer(mime_type: string);
        #[since = 3]
        pub fn source_actions(source_actions: uint<wl_data_device_manager.dnd_action>);
        pub fn action(dnd_action: uint<wl_data_device_manager.dnd_action>);
    }

    #[error]
    pub enum Error {
        /// Finish request was called untimely.
        invalid_finish = 0,
        /// Action mask contains invalid values.
        invalid_action_mask = 1,
        /// Action argument has an invalid value.
        invalid_action = 2,
        /// Offer does not accept this request.
        invalid_offer = 3,
    }
}

// ===== Subcompositor =====

interface! {
    #[global = 1]
    pub struct WlSubcompositor;

    impl Request {
        #[destructor]
        pub fn destroy();
        pub fn get_subsurface(id: new_id<wl_subsurface>, surface: object<wl_surface>, parent: object<wl_surface>);
    }

    #[error]
    pub enum Error {
        /// The to-be sub-surface is invalid.
        bad_surface = 0,
        /// The to-be sub-surface parent is invalid.
        bad_parent = 1,
    }
}

interface! {
    pub struct WlSubsurface;

    impl Request {
        #[destructor]
        pub fn destroy();
        pub fn set_position(x: int, y: int);
        pub fn place_above(sibling: object<wl_surface>);
        pub fn place_below(sibling: object<wl_surface>);
        pub fn set_sync();
        pub fn set_desync();
    }

    #[error]
    pub enum Error {
        /// wl_surface is not a sibling or the parent.
        bad_surface = 0,
    }
}
