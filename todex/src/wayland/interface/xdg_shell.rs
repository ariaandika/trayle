use super::*;

interface! {
    #[global = 7]
    pub struct XdgWmBase;

    impl Request {
        #[destructor]
        pub fn destroy();
        pub fn create_positioner(new_id: new_id<xdg_positioner>);
        pub fn get_xdg_surface(new_id: new_id<xdg_surface>, surface: object<wl_surface>);
        pub fn pong(serial: uint);
    }

    impl Event {
        pub fn ping(serial: uint);
    }

    #[error]
    pub enum Error {
        /// Given wl_surface has another role.
        role = 0,
        /// xdg_wm_base was destroyed before children.
        defunct_surfaces = 1,
        /// The client tried to map or destroy a non-topmost popup.
        not_the_topmost_popup = 2,
        /// The client specified an invalid popup parent surface.
        invalid_popup_parent = 3,
        /// The client provided an invalid surface state.
        invalid_surface_state = 4,
        /// The client provided an invalid positioner.
        invalid_positioner = 5,
        /// The client did not respond to a ping event in time.
        unresponsive = 6,
    }
}

interface! {
    pub struct XdgPositioner;

    impl Request {
        #[destructor]
        pub fn destroy();
        pub fn set_size(width: int, height: int);
        pub fn set_anchor_rect(x: int, y: int, width: int, height: int);
        pub fn set_anchor(anchor: uint<xdg_positioner.anchor>);
        pub fn set_gravity(gravity: uint<xdg_positioner.gravity>);
        pub fn set_constraint_adjustment(constraint_adjustment: uint<xdg_positioner.constraint_adjustment>);
        pub fn set_offset(x: int, y: int);
        #[since = 3]
        pub fn set_reactive();
        pub fn set_parent_size(parent_width: int, parent_height: int);
        pub fn set_parent_configure(serial: uint);
    }

    #[error]
    pub enum Error {
        /// Invalid input provided.
        invalid_input = 0,
    }

    pub enum Anchor {
        none = 0,
        top = 1,
        bottom = 2,
        left = 3,
        right = 4,
        top_left = 5,
        bottom_left = 6,
        top_right = 7,
        bottom_right = 8,
    }

    pub enum Gravity {
        none = 0,
        top = 1,
        bottom = 2,
        left = 3,
        right = 4,
        top_left = 5,
        bottom_left = 6,
        top_right = 7,
        bottom_right = 8,
    }

    #[bitfield]
    pub enum ConstraintAdjustment {
        none = 0,
        slide_x = 1,
        slide_y = 2,
        flip_x = 4,
        flip_y = 8,
        resize_x = 16,
        resize_y = 32,
    }
}

interface! {
    pub struct XdgSurface;

    impl Request {
        #[destructor]
        pub fn destroy();
        pub fn get_toplevel(new_id: new_id<xdg_toplevel>);
        pub fn get_popup(
            new_id: new_id<xdg_popup>,
            parent: object<xdg_surface>?,
            positioner: object<xdg_positioner>,
        );
        pub fn set_window_geometry(x: int, y: int, width: int, height: int);
        pub fn ack_configure(serial: uint);
    }

    impl Event {
        pub fn configure(serial: uint);
    }

    #[error]
    pub enum Error {
        /// Surface was not fully constructed.
        not_constructed = 1,
        /// Surface was already constructed.
        already_constructed = 2,
        /// Attaching a buffer to an unconfigured surface.
        unconfigured_buffer = 3,
        /// Invalid serial number when acking a configure event.
        invalid_serial = 4,
        /// Width or height was zero or negative.
        invalid_size = 5,
        /// Surface was destroyed before its role object.
        defunct_role_object = 6,
    }
}

interface! {
    pub struct XdgToplevel;

    impl Request {
        #[destructor]
        pub fn destroy();
        pub fn set_parent(parent: object<xdg_toplevel>);
        pub fn set_title(title: string);
        pub fn set_app_id(app_id: string);
        pub fn show_window_menu(seat: object<wl_seat>, serial: uint, x: int, y: int);
        pub fn move(seat: object<wl_seat>, serial: uint);
        pub fn resize(seat: object<wl_seat>, serial: uint, edges: uint<xdg_toplevel.resize_edge>);
        pub fn set_max_size(width: int, height: int);
        pub fn set_min_size(width: int, height: int);
        pub fn set_maximized();
        pub fn unset_maximized();
        pub fn set_fullscreen(output: object<wl_output>?);
        pub fn unset_fullscreen();
        pub fn set_minimized();
    }

    impl Event {
        pub fn configure(width: int, height: int, states: array);
        pub fn close();
        #[since = 4]
        pub fn configure_bounds(width: int, height: int);
        #[since = 5]
        pub fn wm_capabilities(capabilities: array);
    }

    #[error]
    pub enum Error {
        /// Provided value is not a valid variant of the resize_edge enum.
        invalid_resize_edge = 0,
        /// Invalid parent toplevel.
        invalid_parent = 1,
        /// Client provided an invalid min or max size.
        invalid_size = 2,
    }

    pub enum ResizeEdge {
        none = 0,
        top = 1,
        bottom = 2,
        left = 4,
        top_left = 5,
        bottom_left = 6,
        right = 8,
        top_right = 9,
        bottom_right = 10,
    }

    pub enum State {
        maximized = 1,
        fullscreen = 2,
        resizing = 3,
        activated = 4,
        // #[since = 2]
        tiled_left = 5,
        tiled_right = 6,
        tiled_top = 7,
        tiled_bottom = 8,
        // #[since = 6]
        suspended = 9,
        // #[since = 7]
        constrained_left = 10,
        constrained_right = 11,
        constrained_top = 12,
        constrained_bottom = 13,
    }

    // #[since = 5]
    pub enum WmCapabilitiesEnum {
        /// `show_window_menu` is available.
        window_menu = 1,
        /// `set_maximized` and `unset_maximized` are available.
        maximize = 2,
        /// `set_fullscreen` and `unset_fullscreen` are available.
        fullscreen = 3,
        /// `set_minimized` is available.
        minimize = 4,
    }
}

interface! {
    pub struct XdgPopup;

    impl Request {
        #[destructor]
        pub fn destroy();
        pub fn grab(seat: object<wl_seat>, serial: uint);
        #[since = 3]
        pub fn reposition(positioner: object<xdg_positioner>, token: uint);
    }

    impl Event {
        pub fn configure(x: int, y: int, width: int, height: int);
        pub fn popup_done();
        #[since = 3]
        pub fn repositioned(token: uint);
    }

    #[error]
    pub enum Error {
        /// Tried to grab after being mapped.
        invalid_grab = 0,
    }
}
