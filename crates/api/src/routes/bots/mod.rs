use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;
use rocket::Route;

mod create;
mod delete;
mod edit;
mod fetch;
mod fetch_owned;
mod fetch_public;
mod invite;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![]
}
