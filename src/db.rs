use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::todo;

#[derive(Clone, Debug)]
pub struct DbConnection(pub Arc<Mutex<Connection>>);

pub fn connect() -> Result<DbConnection, String> {
    let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;

    todo::db::setup(&conn)?;

    Ok(DbConnection(Arc::new(Mutex::new(conn))))
}
