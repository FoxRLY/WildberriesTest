use actix_web::dev::ServiceResponse;
use actix_web::{get, put, post, App, HttpServer, Responder, HttpResponse, web};
use super::postgres_client::DBClient;
use super::models::{UncheckedEmployeeName, UncheckedSalaryMultiplier, UncheckedEmployeeData};
use std::error::Error;
use std::sync::Mutex;



// Эндпоинты приложения

#[get("/salary")]
async fn get_employee_salary(query: web::Query<UncheckedEmployeeName>) -> impl Responder {
    let employee_name = match query.into_inner().check(){
        Ok(name) => name,
        Err(e) => {return HttpResponse::BadRequest().body(format!("{e}"))}
    };
    HttpResponse::Ok().body("Bruh".to_string())
}

#[put("/add")]
async fn add_new_employee(query: web::Query<UncheckedEmployeeData>) -> impl Responder {
    let employee_data = match query.into_inner().check(){
        Ok(name) => name,
        Err(e) => {return HttpResponse::BadRequest().body(format!("{e}"))}
    };
    HttpResponse::Ok().body("Bruh".to_string())
}

#[post("/increase")]
async fn increase_employee_salary(query: web::Query<UncheckedSalaryMultiplier>) -> impl Responder {
    let salary_multiplier = match query.into_inner().check(){
        Ok(name) => name,
        Err(e) => {return HttpResponse::BadRequest().body(format!("{e}"))}
    };
    HttpResponse::Ok().body("Bruh".to_string())
}


// Сервер и его строитель

pub struct Server{
    host: String,
    port: u16,
    db_client: Mutex<DBClient>
}

impl Server{
    pub fn builder(db_client: Mutex<DBClient>) -> ServerBuilder {
        ServerBuilder {
            host: None,
            port: None,
            db_client
        }
    }
    
    pub async fn start(self) -> Result<(), Box<dyn Error>>{
        let postgres_client = web::Data::new(self.db_client);
        HttpServer::new(move || {
            App::new()
                .app_data(postgres_client.clone())
                .service(
                    web::scope("/employee")
                        .service(increase_employee_salary)
                        .service(get_employee_salary)
                        .service(add_new_employee)
                )
        })
        .bind((self.host, self.port))?
        .run()
        .await?;
        Ok(())
    }
    
    pub async fn test_start(self) -> Result<impl actix_service::Service<actix_http::Request, Response = ServiceResponse, Error = actix_web::Error>, Box<dyn Error>> {
        let postgres_client = web::Data::new(self.db_client);
        let app = actix_web::test::init_service(App::new()
            .app_data(postgres_client.clone())
            .service(
                web::scope("/employee")
                    .service(increase_employee_salary)
                    .service(get_employee_salary)
                    .service(add_new_employee)
                )
        ).await;
        Ok(app)
    }
}

struct ServerBuilder{
    host: Option<String>,
    port: Option<u16>,
    db_client: Mutex<DBClient>
}

impl ServerBuilder{
    pub fn host(mut self, value: String) -> Self {
        self.host = Some(value);
        self
    }

    pub fn port(mut self, value: u16) -> Self {
        self.port = Some(value);
        self
    }

    pub fn build(self) -> Server {
        Server{
            host: self.host.unwrap_or("localhost".to_string()),
            port: self.port.unwrap_or(8080),
            db_client: self.db_client
        }
    }
}


#[cfg(test)]
mod tests {
    use serde_json::json;
    use serial_test::serial;
    use actix_service::Service;
    use actix_web::http::StatusCode;
    use urlencoding::encode;
    use super::*;

    fn set_env_vars(){
        dotenv::dotenv().ok();
    }

    #[actix_web::test]
    #[serial]
    async fn test_employee_addition() {
        let db_client = Mutex::new(DBClient::new().await.unwrap());
        let app = Server::builder(db_client.clone())
            .host("localhost".to_owned())
            .port(8080)
            .build()
            .test_start()
            .await
            .unwrap();
        let url = encode("/employee/add?name=Иван Сергеевич Фрунзенко&salary=2000");
        let request = actix_web::test::TestRequest::put()
            .uri(&url)
            .to_request();
        let response = app.call(request).await.unwrap();
        let response_status = response.status();
        // let response_body = actix_web::test::read_body(response).await;
        // let response_body = String::from_utf8(response_body.to_vec()).unwrap();
        // let awaited_body = String::from("Иван Сергеевич Фрунзенко");
        assert_eq!(response_status, StatusCode::OK);
        // assert_eq!(response_body, awaited_body);
    }

    #[actix_web::test]
    #[serial]
    async fn test_employee_getter() {
        let db_client = Mutex::new(DBClient::new().await.unwrap());
        let app = Server::builder(db_client.clone())
            .host("localhost".to_owned())
            .port(8080)
            .build()
            .test_start()
            .await
            .unwrap();
        let url = encode("/employee/get?name=Иван Сергеевич Фрунзенко");
        let request = actix_web::test::TestRequest::get()
            .uri(&url)
            .to_request();
        let response = app.call(request).await.unwrap();
        let response_status = response.status();
        assert_eq!(response_status, StatusCode::OK);
    }
}
