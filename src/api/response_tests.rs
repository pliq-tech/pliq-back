use axum::body::Body;
use axum::response::Response;
use http_body_util::BodyExt;
use serde_json::Value;

use super::response::{json_created, json_data, json_paginated};

async fn body_json(response: Response<Body>) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn json_data_returns_200_with_envelope() {
    let response = json_data("hello");

    assert_eq!(response.status(), 200);

    let json = body_json(response).await;
    assert_eq!(json["data"], "hello");
    assert!(json.get("meta").is_none());
}

#[tokio::test]
async fn json_created_returns_201_with_envelope() {
    let response = json_created(serde_json::json!({"id": 1}));

    assert_eq!(response.status(), 201);

    let json = body_json(response).await;
    assert_eq!(json["data"]["id"], 1);
    assert!(json.get("meta").is_none());
}

#[tokio::test]
async fn json_paginated_returns_200_with_meta() {
    let items = vec!["a", "b", "c"];
    let response = json_paginated(items, 1, 10, 42);

    assert_eq!(response.status(), 200);

    let json = body_json(response).await;
    assert_eq!(json["data"], serde_json::json!(["a", "b", "c"]));

    let meta = &json["meta"];
    assert_eq!(meta["page"], 1);
    assert_eq!(meta["limit"], 10);
    assert_eq!(meta["total"], 42);
}

#[tokio::test]
async fn json_data_serializes_struct() {
    #[derive(serde::Serialize)]
    struct User {
        name: String,
        age: u32,
    }

    let user = User {
        name: "Alice".to_string(),
        age: 30,
    };
    let response = json_data(user);

    assert_eq!(response.status(), 200);

    let json = body_json(response).await;
    assert_eq!(json["data"]["name"], "Alice");
    assert_eq!(json["data"]["age"], 30);
}
