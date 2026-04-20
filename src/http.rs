use axum::{
    Router,
    extract::{FromRef, State},
    routing::get,
};
use maud::{DOCTYPE, Markup, html};
use tower_http::services::ServeDir;

use crate::{
    db,
    sse::{self, ClientId},
    todo,
};

#[derive(Clone, Debug)]
pub struct ServerState {
    db: db::Connection,
    sse: sse::Broadcaster,
}

impl ServerState {
    fn new(db: db::Connection, sse: sse::Broadcaster) -> Self {
        ServerState { db, sse }
    }
}

impl FromRef<ServerState> for db::Connection {
    fn from_ref(state: &ServerState) -> Self {
        state.db.clone()
    }
}

impl FromRef<ServerState> for sse::Broadcaster {
    fn from_ref(state: &ServerState) -> Self {
        state.sse.clone()
    }
}

pub async fn start(db: db::Connection, sse: sse::Broadcaster) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9090")
        .await
        .unwrap();
    let state = ServerState::new(db, sse);

    axum::serve(listener, app(state)).await.unwrap();
}

fn app(state: ServerState) -> Router {
    Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .route("/", get(index))
        .nest("/todo", todo::http::route())
        .route("/sse", get(sse::sse_handler))
        .with_state(state)
}

#[axum::debug_handler]
async fn index(State(db): State<db::Connection>) -> Markup {
    let list = db.select_all_tasks().await;
    let client_id = ClientId::new();
    let sse_url = format!("/sse?client_id={}", client_id);

    html! {
        (DOCTYPE)
        html class="min-w-full min-h-full" {
            head {
                script src="/static/htmx@2.0.8.js" {}
                script src="/static/htmx-sse@2.2.4.js" {}
                script src="/static/tailwindcss@4.2.2.js" {}
                link rel="stylesheet" type="text/css" href={"/static/index.css?t=" (crate::COMPILE_CURRENT_TIME)};
                link rel="stylesheet" type="text/css" href="/static/daisyui@5.5.18.css";
                link rel="icon" href="/static/favicon.png";
                meta name="htmx-config" content="{\"historyEnabled\": false}";
            }
            body class="min-w-full min-h-full h-screen p-4 bg-gray-100" {
                script {
                    (maud::PreEscaped(format!(r#"
                    document.body.addEventListener('htmx:configRequest', function(e) {{
                        e.detail.path += "?client_id={}";
                    }});
                    "#, client_id)))
                }
                section class="max-w-xl mx-auto grid grid-cols-1 gap-2 px-4 rounded-md bg-white"
                        hx-ext="sse" sse-connect=(sse_url) {
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
