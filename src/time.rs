use crate::{
    guest::{
        component::workflow::abi::{Content, SystemTime},
        GuestToHost, HostToGuest,
    },
    workflow::State,
};
use anyhow::Result;
use std::time::UNIX_EPOCH;
use uuid::Uuid;

pub async fn call(state: &mut State, request: GuestToHost) -> Result<HostToGuest> {
    match request {
        GuestToHost::Time => Ok(state
            .retrieve_or_else(|| async {
                let duration_since_epoch =
                    std::time::SystemTime::now().duration_since(UNIX_EPOCH)?;
                Ok(HostToGuest {
                    id: Uuid::new_v4().into(),
                    content: Content::Time(SystemTime {
                        tv_sec: duration_since_epoch.as_secs(),
                        tv_nsec: duration_since_epoch.subsec_nanos(),
                    }),
                })
            })
            .await?),
        _ => unreachable!(),
    }
}
