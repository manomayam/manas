//! I define few implementations of [`HttpService`](super::HttpService).
//!

// TODO implement layers.

mod normal_validate_target_uri;
mod overriding;
mod reconstruct_target_uri;
mod route_by_method;

pub use normal_validate_target_uri::NormalValidateTargetUri;
pub use overriding::OverridingHttpService;
pub use reconstruct_target_uri::ReconstructTargetUri;
pub use route_by_method::RouteByMethod;
