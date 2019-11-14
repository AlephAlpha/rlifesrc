mod cells;
mod config;
pub mod rules;
mod search;
mod world;

pub use cells::State;
pub use config::{Config, NewState, SearchOrder, Symmetry, Transform};
pub use search::{set_world, Search, Status};
pub use world::World;
