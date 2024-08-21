use crate::bindings::{
    self,
    component::workflow::abi::{Content, GuestToHost, HostToGuest},
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Returns the current system time.
///
/// This function makes a call to the host environment to retrieve the current time.
/// It then converts the received time information into a `SystemTime` instance.
///
/// # Returns
///
/// A `SystemTime` representing the current system time.
///
/// # Panics
///
/// This function will panic if the host returns an unexpected response format.
///
/// # Example
///
/// ```
/// let current_time = now();
/// println!("Current time: {:?}", current_time);
/// ```
pub fn now() -> SystemTime {
    match bindings::call(&GuestToHost::Time) {
        HostToGuest {
            content: Content::Time(system_time),
            ..
        } => {
            // Convert the received time information to a SystemTime instance
            UNIX_EPOCH
                + Duration::from_secs(system_time.tv_sec)
                + Duration::from_nanos(system_time.tv_nsec.into())
        }
        m => {
            // Log an error and panic if an unexpected response is received
            log::error!("expected Content::Time got {:?}", m);
            unreachable!()
        }
    }
}
