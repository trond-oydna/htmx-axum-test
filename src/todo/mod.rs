pub mod db;
pub mod http;

#[derive(Debug)]
pub struct TodoList {
    pub tasks: Vec<Task>,
}

#[derive(Clone, Debug)]
pub struct Task {
    pub id: u32,
    pub description: String,
    pub completed: bool,
}

#[derive(Clone, Debug)]
pub struct NewTask {
    pub description: String,
}
