use clap::Parser;
use common::CmsGoConfig;
use std::{
    fs::File,
    io::Read,
    path::Path,
    sync::Arc,
};

use axum::{
    debug_handler, extract::Query, http::StatusCode, response::Html, routing::get, Extension, Json,
    Router,
};
use common::{markdown_filter, AppError, Database};
use minijinja::{context, Environment};
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

fn read_file<T: AsRef<Path>>(path: T) -> anyhow::Result<String> {
    let mut file = File::open(path)?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    Ok(file_contents)
}

async fn try_main() -> anyhow::Result<()> {
    // Read the config
    let args = ProgramArgs::parse();
    let config = Arc::new(CmsGoConfig::new(&args.config_file)?);

    // TODO : we probably don't want to print secrets ;)
    println!("config");
    println!("{:#?}", config);

    let database = Arc::new(RwLock::new(
        Database::new(&config.database_address, config.database_port).await?,
    ));

    // Get the current directory
    let current_dir = std::env::current_dir()?;
    println!("Current directory: {:?}", current_dir);

    // Build the path to the template file
    let template_path = current_dir.join("views").join("index.html.in");
    println!("Template path: {:?}", template_path);

    // Verify the template file exists
    if !template_path.exists() {
        return Err(anyhow::anyhow!(
            "Template file not found at: {:?}",
            template_path
        ));
    }

    // Axum for multiplexing the http connections to endpoints
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(home_handler))
        .layer(Extension(database))
        .layer(Extension(config.clone()));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.webserver_port))
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

fn default_page_num() -> i32 {
    0
}

#[derive(Deserialize)]
struct HomeHandlerParams {
    #[serde(default = "default_page_num")]
    page_num: i32,
}

#[debug_handler]
async fn home_handler(
    Extension(database_lock): Extension<DatabaseT>,
    Extension(config): Extension<Arc<CmsGoConfig>>,
    Query(home_params): Query<HomeHandlerParams>,
) -> Result<Html<String>, Json<AppError>> {
    let database = database_lock.read().await;
    let posts = database.get_posts(home_params.page_num, 10).await?;

    let current_dir = std::env::current_dir().map_err(|e| AppError {
        err_msg: e.to_string(),
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    let template_path = current_dir.join("views").join("index.html.in");

    let html = read_file(template_path).map_err(|e| AppError {
        err_msg: format!("Failed to read template file: {}", e),
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    let mut env = Environment::new();
    env.add_filter("markdown", markdown_filter);

    env.add_template("index", &html).map_err(|_| AppError {
        err_msg: "could not parse template".into(),
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    let tmpl = env.get_template("index").map_err(|_| AppError {
        err_msg: "could not get template".into(),
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    let template = tmpl
        .render(context! {
            posts => posts,
            navbar => config.navbar
        })
        .map_err(|_| AppError {
            err_msg: "could not render template".into(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    Ok(Html(template))
}
