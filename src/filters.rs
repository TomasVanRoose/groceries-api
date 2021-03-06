use crate::database::Db;
use crate::handlers;
use crate::models::{GroceryItem, NewGroceryItem};
use warp::Filter;

//// Mount all CRUD methods
pub fn grocery_items(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    items_list(db.clone())
        .or(item_create(db.clone()))
        .or(item_read(db.clone()))
        .or(item_update(db.clone()))
        .or(item_delete(db.clone()))
}

/// GET /items
fn items_list(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("items")
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::all_grocery_items)
}

/// GET /items/:id
fn item_read(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("items" / i32)
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::read_grocery_item)
}
/// POST /items
fn item_create(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("items")
        .and(warp::post())
        .and(json_body_partial())
        .and(with_db(db))
        .and_then(handlers::create_grocery_item)
}
/// PUT /items/:id
fn item_update(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("items" / i32)
        .and(warp::put())
        .and(json_body())
        .and(with_db(db))
        .and_then(handlers::update_grocery_item)
}
/// DELETE /items/:id
fn item_delete(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("items" / i32)
        .and(warp::delete())
        .and(with_db(db))
        .and_then(handlers::delete_grocery_item)
}

fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn json_body() -> impl Filter<Extract = (GroceryItem,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

/// When creating a new GroceryItem, some fields (like id) are autogenerated by the database,
/// so we only need some values
fn json_body_partial() -> impl Filter<Extract = (NewGroceryItem,), Error = warp::Rejection> + Clone
{
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
