use std::net::TcpListener;

use sqlx::{Connection, PgConnection, PgPool, Executor};
use uuid::Uuid;
use zero2prod::configuration::{DatabaseSettings, get_configuration};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    // nos quedamos con el listener
    // el puerto 0 nos da un puerto random disponible por el SO
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind a random port");

    // nos quedamos con el puerto asignado por el SO
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    let server =
        zero2prod::startup::run(listener, connection_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    // retornamos el address del caller
    TestApp {
        address,
        db_pool: connection_pool,
    }
}
pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("failed to create database");

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}

#[tokio::test]
async fn health_check_succeds() {
    // arrange
    let test_app = spawn_app().await;
    // let configuration = get_configuration().expect("failed to read configuration");
    // let connection_string = configuration.database.connection_string();
    // el trait Connection debe estar presente en el crate
    // no es un metodo de una struc
    // creamos la conexion a la database con nuestra aplicacion
    // eso es lo que comprobamos con esta linea que se haga la conexion
    // let mut connection = PgConnection::connect(&connection_string)
    //     .await
    //     .expect("failed to connect postgres");
    let client = reqwest::Client::new();

    // act
    let response = client
        //usamos el address de la aplicacion
        .get(&format!("{}/health_check", &test_app.address))
        .send()
        .await
        .expect("failed to execute request.");

    // assert
    assert!(response.status().is_success());
}

#[tokio::test]
async fn suscribe_resturns_a_200_for_valid_form_data() {
    //Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    // let configuration = get_configuration().expect("failed to read configuration");
    // let connection_string = configuration.database.connection_string();
    // let mut connection = PgConnection::connect(&connection_string)
    //     .await
    //     .expect("failed to connect postgres");

    //Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("failed to execute request");
    //Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("failed to fetch saved suscriptions");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn suscribe_returns_a_400_when_data_is_missing() {
    //arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20gui", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    //Act
    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("failed to execute request");

        //Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            //Mensaje adicional de porqué falla
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message
        )
    }
}
