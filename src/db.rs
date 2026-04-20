use std::sync::{Arc, Mutex};

use crate::todo;

pub type SqliteConnection = rusqlite::Connection;

#[derive(Clone, Debug)]
pub struct Connection(pub Arc<Mutex<SqliteConnection>>);

pub fn connect() -> Result<Connection, String> {
    let db = SqliteConnection::open_in_memory().map_err(|e| e.to_string())?;
    let db = Connection(Arc::new(Mutex::new(db)));

    todo::db::setup(&db)?;

    Ok(db)
}
