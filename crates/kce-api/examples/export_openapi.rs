//! Export OpenAPI JSON specification to stdout or file.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use utoipa::OpenApi;

fn main() {
    let spec = kce_api::routes::ApiDoc::openapi()
        .to_pretty_json()
        .expect("failed to serialize OpenAPI specification");

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        std::fs::write(&args[1], spec).expect("failed to write openapi file");
        println!("Exported OpenAPI spec to {}", args[1]);
    } else {
        println!("{}", spec);
    }
}
