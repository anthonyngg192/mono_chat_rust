mod generic;
mod mongo;
#[cfg(feature = "rocket_impl")]
mod rocket;

pub use mongo::MongoDb;
