use std::{convert::Infallible, fmt, sync::Arc, time::Duration};

use axum::{
    extract::{Query, State},
    response::sse::KeepAlive,
};
use futures::stream::Stream;
use maud::html;
use serde::Deserialize;
use tokio::sync::broadcast::{self, Sender};
use tokio_stream::{
    StreamExt,
    wrappers::{BroadcastStream, errors::BroadcastStreamRecvError},
};
use uuid::Uuid;

use crate::todo::Task;

type AxumSseEvent = axum::response::sse::Event;
type AxumSse<S> = axum::response::Sse<S>;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct ClientId(Uuid);

impl ClientId {
    pub fn new() -> Self {
        ClientId(Uuid::now_v7())
    }
}

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[derive(Deserialize)]
pub struct SseQueryParams {
    client_id: ClientId,
}

pub async fn sse_handler(
    Query(params): Query<SseQueryParams>,
    State(broadcaster): State<Broadcaster>,
) -> AxumSse<impl Stream<Item = Result<AxumSseEvent, Infallible>>> {
    let stream = broadcaster.subscribe(params.client_id);

    AxumSse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("keep-alive"),
    )
}

#[derive(Clone, Debug)]
pub struct Broadcaster {
    sender: Arc<Sender<Event>>,
}

impl Broadcaster {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(32);
        Broadcaster {
            sender: Arc::new(sender),
        }
    }

    pub fn send(&self, event: Event) {
        // Only error is no receivers
        let _ = self.sender.send(event);
    }

    fn subscribe(
        &self,
        client_id: ClientId,
    ) -> impl Stream<Item = Result<AxumSseEvent, Infallible>> + use<> {
        let rx = self.sender.subscribe();

        BroadcastStream::new(rx).filter_map(move |result| match result {
            Err(BroadcastStreamRecvError::Lagged(_)) => None,
            Ok(event) if (event.get_client_id() == client_id) => None,
            Ok(event) => Some(Ok(event.into_sse_event())),
        })
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    TaskCreated(ClientId, Task),
    TaskUpdated(ClientId, Task),
}

impl Event {
    pub fn task_created(client_id: ClientId, task: Task) -> Self {
        Event::TaskCreated(client_id, task)
    }

    pub fn task_updated(client_id: ClientId, task: Task) -> Self {
        Event::TaskUpdated(client_id, task)
    }

    fn get_client_id(&self) -> ClientId {
        match self {
            Event::TaskCreated(client_id, _) => *client_id,
            Event::TaskUpdated(client_id, _) => *client_id,
        }
    }

    fn into_sse_event(self) -> AxumSseEvent {
        let markup = match self {
            Event::TaskCreated(_, task) => html! {
                hx-partial hx-target="#todos tbody" hx-swap="beforeend" {
                    (crate::todo::http::row(&task))
                }
            },
            Event::TaskUpdated(_, task) => html! {
                hx-partial hx-target={"#task-" (task.id)} hx-swap="outerHTML" {
                    (crate::todo::http::row(&task))
                }
            },
        };

        AxumSseEvent::default().data(markup.into_string())
    }
}
