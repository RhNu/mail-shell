//! mail-shell server library
//!
//! Provides the core backend modules: database management, MIME parsing,
//! request routing, file storage, error handling, and data models.

pub mod db;
pub mod error;
pub mod mime_parser;
pub mod models;
pub mod routes;
pub mod storage;
