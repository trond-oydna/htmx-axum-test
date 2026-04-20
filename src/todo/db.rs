use tokio::task::spawn_blocking;

use super::{NewTask, Task, TodoList};
use crate::db::{Connection, SqliteConnection};

pub fn setup(db: &Connection) -> Result<(), String> {
    db.0.lock()
        .unwrap()
        .execute(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id          INTEGER PRIMARY KEY,
                description TEXT    NOT NULL,
                completed   BOOLEAN DEFAULT false
            )
            "#,
            (),
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn select_task(conn: &SqliteConnection, id: u32) -> Task {
    let mut stmt = conn
        .prepare("SELECT id, description, completed FROM tasks WHERE id = ?1")
        .unwrap();

    stmt.query_one([id], |row| {
        Ok(Task {
            id: row.get(0)?,
            description: row.get(1)?,
            completed: row.get(2)?,
        })
    })
    .unwrap()
}

impl Connection {
    pub async fn select_all_tasks(&self) -> TodoList {
        let db = self.0.clone();
        spawn_blocking(move || {
            let db = db.lock().unwrap();
            let mut stmt = db
                .prepare("SELECT id, description, completed FROM tasks")
                .unwrap();
            let tasks: Vec<Task> = stmt
                .query_map([], |row| {
                    Ok(Task {
                        id: row.get(0)?,
                        description: row.get(1)?,
                        completed: row.get(2)?,
                    })
                })
                .unwrap()
                .collect::<Result<Vec<Task>, rusqlite::Error>>()
                .unwrap();

            TodoList { tasks }
        })
        .await
        .unwrap()
    }

    pub async fn search_tasks<'s, 'a>(&'s self, query: String) -> TodoList {
        let db = self.0.clone();
        spawn_blocking(move || {
            let db = db.lock().unwrap();
            let mut stmt = db
                .prepare("SELECT id, description, completed FROM tasks WHERE description LIKE '%' || ?1 || '%'")
                .unwrap();
            let tasks: Vec<Task> = stmt
                .query_map([&query], |row| {
                    Ok(Task {
                        id: row.get(0)?,
                        description: row.get(1)?,
                        completed: row.get(2)?,
                    })
                })
                .unwrap()
                .collect::<Result<Vec<Task>, rusqlite::Error>>()
                .unwrap();

            TodoList { tasks }
        })
        .await
        .unwrap()
    }

    pub async fn insert_task(&self, task: NewTask) -> Task {
        let db = self.0.clone();
        spawn_blocking(move || {
            let db = db.lock().unwrap();
            let mut stmt = db
                .prepare("INSERT INTO tasks (description) VALUES(?1) RETURNING id")
                .unwrap();
            let id = stmt
                .query_one([&task.description], |row| row.get::<_, u32>(0))
                .unwrap();

            Task {
                id,
                description: task.description,
                completed: false,
            }
        })
        .await
        .unwrap()
    }

    pub async fn complete_task(&self, id: u32) -> Task {
        let db = self.0.clone();
        spawn_blocking(move || {
            let db = db.lock().unwrap();

            db.execute("UPDATE tasks SET completed = true WHERE id = ?1", [id])
                .unwrap();

            select_task(&db, id)
        })
        .await
        .unwrap()
    }

    pub async fn uncomplete_task(&self, id: u32) -> Task {
        let db = self.0.clone();
        spawn_blocking(move || {
            let db = db.lock().unwrap();

            db.execute("UPDATE tasks SET completed = false WHERE id = ?1", [id])
                .unwrap();

            select_task(&db, id)
        })
        .await
        .unwrap()
    }
}
