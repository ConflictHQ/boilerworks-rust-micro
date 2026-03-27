pub mod auth;
pub mod config;
pub mod db;
pub mod handlers;
pub mod models;
pub mod response;
pub mod routes;

pub use routes::build_router;
