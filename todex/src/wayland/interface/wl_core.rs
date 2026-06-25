use super::*;

interface! {
    pub struct WlDisplay;

    impl Request {
        pub fn sync(callback: new_id<wl_callback>);
        pub fn get_registry(registry: new_id<wl_registry>);
    }

    impl Event {
        pub fn error(object_id: uint, code: uint, message: string);
        pub fn delete_id(id: uint);
    }

    pub enum DisplayError {
        invalid_object,
        invalid_method,
        no_memory,
        implementation,
    }
}

interface! {
    pub struct WlRegistry;

    impl Request {
        pub fn bind(name: uint, id: string, id_version: uint, new_id: uint);
    }

    impl Event {
        pub fn global(name: uint, interface: string, version: uint);
        pub fn global_remove(name: uint);
    }
}

interface! {
    pub struct WlCallback;

    impl Event {
        pub fn done(callback_data: uint);
    }
}
