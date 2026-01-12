use crate::routes::{suscribe, health_check};
use sqlx::PgPool;
use std::net::TcpListener; // el listener nos da un socketaddr

use actix_web::{App, HttpServer, dev::Server, web};



// Nombramos run() como publica porque es el entry point del binario
pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // wrapeamos nuestra conexion con un ARC pointer
    let db_pool = web::Data::new(db_pool);
    // me llega un socket TCP
    // y yo lo uso para levantar un servidor HTTP
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(suscribe))
            // agregamos estado a la app
            // toda la app va a tener la conexion
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    // sin el await, sino se queda esperando para siempre
    Ok(server)
}
