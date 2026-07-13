use super::*;

interface! {
    #[global = 5]
    pub struct ZwpLinuxDmabufV1;

    impl Request {
        #[destructor]
        pub fn destroy();
        pub fn create_params(params_id: new_id<zwp_linux_buffer_params_v1>);
        #[since = 4]
        pub fn get_default_feedback(id: new_id<zwp_linux_dmabuf_feedback_v1>);
        pub fn get_surface_feedback(id: new_id<zwp_linux_dmabuf_feedback_v1>, surface: object<wl_surface>);
    }

    impl Event {
        // #[deprecated = 4]
        pub fn format(format: uint);
        #[since = 3]
        // #[deprecated = 4]
        pub fn modifier(format: uint);
    }
}

interface! {
    pub struct ZwpLinuxBufferParamsV1;

    impl Request {
        #[destructor]
        pub fn destroy();
        pub fn add(fd: fd, plane_idx: uint, offset: uint, stride: uint, modifier_hi: uint, modifier_lo: uint);
        pub fn create(width: int, height: int, format: uint, flags: uint<zwp_linux_buffer_params_v1.flags>);
        #[since = 2]
        pub fn create_immed(
            buffer_id: new_id<wl_buffer>,
            width: int, height: int, format: uint,
            flags: uint<zwp_linux_buffer_params_v1.flags>,
        );
    }

    impl Event {
        pub fn created(buffer: new_id<wl_buffer>);
        pub fn failed();
    }

    #[error]
    enum Error {
        /// The dmabuf_batch object has already been used to create a wl_buffer.
        already_used = 0,
        /// Plane index out of bounds.
        plane_idx = 1,
        /// The plane index was already set.
        plane_set = 2,
        /// Missing or too many planes to create a buffer.
        incomplete = 3,
        /// Format not supported.
        invalid_format = 4,
        /// Invalid width or height.
        invalid_dimensions = 5,
        /// Offset + stride * height goes out of dmabuf bounds.
        out_of_bounds = 6,
        /// Invalid wl_buffer resulted from importing dmabufs via the create_immed request on given buffer_params.
        invalid_wl_buffer = 7,
    }

    #[bitfield]
    enum Flags {
        y_invert = 1,
        interlaced = 2,
        bottom_first = 4,
    }
}

interface! {
    pub struct ZwpLinuxDmabufFeedbackV1;

    impl Request {
        #[destructor]
        pub fn destroy();
    }

    impl Event {
        pub fn done();
        pub fn format_table(fd: fd, size: uint);
        pub fn main_device(device: array);
        pub fn tranche_done();
        pub fn tranche_target_device(device: array);
        pub fn tranche_formats(indices: array);
        pub fn tranche_flags(flags: uint<zwp_linux_dmabuf_feedback_v1.tranche_flags_enum>);
    }

    #[bitfield]
    enum TrancheFlagsEnum {
        scanout = 1,
    }
}
