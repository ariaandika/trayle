use super::*;

interface! {
    #[global = 7]
    pub struct XdgWmBase;

    impl Request {
        #[destructor]
        pub fn destroy();
        pub fn create_positioner(id: new_id<xdg_positioner>);
        pub fn get_xdg_surface(id: new_id<xdg_surface>, surface: object<wl_surface>);
        pub fn pong(serial: uint);
    }

    impl Event {
        pub fn ping(serial: uint);
    }

    pub enum Error {
        role,
        defunct_surfaces,
        not_the_topmost_popup,
        invalid_popup_parent,
        invalid_surface_state,
        invalid_positioner,
        unresponsive,
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

    pub enum Error {
        invalid_input,
    }

    pub enum Anchor {
        none,
        top,
        bottom,
        left,
        right,
        top_left,
        bottom_left,
        top_right,
        bottom_right,
    }

    pub enum Gravity {
        none,
        top,
        bottom,
        left,
        right,
        top_left,
        bottom_left,
        top_right,
        bottom_right,
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
        pub fn get_toplevel(id: new_id<xdg_toplevel>);
        pub fn get_popup(id: new_id<xdg_popup>, parent: object<xdg_surface>?, positioner: object<xdg_positioner>);
        pub fn set_window_geometry(x: int, y: int, width: int, height: int);
        pub fn ack_configure(serial: uint);
    }

    impl Event {
        pub fn configure(serial: uint);
    }

    pub enum Error {
        not_constructed,
        already_constructed,
        unconfigured_buffer,
        invalid_serial,
        invalid_size,
        defunct_role_object,
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

    pub enum Error {
        invalid_resize_edge,
        invalid_parent,
        invalid_size,
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
        maximized,
        fullscreen,
        resizing,
        activated,
        // #[since = 2]
        tiled_left,
        tiled_right,
        tiled_top,
        tiled_bottom,
        // #[since = 6]
        suspended,
        // #[since = 7]
        constrained_left,
        constrained_right,
        constrained_top,
        constrained_bottom,
    }

    pub enum WmCapabilitiesEnum {
        window_menu,
        maximize,
        fullscreen,
        minimize,
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

    pub enum Error {
        invalid_grab,
    }
}
