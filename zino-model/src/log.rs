use serde::{Deserialize, Serialize};
use zino_core::{datetime::DateTime, model::Model, request::Validation, Map, Uuid};
use zino_derive::Schema;

/// The log model.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Schema)]
#[serde(rename_all = "snake_case")]
#[serde(default)]
pub struct Log {
    // Basic fields.
    #[schema(readonly)]
    id: Uuid,
    #[schema(not_null, index = "text")]
    name: String,
    #[schema(default = "Log::model_namespace", index = "hash")]
    namespace: String,
    #[schema(default = "internal")]
    visibility: String,
    #[schema(default = "active", index = "hash")]
    status: String,
    #[schema(index = "text")]
    description: String,

    // Info fields.
    #[schema(not_null, readonly)]
    service: String,
    #[schema(readonly)]
    server_host: String,
    #[schema(readonly)]
    client_ip: String,
    topic: String,
    #[schema(readonly)]
    level: String,
    #[schema(readonly, index = "text")]
    message: String,
    #[schema(readonly)]
    source: String,
    #[schema(readonly, index = "btree")]
    recorded_at: DateTime,

    // Extensions.
    #[schema(readonly)]
    content: Map,
    metrics: Map,
    extras: Map,

    // Revisions.
    manager_id: Uuid,    // user.id
    maintainer_id: Uuid, // user.id
    #[schema(readonly, default = "now", index = "btree")]
    created_at: DateTime,
    #[schema(default = "now", index = "btree")]
    updated_at: DateTime,
    version: u64,
    edition: u32,
}

impl Model for Log {
    #[inline]
    fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            ..Self::default()
        }
    }

    fn read_map(&mut self, data: &Map) -> Validation {
        let mut validation = Validation::new();
        if let Some(result) = Validation::parse_uuid(data.get("id")) {
            match result {
                Ok(id) => self.id = id,
                Err(err) => validation.record_fail("id", err),
            }
        }
        if let Some(name) = Validation::parse_string(data.get("name")) {
            self.name = name.into_owned();
        }
        if self.name.is_empty() {
            validation.record("name", "should be nonempty");
        }
        validation
    }
}

super::impl_model_accessor!(
    Log,
    id,
    name,
    namespace,
    visibility,
    status,
    description,
    content,
    metrics,
    extras,
    manager_id,
    maintainer_id,
    created_at,
    updated_at,
    version,
    edition
);
