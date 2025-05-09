use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use futures::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime as BsonDateTime},
    Client, Collection,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use uuid::Uuid;

// Todo 항목을 위한 구조체
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Todo {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    #[serde(rename = "uuid")]
    uuid: String,
    title: String,
    completed: bool,
    created_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

// 새로운 Todo 생성을 위한 구조체
#[derive(Debug, Deserialize)]
struct CreateTodo {
    title: String,
}

// MongoDB 연결을 위한 상태
struct AppState {
    collection: Collection<Todo>,
}

// 새로운 Todo 생성
async fn create_todo(data: web::Json<CreateTodo>, state: web::Data<AppState>) -> impl Responder {
    let new_todo = Todo {
        id: None,
        uuid: Uuid::new_v4().to_string(),
        title: data.title.clone(),
        completed: false,
        created_at: Utc::now(),
        completed_at: None,
    };

    match state.collection.insert_one(&new_todo, None).await {
        Ok(result) => {
            let mut created_todo = new_todo;
            created_todo.id = result.inserted_id.as_object_id();
            HttpResponse::Created().json(created_todo)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Todo 목록 조회
async fn get_todos(state: web::Data<AppState>) -> impl Responder {
    match state.collection.find(None, None).await {
        Ok(mut cursor) => {
            let mut todos = Vec::new();
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(todo) => todos.push(todo),
                    Err(_) => return HttpResponse::InternalServerError().finish(),
                }
            }
            HttpResponse::Ok().json(todos)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Todo 완료 상태 변경
async fn toggle_todo(uuid: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let filter = doc! { "uuid": uuid.to_string() };

    match state.collection.find_one(filter.clone(), None).await {
        Ok(Some(mut todo)) => {
            todo.completed = !todo.completed;
            todo.completed_at = if todo.completed {
                Some(Utc::now())
            } else {
                None
            };

            let update = doc! {
                "$set": {
                    "completed": todo.completed,
                    "completed_at": todo.completed_at.map(|dt| BsonDateTime::from_millis(dt.timestamp_millis()))
                }
            };

            match state.collection.update_one(filter, update, None).await {
                Ok(_) => HttpResponse::Ok().json(todo),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // MongoDB 연결
    let mongodb_uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let database_name = env::var("DATABASE_NAME").expect("DATABASE_NAME must be set");
    let collection_name = env::var("COLLECTION_NAME").expect("COLLECTION_NAME must be set");

    let client = Client::with_uri_str(mongodb_uri)
        .await
        .expect("Failed to initialize MongoDB client");

    let database = client.database(&database_name);
    let collection = database.collection::<Todo>(&collection_name);

    let app_state = web::Data::new(AppState { collection });

    println!("서버가 http://127.0.0.1:8080 에서 실행 중입니다.");

    HttpServer::new(move || {
        let cors = Cors::permissive()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .expose_headers(["content-type"])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .route("/todos", web::post().to(create_todo))
            .route("/todos", web::get().to(get_todos))
            .route("/todos/{uuid}/toggle", web::post().to(toggle_todo))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
