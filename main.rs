use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_web::middleware::Logger;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use std::sync::Arc;
use log::info;
use lazy_static::lazy_static;

// Define a struct to represent a book
#[derive(Serialize, Deserialize, Clone)]
struct Book {
    id: i32,
    title: String,
    author: String,
}

// Define a struct to represent a new book
#[derive(Serialize, Deserialize)]
struct NewBook {
    title: String,
    author: String,
}

// In-memory storage for books
type Books = Arc<RwLock<Vec<Book>>>;

lazy_static! {
    static ref BOOKS: Books = Arc::new(RwLock::new(vec![]));
}


// Endpoint to get all books
async fn get_books(books: web::Data<Books>) -> impl Responder {
            info!("get all books");
    let books = books.read().await;
    let books = books.clone();
    HttpResponse::Ok().json(books)
}

// Endpoint to get a book by id
async fn get_book(id: web::Path<i32>, books: web::Data<Books>) -> impl Responder {
        info!("get book");
    let books = books.read().await;
    let book = books.iter().find(|b| b.id == *id);
    match book {
        Some(book) => HttpResponse::Ok().json(book),
        None => HttpResponse::NotFound().body("Book not found"),
    }
}

// Endpoint to create a new book
async fn create_book(new_book: web::Json<NewBook>, books: web::Data<Books>) -> impl Responder {
    info!("create book");
    let mut books = books.write().await;
    let id = books.len() as i32 + 1;
    let book = Book {
        id,
        title: new_book.title.clone(),
        author: new_book.author.clone(),
    };
    books.push(book.clone());
    HttpResponse::Created().json(book)
}

// Endpoint to update a book
async fn update_book(id: web::Path<i32>, new_book: web::Json<NewBook>, books: web::Data<Books>) -> impl Responder {
    info!("update book");
    let mut books = books.write().await;
    let book = books.iter_mut().find(|b| b.id == *id);
    match book {
        Some(book) => {
            book.title = new_book.title.clone();
            book.author = new_book.author.clone();
            HttpResponse::Ok().json(book)
        }
        None => HttpResponse::NotFound().body("Book not found"),
    }
}

// Endpoint to delete a book
async fn delete_book(id: web::Path<i32>, books: web::Data<Books>) -> impl Responder {
    info!("delete books");
    let mut books = books.write().await;
    let index = books.iter().position(|b| b.id == *id);
    match index {
        Some(index) => {
            books.remove(index);
            HttpResponse::Ok().body("Book deleted")
        }
        None => HttpResponse::NotFound().body("Book not found"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("Server started on port 8080");
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(BOOKS.clone()))
            .service(web::resource("/books").route(web::get().to(get_books)))
            .service(web::resource("/books/{id}").route(web::get().to(get_book)))
            .service(web::resource("/books").route(web::post().to(create_book)))
            .service(web::resource("/books/{id}").route(web::put().to(update_book)))
            .service(web::resource("/books/{id}").route(web::delete().to(delete_book)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}




#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[actix_web::test]
    async fn test_get_books() {
        let app = test::init_service(App::new().app_data(web::Data::new(BOOKS.clone())).service(web::resource("/books").route(web::get().to(get_books)))).await;
        let req = test::TestRequest::get().uri("/books").to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), 200);
    }

    #[actix_web::test]
    async fn test_create_book() {
        let app = test::init_service(App::new().app_data(web::Data::new(BOOKS.clone())).service(web::resource("/books").route(web::post().to(create_book)))).await;
        let req = test::TestRequest::post()
            .uri("/books")
            .set_json(&NewBook {
                title: "Book Title".to_string(),
                author: "Book Author".to_string(),
            })
            .to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), 201);
    }

    #[actix_web::test]
    async fn test_get_book() {
        let app = test::init_service(App::new().app_data(web::Data::new(BOOKS.clone()))
        .service(web::resource("/books").route(web::post().to(create_book)))
        .service(web::resource("/books/{id}").route(web::get().to(get_book)))).await;
        
        let req = test::TestRequest::post()
            .uri("/books")
            .set_json(&NewBook {
                title: "Book Title".to_string(),
                author: "Book Author".to_string(),
            })
            .to_request();

        let _res = test::call_service(&app, req).await;

        let req = test::TestRequest::get().uri("/books/1").to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), 200);
    }

    #[actix_web::test]
    async fn test_update_book() {
        let app = test::init_service(App::new().app_data(web::Data::new(BOOKS.clone())).service(web::resource("/books/{id}").route(web::put().to(update_book)))).await;
        let req = test::TestRequest::put()
            .uri("/books/1")
            .set_json(&NewBook {
                title: "Updated Book Title".to_string(),
                author: "Updated Book Author".to_string(),
            })
            .to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), 200);
    }

    #[actix_web::test]
    async fn test_delete_book() {
        let app = test::init_service(App::new().app_data(web::Data::new(BOOKS.clone())).service(web::resource("/books/{id}").route(web::delete().to(delete_book)))).await;

        let req = test::TestRequest::post()
            .uri("/books")
            .set_json(&NewBook {
                title: "Book Title".to_string(),
                author: "Book Author".to_string(),
            })
            .to_request();
        let _res = test::call_service(&app, req).await;

        let req = test::TestRequest::delete().uri("/books/1").to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), 200);
    }
}
