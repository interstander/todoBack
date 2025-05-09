use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

// Todo 항목을 위한 구조체
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Todo {
    id: String,
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

// Todo 목록을 저장할 상태
struct AppState {
    todos: Mutex<Vec<Todo>>,
}

// 새로운 Todo 생성
async fn create_todo(data: web::Json<CreateTodo>, state: web::Data<AppState>) -> impl Responder {
    let new_todo = Todo {
        id: Uuid::new_v4().to_string(),
        title: data.title.clone(),
        completed: false,
        created_at: Utc::now(),
        completed_at: None,
    };

    let mut todos = state.todos.lock().unwrap();
    todos.push(new_todo.clone());

    HttpResponse::Created().json(new_todo)
}

// Todo 목록 조회
async fn get_todos(state: web::Data<AppState>) -> impl Responder {
    let todos = state.todos.lock().unwrap();
    HttpResponse::Ok().json(todos.clone())
}

// Todo 완료 상태 변경
async fn toggle_todo(id: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let mut todos = state.todos.lock().unwrap();

    if let Some(todo) = todos.iter_mut().find(|t| t.id == *id) {
        todo.completed = !todo.completed;
        todo.completed_at = if todo.completed {
            Some(Utc::now())
        } else {
            None
        };
        HttpResponse::Ok().json(todo.clone())
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        todos: Mutex::new(Vec::new()),
    });

    println!("서버가 http://127.0.0.1:8080 에서 실행 중입니다.");

    HttpServer::new(move || {
        // CORS 설정
        let cors = Cors::permissive() // 모든 CORS 요청을 허용
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .expose_headers(["content-type"])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors) // CORS 미들웨어 추가
            .app_data(app_state.clone())
            .route("/todos", web::post().to(create_todo))
            .route("/todos", web::get().to(get_todos))
            .route("/todos/{id}/toggle", web::post().to(toggle_todo))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
