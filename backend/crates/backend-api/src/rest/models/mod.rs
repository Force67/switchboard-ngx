//! REST API models (DTOs) for requests and responses
//!
//! This module centralizes all data transfer objects used in the REST API,
//! eliminating duplicate type definitions across handler files.

pub mod attachment;
pub mod auth;
pub mod chat;
pub mod chat_completion;
pub mod common;
pub mod folder;
pub mod health;
pub mod invite;
pub mod member;
pub mod message;
pub mod model;
pub mod notification;
pub mod permission;

// Re-export all types for convenience
pub use attachment::*;
pub use auth::*;
pub use chat::*;
pub use chat_completion::*;
pub use common::*;
pub use folder::*;
pub use health::*;
pub use invite::*;
pub use member::*;
pub use message::*;
pub use model::*;
pub use notification::*;
pub use permission::*;
