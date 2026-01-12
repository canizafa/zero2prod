#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
  pub username: String,
  pub password: String,
  pub host: String,
  pub port: u16,
  pub database_name: String,
}

impl DatabaseSettings {
  // creamos la connection string que nos pide pgconnection
  pub fn connection_string(&self) -> String {
    format!(
      "postgres://{}:{}@{}:{}/{}",
      self.username, self.password, self.host, self.port, self.database_name
    )
  }
  pub fn connection_string_without_db(&self) -> String {
    format!(
      "postgres://{}:{}@{}:{}",
      self.username, self.password, self.host, self.port
    )
  }
}


pub fn get_configuration() -> Result<Settings, config::ConfigError> {
  // se inicia la configuracion
  let settings = config::Config::builder()
    //agregamos la fuente de la configuration 'configuration.yaml'
    .add_source(
      config::File::new("configuration.yaml", config::FileFormat::Yaml)
    )
    .build()?;
  // trata de convertir los valores en nuestro tipo settings
  settings.try_deserialize::<Settings>()
}