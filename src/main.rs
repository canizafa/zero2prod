use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // paniquea si no puede leer la configuracion
    let configuration = get_configuration().expect("Failed to read configuration");

    // necesitamos una pool de conexiones para poder manejar el Excutor trait en nuestro estado de la app
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("failed to connect to postgres,");

    //removemos los hardcodeado
    let address = format!("127.0.0.1:{}", configuration.application_port);

    // creamos el listener
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
