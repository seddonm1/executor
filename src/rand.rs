use crate::{
    guest::{
        component::workflow::abi::{Content, Types},
        GuestToHost, HostToGuest,
    },
    workflow::State,
};
use anyhow::Result;
use rand::Rng;
use uuid::Uuid;

pub async fn call(state: &mut State, request: GuestToHost) -> Result<HostToGuest> {
    let rng = state.rng.clone();
    match request {
        GuestToHost::Random(ty) => match ty {
            Types::BoolType(_) => Ok(state
                .retrieve_or_else(|| async {
                    let mut rng = rng.lock().unwrap();
                    Ok(HostToGuest {
                        id: Uuid::new_v4().into(),
                        content: Content::Value(Types::BoolType(rng.gen::<bool>())),
                    })
                })
                .await?),
            Types::I32Type(_) => Ok(state
                .retrieve_or_else(|| async {
                    let mut rng = rng.lock().unwrap();
                    Ok(HostToGuest {
                        id: Uuid::new_v4().into(),
                        content: Content::Value(Types::I32Type(rng.gen::<i32>() as u32)),
                    })
                })
                .await?),
            Types::F32Type(_) => Ok(state
                .retrieve_or_else(|| async {
                    let mut rng = rng.lock().unwrap();
                    Ok(HostToGuest {
                        id: Uuid::new_v4().into(),
                        content: Content::Value(Types::F32Type(rng.gen::<f32>())),
                    })
                })
                .await?),
            Types::StringType(_) => unreachable!(),
        },
        _ => unreachable!(),
    }
}
