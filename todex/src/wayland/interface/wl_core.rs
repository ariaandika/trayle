use super::*;

interface! {
    pub struct WlDisplay;

    impl Request {
        pub fn sync(callback: new_id<wl_callback>);
        pub fn get_registry(registry: new_id<wl_registry>);
    }

    impl Event {
        pub fn error(object_id: object, code: uint, message: string);
        pub fn delete_id(id: uint);
    }

    pub enum DisplayError {
        invalid_object,
        invalid_method,
        no_memory,
        implementation,
    }
}

impl<'a> wl_display::Error<'a> {
    pub fn new(object_id: Object, code: u32, message: &'a str) -> Self {
        Self {
            object_id,
            code,
            message,
        }
    }
}

impl AsObjectId for wl_display::Error<'_> {
    fn object_id(&self) -> ObjectId {
        ObjectId::wl_display()
    }
}

impl AsObjectId for wl_display::DeleteId {
    fn object_id(&self) -> ObjectId {
        ObjectId::wl_display()
    }
}

interface! {
    pub struct WlRegistry;

    impl Request {
        pub fn bind(name: uint, id_name: string, id_version: version, new_id: new_id);
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
        pub fn create_surface(id: new_id<wl_surface>);
        pub fn create_region(id: new_id<wl_region>);
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
        pub fn frame(callback: new_id<wl_callback>);
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
        pub fn get_release(callback: new_id<wl_callback>);
    }

    impl Event {
        pub fn enter(output: object<wl_output>);
        pub fn leave(output: object<wl_output>);
        #[since = 6]
        pub fn preferred_buffer_scale(factor: int);
        pub fn preferred_buffer_transform(transform: uint<wl_output.transform>);
    }

    pub enum Error {
        invalid_scale,
        invalid_transform,
        invalid_size,
        invalid_offset,
        defunct_role_object,
        no_buffer,
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
        unknown,
        none,
        horizontal_rgb,
        horizontal_bgr,
        vertical_rgb,
        vertical_bgr,
    }

    pub enum Transform {
        normal,
        d90,
        d180,
        d270,
        flipped,
        flipped_90,
        flipped_180,
        flipped_270,
    }

    #[bitfield]
    pub enum Mode {
        current = 1,
        preferred = 2,
    }
}

// ===== Shm =====

interface! {
    #[global = 2]
    pub struct WlShm;

    impl Request {
        pub fn create_pool(id: new_id<wl_shm_pool>, fd: fd, size: int);
        #[since = 2, destructor]
        pub fn release();
    }

    impl Event {
        #[since = 2, destructor]
        pub fn format(format: uint<wl_shm.format_enum>);
    }

    pub enum Error {
        invalid_format,
        invalid_stride,
        invalid_fd,
    }

    pub enum FormatEnum {
        argb8888,
        xrgb8888,
    }
}

interface! {
    pub struct WlShmPool;

    impl Request {
        pub fn create_buffer(
            id: new_id<wl_buffer>,
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
        pub fn get_pointer(id: new_id<wl_pointer>);
        pub fn get_keyboard(id: new_id<wl_keyboard>);
        pub fn get_touch(id: new_id<wl_touch>);
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
        pointer = 1,
        keyboard = 2,
        touch = 4,
    }

    pub enum Error {
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

    pub enum Error {
        role,
    }

    pub enum ButtonState {
        released,
        pressed,
    }

    pub enum AxisEnum {
        vertical_scroll,
        horizontal_scroll,
    }

    pub enum AxisSourceEnum {
        wheel,
        finger,
        continuous,
        // since 6
        wheel_tilt,
    }

    pub enum AxisRelativeDirectionEnum {
        identical,
        inverted,
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
        no_keymap,
        xkb_v1,
    }

    pub enum KeyState {
        released,
        pressed,
        // since 10
        repeated,
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
        pub fn create_data_source(id: new_id<wl_data_source>);
        pub fn get_data_device(id: new_id<wl_data_device>, seat: object<wl_seat>);
        #[since = 4, destructor]
        pub fn release();
    }

    pub enum DndAction {
        none,
        copy,
        move,
        ask,
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

    pub enum Error {
        invalid_action_mask,
        invalid_source,
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
        pub fn data_offer(id: new_id<wl_data_offer>);
        pub fn enter(serial: uint, surface: object<wl_surface>, x: fixed, y: fixed, id: object<wl_data_offer>?);
        pub fn leave();
        pub fn motion(time: uint, x: fixed, y: fixed);
        pub fn drop();
        pub fn selection(id: object<wl_data_offer>?);
    }

    pub enum Error {
        role,
        used_source,
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

    pub enum Error {
        invalid_finish,
        invalid_action_mask,
        invalid_action,
        invalid_offer,
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

    pub enum Error {
        bad_surface,
        bad_parent,
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

    pub enum Error {
        bad_surface,
    }
}
