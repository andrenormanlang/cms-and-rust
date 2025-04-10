use clap::Parser;
use common::CmsRustConfig;
use std::sync::Arc;

use axum::routing::{delete, get, post};
use axum::{
    debug_handler,
    extract::{self, Query},
    Extension, Json, Router,
};
use common::{
    AddPostRequest, AddPostResponse, AppError, Database, DeletePostResponse, GetPostResponse,
};
use http::StatusCode;
use serde::Deserialize;

use tokio::sync::RwLock;

// TODO : Rename this to something more useful
type DatabaseT = Arc<RwLock<Database>>;

#[derive(clap::Parser)]
struct ProgramArgs {
    // path to the config toml
    #[clap(long, short)]
    config_file: String,
}

// Define default pagination values
fn default_offset() -> i32 {
    0
}
fn default_limit() -> i32 {
    10
} // Or another sensible default

// Struct for pagination query parameters
#[derive(Deserialize)]
struct PaginationParams {
    #[serde(default = "default_offset")]
    offset: i32,
    #[serde(default = "default_limit")]
    limit: i32,
}

async fn try_main() -> anyhow::Result<()> {
    // Read the config
    let args = ProgramArgs::parse();
    let config = CmsRustConfig::new(&args.config_file)?;

    let database = Arc::new(RwLock::new(
        Database::new(&config.database_address, config.database_port).await?,
    ));

    // Axum for multiplexing the http connections to endpoints
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/posts", get(get_posts_handler))
        .route("/posts", post(add_post_handler))
        .route("/posts/:id", get(get_post_handler))
        .route("/posts/:id", delete(delete_post_handler))
        .layer(Extension(database));

    // run our app with hyper, listening globally on the configured port
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.admin_port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

#[tokio::main]
async fn main() {
    match try_main().await {
        Err(e) => {
            println!("exited program, error: {:?}", e);
        }
        _ => {}
    }
}

#[debug_handler]
async fn get_posts_handler(
    Extension(database_lock): Extension<DatabaseT>,
    Query(pagination): Query<PaginationParams>, // Extract pagination params
) -> Result<Json<Vec<GetPostResponse>>, AppError> {
    let database = database_lock.read().await;
    // Use pagination parameters when calling get_posts
    let posts = database
        .get_posts(pagination.offset, pagination.limit)
        .await?;
    Ok(Json(posts))
}

#[debug_handler]
async fn add_post_handler(
    Extension(database_lock): Extension<DatabaseT>,
    extract::Json(post_request): extract::Json<AddPostRequest>,
) -> Result<Json<AddPostResponse>, Json<AppError>> {
    // Check that everything is actually populated
    if post_request.title.is_empty() {
        // return some app error
        return Err(Json(AppError {
            err_msg: "cannot have empty post title".into(),
            status_code: StatusCode::BAD_REQUEST,
        }));
    }

    if post_request.excerpt.is_empty() {
        // return some app error
        return Err(Json(AppError {
            err_msg: "cannot have empty post excerpt".into(),
            status_code: StatusCode::BAD_REQUEST,
        }));
    }

    if post_request.content.is_empty() {
        // return some app error
        return Err(Json(AppError {
            err_msg: "cannot have empty post content".into(),
            status_code: StatusCode::BAD_REQUEST,
        }));
    }

    let database = database_lock.read().await;

    let post_id = match database
        .add_post(
            &post_request.title,
            &post_request.content,
            &post_request.excerpt,
        )
        .await
    {
        Ok(id) => id,
        Err(e) => {
            return Err(Json(AppError {
                err_msg: format!("could not store post in db: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            }))
        }
    };

    Ok(Json(AddPostResponse { post_id }))
}

#[debug_handler]
async fn get_post_handler(
    Extension(database_lock): Extension<DatabaseT>,
    extract::Path(post_id): extract::Path<i32>,
) -> Result<Json<GetPostResponse>, AppError> {
    let database = database_lock.read().await;
    let post = database.get_post(post_id).await?;

    Ok(Json(post))
}

#[debug_handler]
async fn delete_post_handler(
    Extension(database_lock): Extension<DatabaseT>,
    extract::Path(post_id): extract::Path<i32>,
) -> Result<Json<DeletePostResponse>, AppError> {
    let database = database_lock.read().await;
    let post = database.delete_post(post_id).await?;

    Ok(Json(post))
}
