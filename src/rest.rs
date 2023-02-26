use crate::storage::StorageServiceMessage;
use std::convert::Infallible;
use warp::Filter;

type Database = tokio::sync::mpsc::Sender<StorageServiceMessage>;

fn with_channel(
    channel: Database,
) -> impl Filter<Extract = (Database,), Error = Infallible> + Clone {
    warp::any().map(move || channel.clone())
}

pub fn routes(
    db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "v1" / "health")
        .and(warp::get())
        .and_then(handlers::health)
        .or(warp::path!("api" / "v1" / "ocupation" / String)
            .and(warp::get())
            .and(with_channel(db))
            .and_then(|id, db: Database| async move { handlers::occupation(id, db.clone()).await }))
        .with(warp::log("api"))
}

mod handlers {
    use super::*;
    use crate::Sample;

    pub async fn health() -> Result<impl warp::Reply, Infallible> {
        Ok(warp::reply::json(&model::Health { running: true }))
    }

    pub async fn occupation(
        identifier: String,
        db: Database,
    ) -> Result<impl warp::Reply, Infallible> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        db.send(StorageServiceMessage::Fetch {
            identifier,
            response_channel: tx,
        })
        .await
        .unwrap();
        let db_response = rx.await.unwrap();
        Ok(warp::reply::json(
            &db_response
                .iter()
                .map(|sample| {
                    let Sample {
                        timestamp, value, ..
                    }: &Sample = sample;
                    model::OccupationSample {
                        timestamp: *timestamp,
                        value: *value,
                    }
                })
                .collect::<Vec<_>>(),
        ))
    }
}

mod model {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Health {
        pub running: bool,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct OccupationSample {
        pub timestamp: u64,
        pub value: f32,
    }
}
