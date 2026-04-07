use axum::{
    Router,
    extract::{FromRef, State},
    routing::get,
};
use maud::{DOCTYPE, Markup, html};
use tower_http::services::ServeDir;

use crate::{db::DbConnection, todo};

#[derive(Clone, Debug)]
pub struct ServerState {
    db: DbConnection,
}

impl ServerState {
    fn new(db: DbConnection) -> Self {
        ServerState { db }
    }
}

impl FromRef<ServerState> for DbConnection {
    fn from_ref(state: &ServerState) -> Self {
        state.db.clone()
    }
}

pub async fn start(db: DbConnection) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9090")
        .await
        .unwrap();
    let state = ServerState::new(db);

    axum::serve(listener, app(state)).await.unwrap();
}

fn app(state: ServerState) -> Router {
    Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .nest("/todo", todo::http::route())
        .route("/", get(index))
        .with_state(state)
}

#[axum::debug_handler]
async fn index(State(db): State<DbConnection>) -> Markup {
    let list = db.select_all_tasks().await;

    html! {
        (DOCTYPE)
        html class="min-w-full min-h-full" {
            head {
                script src="/static/htmx@2.0.8.js" {}
                script src="/static/tailwindcss@4.2.2.js" {}
                link rel="stylesheet" type="text/css" href={"/static/index.css?t=" (crate::COMPILE_CURRENT_TIME)};
                link rel="stylesheet" type="text/css" href="/static/daisyui@5.5.18.css";
                link rel="icon" href="/static/favicon.png";
                meta name="htmx-config" content="{\"historyEnabled\": false}";
            }
            body class="min-w-full min-h-full h-screen p-4 bg-gray-100" {
                section class="max-w-xl mx-auto grid grid-cols-1 gap-2 px-4 rounded-md bg-white" {
                    h1 class="text-3xl text-center p-4" {
                        "HTMX Axum TODO"
                    }
                    (crate::todo::http::create())
                    div class="divider h-2 m-0" {}
                    (crate::todo::http::list(&list))
                }
            }
        }
    }
}
