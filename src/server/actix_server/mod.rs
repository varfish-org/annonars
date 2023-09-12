//! Implementation of the actix server.

pub mod annos_db_info;
pub mod annos_range;
pub mod annos_variant;
pub mod error;
pub mod fetch;
pub mod genes_clinvar;
pub mod genes_info;
pub mod genes_lookup;
pub mod genes_search;

use actix_web::{middleware::Logger, web::Data, App, HttpServer};

use super::{Args, WebServerData};

/// Main entry point for the actix server.
///
/// # Errors
///
/// If the server cannot be started.
#[actix_web::main]
pub async fn main(args: &Args, dbs: Data<WebServerData>) -> std::io::Result<()> {
    HttpServer::new(move || {
        let app = App::new()
            .app_data(dbs.clone())
            .service(annos_variant::handle)
            .service(annos_range::handle)
            .service(annos_db_info::handle)
            .service(genes_clinvar::handle)
            .service(genes_info::handle)
            .service(genes_search::handle)
            .service(genes_lookup::handle);
        app.wrap(Logger::default())
    })
    .bind((args.listen_host.as_str(), args.listen_port))?
    .run()
    .await
}
