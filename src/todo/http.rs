use axum::{
    Form, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::post,
};
use maud::{Markup, html};
use serde::Deserialize;
use tokio::time::sleep;

use crate::{
    ENDPOINT_DELAY,
    db::DbConnection,
    http::ServerState,
    todo::{NewTask, Task, TodoList},
};

pub fn create() -> Markup {
    html! {
        form id="create-task-form" class="flex content-stretch gap-2" {
            label class="input grow-2" {
                input
                    type="text"
                    name="description"
                    placeholder="Description"
                    ;
            }
            input
                type="submit"
                value="Add"
                class="btn"
                hx-post="/todo"
                hx-trigger="click"
                hx-target="#todos tbody"
                hx-swap="beforeend"
                hx-on::before-request=r#"
                    this.disabled = true;
                    "#
                hx-on::after-request=r#"
                    if(event.detail.successful) {
                        document.getElementById("create-task-form").reset();
                        this.disabled = false;
                    }
                    "#
                ;
        }
    }
}

pub fn list(state: &TodoList) -> Markup {
    html! {
        input
            type="search"
            name="search"
            class="input w-full"
            placeholder="Begin Typing To Search Users..."
            hx-post="/todo/search"
            hx-trigger="input changed delay:100ms, keyup[key=='Enter']"
            hx-target="#todos tbody"
            ;

        table id="todos" class="table table-zebra pb-4" {
            thead {
                tr {
                    td {
                        "Description"
                    }
                    td {
                        "Complete"
                    }
                }
            }
            tbody {
                @if state.tasks.is_empty() {
                    // TODO: placeholder
                } @else {
                    @for task in &state.tasks {
                        (row(task))
                    }
                }
            }
         }
    }
}

pub fn route() -> Router<ServerState> {
    Router::new()
        .route("/", post(add_task))
        .route("/search", post(search_task))
        .route("/{id}/complete", post(complete_task))
        .route("/{id}/uncomplete", post(uncomplete_task))
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct AddTaskFormInput {
    description: String,
}

#[axum::debug_handler]
pub async fn add_task(
    State(db): State<DbConnection>,
    Form(input): Form<AddTaskFormInput>,
) -> impl IntoResponse {
    sleep(ENDPOINT_DELAY).await;
    let task = NewTask {
        description: input.description,
    };
    let task = db.insert_task(task).await;

    row(&task)
}

#[axum::debug_handler]
pub async fn complete_task(
    State(db): State<DbConnection>,
    Path(id): Path<u32>,
) -> impl IntoResponse {
    let task = db.complete_task(id).await;

    row(&task)
}

#[axum::debug_handler]
pub async fn uncomplete_task(
    State(db): State<DbConnection>,
    Path(id): Path<u32>,
) -> impl IntoResponse {
    sleep(ENDPOINT_DELAY).await;
    let task = db.uncomplete_task(id).await;

    row(&task)
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct SearchTaskFormInput {
    search: String,
}

#[axum::debug_handler]
pub async fn search_task(
    State(db): State<DbConnection>,
    Form(input): Form<SearchTaskFormInput>,
) -> impl IntoResponse {
    sleep(ENDPOINT_DELAY).await;
    let list = db.search_tasks(input.search).await;

    html! {
        @for task in &list.tasks {
            (row(task))
        }
    }
}

fn row(task: &Task) -> Markup {
    let complete_path = if task.completed {
        "uncomplete"
    } else {
        "complete"
    };

    html! {
        tr id={"task-" (task.id)} {
            td {
                (task.description)
            }
            td {
                input
                    type="checkbox"
                    class="checkbox"
                    checked?[task.completed]
                    hx-post={"/todo/" (task.id) "/" (complete_path)}
                    hx-trigger="click"
                    hx-target={"#task-" (task.id)}
                    hx-swap="outerHTML"
                    hx-on::before-request=r#"
                        this.disabled = true;
                        "#
                    hx-on::after-request=r#"
                        if(event.detail.successful) this.disabled = false;
                        "#
                    ;
            }
        }
    }
}
