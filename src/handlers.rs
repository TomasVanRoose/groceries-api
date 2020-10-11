use crate::database::Db;
use crate::models::{GroceryItem, NewGroceryItem};
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::Reply;

pub async fn create_grocery_item(
    new_item: NewGroceryItem,
    db: Db,
) -> Result<impl warp::Reply, Infallible> {
    log::debug!("create_grocery_item: {:?}", new_item);

    Ok(sqlx::query!(
        r#"INSERT INTO items (name, checked_off, position)
            VALUES($1, $2, $3)"#,
        new_item.name,
        new_item.checked_off,
        new_item.position as i32,
    )
    .execute(db.database())
    .await
    .map_or_else(|_| StatusCode::BAD_REQUEST, |_| StatusCode::CREATED))
}

pub async fn read_grocery_item(id: i32, db: Db) -> Result<impl warp::Reply, Infallible> {
    log::debug!("read_grocery_item for id: {:?}", id);

    Ok(sqlx::query_as!(
        GroceryItem,
        r#"
                SELECT id, name, checked_off, position, created_at
                FROM items 
                WHERE id = $1
            "#,
        id
    )
    .fetch_one(db.database())
    .await
    .map_or_else(
        |_| StatusCode::NOT_FOUND.into_response(),
        |item| warp::reply::json(&item).into_response(),
    ))
}
pub async fn all_grocery_items(db: Db) -> Result<impl warp::Reply, Infallible> {
    log::debug!("all_grocery_items");

    Ok(sqlx::query_as!(
        GroceryItem,
        r#"
                SELECT id, name, checked_off, position, created_at
                FROM items 
            "#,
    )
    .fetch_all(db.database())
    .await
    .map_or_else(
        |_| StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        |items| warp::reply::json(&items).into_response(),
    ))
}
pub async fn delete_grocery_item(id: i32, db: Db) -> Result<impl warp::Reply, Infallible> {
    log::debug!("delete_grocery_item with id: {}", id);

    let trans = match db.database().begin().await {
        Err(_) => return Ok(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(trans) => trans,
    };
    log::debug!("Started transaction...");

    let pos_of_delete = match sqlx::query!(r#"SELECT position FROM items WHERE id = $1"#, id)
        .fetch_one(db.database())
        .await
    {
        Err(_) => return Ok(StatusCode::NOT_FOUND),
        Ok(result) => result.position,
    };

    log::debug!("Position of delete: {}", pos_of_delete);
    if sqlx::query!(r#"DELETE FROM items WHERE id = $1"#, id,)
        .execute(db.database())
        .await
        .is_err()
    {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }

    log::debug!("Updating all positions > {}", pos_of_delete);
    if sqlx::query!(
        r#"UPDATE items SET position = position - 1 WHERE position > $1"#,
        pos_of_delete
    )
    .execute(db.database())
    .await
    .is_err()
    {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    };

    log::debug!("Comitting...");
    Ok(trans.commit().await.map_or_else(
        |_| StatusCode::INTERNAL_SERVER_ERROR,
        |_| StatusCode::NO_CONTENT,
    ))
}
