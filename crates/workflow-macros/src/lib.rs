use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn workflow(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = match syn::parse::<syn::ItemFn>(item.clone()) {
        Ok(item_fn) => item_fn,
        Err(err) => return token_stream_with_error(item, err),
    };

    let block = function.block;

    quote! {
        pub use workflow::bindings::{self, Guest};

        struct Component;

        impl Guest for Component {
            fn execute() -> Result<()> {
                workflow::logger::init(log::LevelFilter::Trace).unwrap();

                #block
            }
        }

        workflow::bindings::export!(Component with_types_in bindings);
    }
    .into()
}

fn token_stream_with_error(mut tokens: TokenStream, error: syn::Error) -> TokenStream {
    tokens.extend(TokenStream::from(error.into_compile_error()));
    tokens
}
