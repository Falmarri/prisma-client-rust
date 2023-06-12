mod client;
mod composite_types;
mod enums;
mod header;
mod internal_enums;
mod models;
mod types;
mod write_params;

use prisma_client_rust_sdk::prelude::*;
use serde::Serialize;

fn default_module_path() -> String {
    "crate::prisma".to_string()
}

#[derive(serde::Deserialize)]
pub struct PrismaClientRustGenerator {
    #[serde(default = "default_module_path")]
    module_path: String,
}

#[derive(Debug, Serialize, thiserror::Error)]
pub enum Error {
    #[error("Failed to parse module_path")]
    InvalidModulePath,
}

impl PrismaGenerator for PrismaClientRustGenerator {
    const NAME: &'static str = "Prisma Client Rust";
    const DEFAULT_OUTPUT: &'static str = "../src/prisma.rs";

    type Error = Error;

    fn generate(self, args: GenerateArgs) -> Result<String, Self::Error> {
        let header = header::generate(&args);

        let module_path = self
            .module_path
            .parse()
            .map_err(|_| Error::InvalidModulePath)?;

        let models = models::modules(&args, &module_path);
        let composite_types = composite_types::modules(&args, &module_path);

        let client = client::generate(&args);
        let internal_enums = internal_enums::generate(&args);
        let write_params_module = write_params::generate_module(&args);
        let types = types::types(&args);

        let enums = enums::generate(&args);

        let tokens = quote! {
            #header

            #(#models)*
            #(#composite_types)*

            pub mod _prisma {
                #client
                #internal_enums
                #write_params_module
                pub mod types {
                    use super::*;

                    #types
                }
                pub use types::*;
            }

            pub use _prisma::*;

            #enums
        };

        Ok(tokens.to_string())
    }
}
