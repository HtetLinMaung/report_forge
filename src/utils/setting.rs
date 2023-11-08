use std::{env, sync::Arc};

use tokio_postgres::{Client, NoTls};

pub fn get_port() -> u16 {
    let port: u16 = env::var("PORT")
        .unwrap_or(String::from("8080"))
        .parse()
        .expect("Port must be number");
    port
}

pub async fn get_postgres_connection() -> Arc<Client> {
    let conn = env::var("DB_CONNECTION").expect("DB_CONNECTION must be set");
    let (client, connection) = tokio_postgres::connect(conn.as_str(), NoTls).await.unwrap();

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    Arc::new(client)
}
