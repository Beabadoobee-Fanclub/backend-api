use sea_query::{Value, Values};
use tokio_postgres::types::ToSql;
use worker::{console_error, postgres_tls, Error, Hyperdrive, Result, SecureTransport, Socket};

#[derive(Debug)]
pub struct Database {
    hyperdrive: Hyperdrive,
}

impl Database {
    pub fn new(hyperdrive: Hyperdrive) -> Self {
        Database { hyperdrive }
    }
    pub async fn connect_to_db(&self) -> Result<tokio_postgres::Client> {
        let hyperdrive = &self.hyperdrive;
        let config = hyperdrive
            .connection_string()
            .parse::<tokio_postgres::Config>()
            .map_err(|e| Error::RustError(format!("Failed to parse connection string: {}", e)))?;

        let socket = Socket::builder()
            .secure_transport(SecureTransport::StartTls)
            .connect(hyperdrive.host(), hyperdrive.port())?;

        let (client, connection) = config
            .connect_raw(socket, postgres_tls::PassthroughTls)
            .await
            .map_err(|e| Error::RustError(format!("Failed to connect to database: {}", e)))?;

        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = connection.await {
                console_error!("Database connection error: {}", e);
            }
        });

        Ok(client)
    }
    pub fn convert_params(values: Values) -> Result<Vec<Box<dyn ToSql + Sync>>> {
        let mut params: Vec<Box<dyn ToSql + Sync>> = Vec::with_capacity(values.0.len());

        for v in values.0 {
            match v {
                Value::Bool(Some(b)) => params.push(Box::new(b)),
                Value::Int(Some(i)) => params.push(Box::new(i)),
                Value::BigInt(Some(i)) => params.push(Box::new(i)),
                Value::TinyInt(Some(i)) => params.push(Box::new(i)),
                Value::SmallInt(Some(i)) => params.push(Box::new(i)),
                Value::Char(Some(c)) => params.push(Box::new(c.to_string())),
                Value::Double(Some(f)) => params.push(Box::new(f)),
                Value::Float(Some(f)) => params.push(Box::new(f)),
                Value::String(Some(s)) => params.push(Box::new((*s).clone())),
                Value::Bytes(Some(b)) => params.push(Box::new((*b).clone())),
                _ => return Err("Unsupported or NULL parameter".into()),
            }
        }

        Ok(params)
    }
}
