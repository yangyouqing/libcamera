#![no_std]

extern crate alloc;

mod fixed_vec;
mod fixed_string;
mod error;
mod types;
pub mod frame;
pub mod logging;

pub use fixed_vec::FixedVec;
pub use fixed_string::FixedString;
pub use error::{CamError, CommResult};
pub use types::{ServiceId, ServiceState, Topic, MethodId, CtrlMsg, AuthLevel, HealthStatus};
pub use frame::FrameHeader;
