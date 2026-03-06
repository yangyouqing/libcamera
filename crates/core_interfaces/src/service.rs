use core_types::{CommResult, HealthStatus, ServiceId};
use crate::CommBus;

/// Trait for all camera system services.
/// Each service declares its dependencies for staged startup (topological sort).
pub trait Service {
    fn service_id(&self) -> ServiceId;

    /// Services that must be started before this one.
    fn dependencies(&self) -> &'static [ServiceId];

    /// Initialize with a reference to the communication bus.
    fn init(&mut self, bus: &dyn CommBus) -> CommResult<()>;

    /// Start the service (after init and all dependencies are started).
    fn start(&mut self) -> CommResult<()>;

    /// Stop the service gracefully.
    fn stop(&mut self) -> CommResult<()>;

    /// Report current health/degradation status.
    fn health(&self) -> HealthStatus;

    /// Called periodically from the service's run loop. Returns true if work was done.
    fn poll(&mut self) -> CommResult<bool> {
        Ok(false)
    }
}
