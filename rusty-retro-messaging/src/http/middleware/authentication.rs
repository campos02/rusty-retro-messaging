use axum::Json;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::IntoResponse;
use chrono::Utc;
use sqlx::{MySql, Pool};

pub async fn authentication(
    State(pool): State<Pool<MySql>>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let headers = request.headers();
    if let Some(authorization) = headers.get(AUTHORIZATION) {
        let token = authorization
            .to_str()
            .or(Err((
                StatusCode::UNAUTHORIZED,
                Json(String::from("User not logged in")),
            )))?
            .replace("Bearer ", "");

        let token = sqlx::query!(
            "SELECT token, valid_until, user_id FROM tokens WHERE token = ? LIMIT 1",
            token
        )
        .fetch_one(&pool)
        .await
        .or(Err((
            StatusCode::UNAUTHORIZED,
            Json(String::from("User not logged in")),
        )))?;

        if Utc::now().naive_utc() <= token.valid_until {
            Ok(next.run(request).await)
        } else {
            Err((
                StatusCode::UNAUTHORIZED,
                Json(String::from("User not logged in")),
            ))
        }
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(String::from("User not logged in")),
        ))
    }
}
