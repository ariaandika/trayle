//! DRM Resource.
pub use object_type::ObjectType;
pub use traits::Resource;
pub use error::ResourceError;
pub use resource::Resources;

mod object_type;
mod traits;
mod error;
mod resource;
