use crate::{http, logger, rand, time, workflow::State};

wasmtime::component::bindgen!({
    world: "workflow",
    path: "crates/workflow/wit/world.wit",
    trappable_imports: false,
    async: true,
    additional_derives: [
        serde::Deserialize,
        serde::Serialize,
    ],
});

impl component::workflow::abi::Host for State {}
impl component::workflow::http::Host for State {}
impl WorkflowImports for State {
    fn call<'life0, 'async_trait>(
        &'life0 mut self,
        request: GuestToHost,
    ) -> ::core::pin::Pin<
        Box<dyn ::core::future::Future<Output = HostToGuest> + ::core::marker::Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            match request {
                GuestToHost::Input => todo!(),
                GuestToHost::Random(_) => rand::call(self, request).await.unwrap(),
                GuestToHost::Log(_) => logger::call(self, request).await.unwrap(),
                GuestToHost::HttpRequest(_) => http::call(self, request).await.unwrap(),
                GuestToHost::Time => time::call(self, request).await.unwrap(),
            }
        })
    }
}
