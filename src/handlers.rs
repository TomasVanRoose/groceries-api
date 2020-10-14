use crate::database::Db;
use crate::models::{GroceryItem, NewGroceryItem};
use chrono::NaiveDate;
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::Reply;

pub async fn read_grocery_item(id: i32, db: Db) -> Result<impl warp::Reply, Infallible> {
    log::debug!("read_grocery_item for id: {:?}", id);

    Ok(sqlx::query_as!(
        GroceryItem,
        r#"
                SELECT id, name, checked_off, position, checked_off_at, created_at
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

    // delete all items that where ticked off yesterday or longer
    if let Err(_) = sqlx::query!(r#"DELETE FROM items where checked_off_at < 'yesterday'"#)
        .execute(db.database())
        .await
    {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }

    Ok(sqlx::query_as!(
        GroceryItem,
        r#"
                SELECT id, name, checked_off, position, checked_off_at, created_at
                FROM items 
                ORDER BY position
            "#,
    )
    .fetch_all(db.database())
    .await
    .map_or_else(
        |_| StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        |items| warp::reply::json(&items).into_response(),
    ))
}

pub async fn create_grocery_item(
    new_item: NewGroceryItem,
    db: Db,
) -> Result<impl warp::Reply, Infallible> {
    log::debug!("create_grocery_item: {:?}", new_item);

    Ok(sqlx::query_as!(
        GroceryItem,
        r#"INSERT INTO items (name, checked_off, position)
           VALUES($1, $2, $3)
           RETURNING id, name, checked_off, position, checked_off_at, created_at"#,
        new_item.name,
        new_item.checked_off,
        new_item.position as i32,
    )
    .fetch_one(db.database())
    .await
    .map_or_else(
        |_| StatusCode::BAD_REQUEST.into_response(),
        |item| {
            warp::reply::with_status(warp::reply::json(&item), StatusCode::CREATED).into_response()
        },
    ))
}

pub async fn update_grocery_item(
    id: i32,
    new_item: GroceryItem,
    db: Db,
) -> Result<impl warp::Reply, Infallible> {
    log::debug!("update_grocery_item: {:?}", new_item);

    let checked_off_at = if new_item.checked_off {
        Some(chrono::offset::Utc::now().naive_utc())
    } else {
        None
    };

    Ok(sqlx::query!(
        r#"UPDATE items SET (name, checked_off, position, checked_off_at, created_at) = ($1, $2, $3, $4, $5)
           WHERE id = $6"#,
        new_item.name,
        new_item.checked_off,
        new_item.position,
        checked_off_at,
        new_item.created_at,
        id,
    )
    .execute(db.database())
    .await
    .map_or_else(
        |_| StatusCode::INTERNAL_SERVER_ERROR,
        |_| StatusCode::NO_CONTENT,
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
