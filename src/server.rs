use actix_web::dev::ServiceResponse;
use actix_web::{get, put, post, App, HttpServer, Responder, HttpResponse, web};
use super::postgres_client::DBClient;
use super::models::{UncheckedEmployeeName, UncheckedSalaryMultiplier, UncheckedEmployeeData};
use std::error::Error;
use std::sync::Mutex;
use log::{info, error};
use simplelog::{CombinedLogger, Config, LevelFilter, WriteLogger};
use std::fs::File;


// Эндпоинты приложения

/// Получить зарплату работника по имени
///
/// Пример: /salary?name="Василий Петрович"
#[get("/salary")]
async fn get_employee_salary(query: web::Query<UncheckedEmployeeName>, db_client: web::Data<Mutex<DBClient>>) -> impl Responder {
    let db_client = db_client.lock().unwrap();
    let employee_name = match query.into_inner().check(){
        Ok(name) => name,
        Err(e) => {
            error!("Bad request: {e}");
            return HttpResponse::BadRequest().body(format!("{e}"))
        }
    };
    match db_client.get_employee_salary(employee_name.clone()).await {
        Ok(salary) => {
            info!("Sent salary of employee with name {:?}", employee_name);
            HttpResponse::Ok().body(format!("{}", salary.amount))
        },
        Err(e) => {
            error!("Internal error: {e}");
            HttpResponse::BadRequest().body(format!("{e}"))
        }
    }
}


/// Добавить нового работника
///
/// Пример: /add?name="Василий Петрович"&salary=8000
#[put("/add")]
async fn add_new_employee(query: web::Query<UncheckedEmployeeData>, db_client: web::Data<Mutex<DBClient>>) -> impl Responder {
    let db_client = db_client.lock().unwrap();
    let employee_data = match query.into_inner().check(){
        Ok(data) => data,
        Err(e) => {
            error!("Bad Request: {e}");
            return HttpResponse::BadRequest().body(format!("{e}"))
        }
    };
    match db_client.add_new_employee(employee_data.clone()).await {
        Ok(_) => {
            info!{"Added new employee: {:?}", employee_data};
            HttpResponse::Ok().body("Successfully added new employee".to_string())
        },
        Err(e) => {
            error!("Internal error: {e}");
            HttpResponse::BadRequest().body(format!("{e}"))
        }
    }
}


/// Увеличить зарплату сотруднику
///
/// Пример: /increase?name="Василий Петрович"&percentage=20
#[post("/increase")]
async fn increase_employee_salary(query: web::Query<UncheckedSalaryMultiplier>, db_client: web::Data<Mutex<DBClient>>) -> impl Responder {
    let db_client = db_client.lock().unwrap();
    let salary_multiplier = match query.into_inner().check(){
        Ok(multiplier) => multiplier,
        Err(e) => {
            error!{"Bad Request: {e}"};
            return HttpResponse::BadRequest().body(format!("{e}"))
        }
    };
    match db_client.increase_employee_salary(salary_multiplier.clone()).await {
        Ok(old_salary) => {
            info!("Increased the salary with data {:?}", salary_multiplier);
            HttpResponse::Ok().body(format!("{}", old_salary.amount))
        },
        Err(e) => {
            error!("Internal error: {e}");
            HttpResponse::BadRequest().body(format!("{e}"))
        }
    }
}


// Сервер и его строитель

pub struct Server{
    host: String,
    port: u16,
}

impl Server{
    pub fn builder() -> ServerBuilder {
        ServerBuilder {
            host: None,
            port: None,
        }
    }
    
    pub async fn start(self) -> Result<(), Box<dyn Error>>{
        let log_file = File::create("./app.log").unwrap();
        CombinedLogger::init(
            vec![
                WriteLogger::new(
                    LevelFilter::Debug,
                    Config::default(),
                    log_file,
                ),
                WriteLogger::new(
                    LevelFilter::Debug,
                    Config::default(),
                    std::io::stdout()
                )
            ]
        ).unwrap();

        let postgres_client = DBClient::new().await?;
        postgres_client.init_db().await?;
        let postgres_client = web::Data::new(Mutex::new(postgres_client));
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
        let postgres_client = DBClient::new_test().await?;
        postgres_client.init_db_clear().await?;
        let postgres_client = web::Data::new(Mutex::new(postgres_client));
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

pub struct ServerBuilder{
    host: Option<String>,
    port: Option<u16>,
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
        }
    }
}


#[cfg(test)]
mod tests {
    use serial_test::serial;
    use actix_service::Service;
    use actix_web::http::StatusCode;
    use crate::models::{EmployeeName, EmployeeData};

    use super::*;

    fn set_env_vars(){
        dotenv::dotenv().ok();
    }

    #[actix_web::test]
    #[serial]
    async fn test_employee_addition() {
        set_env_vars();
        let db_client = DBClient::new_test().await.unwrap();
        let app = Server::builder()
            .host("localhost".to_owned())
            .port(8080)
            .build()
            .test_start()
            .await
            .unwrap();
        let request = actix_web::test::TestRequest::put()
            .uri("/employee/add?name=Test%20Employee&salary=2000")
            .to_request();
        let response = app.call(request).await.unwrap();
        let response_status = response.status();
        assert_eq!(response_status, StatusCode::OK);
        let salary = db_client.get_employee_salary(EmployeeName{name: "Test Employee".to_owned()}).await.unwrap();
        assert_eq!(2000, salary.amount);
    }

    #[actix_web::test]
    #[serial]
    async fn test_employee_addition_failing_1() {
        set_env_vars();
        let app = Server::builder()
            .host("localhost".to_owned())
            .port(8080)
            .build()
            .test_start()
            .await
            .unwrap();
        let request = actix_web::test::TestRequest::put()
            .uri("/employee/add?name=Test%20Employee&salary=0")
            .to_request();
        let response = app.call(request).await.unwrap();
        let response_status = response.status();
        assert_eq!(response_status, StatusCode::BAD_REQUEST);
    }


    #[actix_web::test]
    #[serial]
    async fn test_employee_addition_failing_2() {
        set_env_vars();
        let app = Server::builder()
            .host("localhost".to_owned())
            .port(8080)
            .build()
            .test_start()
            .await
            .unwrap();
        let request = actix_web::test::TestRequest::put()
            .uri("/employee/add?name=%20&salary=2000")
            .to_request();
        let response = app.call(request).await.unwrap();
        let response_status = response.status();
        assert_eq!(response_status, StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    #[serial]
    async fn test_employee_addition_failing_3() {
        set_env_vars();
        let app = Server::builder()
            .host("localhost".to_owned())
            .port(8080)
            .build()
            .test_start()
            .await
            .unwrap();
        let request = actix_web::test::TestRequest::put()
            .uri("/employee/add?name=%20&salary=-35")
            .to_request();
        let response = app.call(request).await.unwrap();
        let response_status = response.status();
        assert_eq!(response_status, StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    #[serial]
    async fn test_employee_salary_getter() {
        set_env_vars();
        let db_client = DBClient::new_test().await.unwrap();
        let app = Server::builder()
            .host("localhost".to_owned())
            .port(8080)
            .build()
            .test_start()
            .await
            .unwrap();
        db_client.add_new_employee(EmployeeData{name: "Test Employee".to_owned(), salary: 100}).await.unwrap();
        let request = actix_web::test::TestRequest::get()
            .uri("/employee/salary?name=Test%20Employee")
            .to_request();
        let response = app.call(request).await.unwrap();
        let response_status = response.status();
        assert_eq!(response_status, StatusCode::OK);
        let response_body = actix_web::test::read_body(response).await;
        let response_body = String::from_utf8(response_body.to_vec()).unwrap();
        let response_body: i32 = response_body.trim().parse().unwrap();
        assert_eq!(100, response_body);
    }


    #[actix_web::test]
    #[serial]
    async fn test_employee_salary_getter_failing() {
        set_env_vars();
        let db_client = DBClient::new_test().await.unwrap();
        let app = Server::builder()
            .host("localhost".to_owned())
            .port(8080)
            .build()
            .test_start()
            .await
            .unwrap();
        db_client.add_new_employee(EmployeeData{name: "Test Employee".to_owned(), salary: 100}).await.unwrap();
        let request = actix_web::test::TestRequest::get()
            .uri("/employee/salary?name=Tost%20Empliyep")
            .to_request();
        let response = app.call(request).await.unwrap();
        let response_status = response.status();
        assert_eq!(response_status, StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    #[serial]
    async fn test_employee_increase_salary(){
        set_env_vars();
        let db_client = DBClient::new_test().await.unwrap();
        let app = Server::builder()
            .host("localhost".to_owned())
            .port(8080)
            .build()
            .test_start()
            .await
            .unwrap();
        db_client.add_new_employee(EmployeeData{name: "Test Employee".to_owned(), salary: 100}).await.unwrap();
        let request = actix_web::test::TestRequest::post()
            .uri("/employee/increase?name=Test%20Employee&percentage=25")
            .to_request();
        let response = app.call(request).await.unwrap();
        let response_status = response.status();
        assert_eq!(response_status, StatusCode::OK);
        let response_body = actix_web::test::read_body(response).await;
        let response_body = String::from_utf8(response_body.to_vec()).unwrap();
        let response_body: i32 = response_body.trim().parse().unwrap();
        assert_eq!(100, response_body);
        let new_salary = db_client.get_employee_salary(EmployeeName { name: "Test Employee".to_owned() }).await.unwrap();
        assert_eq!(125, new_salary.amount);
    }

    #[actix_web::test]
    #[serial]
    async fn test_employee_increase_salary_failing_1(){
        set_env_vars();
        let db_client = DBClient::new_test().await.unwrap();
        let app = Server::builder()
            .host("localhost".to_owned())
            .port(8080)
            .build()
            .test_start()
            .await
            .unwrap();
        db_client.add_new_employee(EmployeeData{name: "Test Employee".to_owned(), salary: 100}).await.unwrap();
        let request = actix_web::test::TestRequest::post()
            .uri("/employee/increase?name=Tust%20Mmployee&percentage=25")
            .to_request();
        let response = app.call(request).await.unwrap();
        let response_status = response.status();
        assert_eq!(response_status, StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    #[serial]
    async fn test_employee_increase_salary_failing_2(){
        set_env_vars();
        let db_client = DBClient::new_test().await.unwrap();
        let app = Server::builder()
            .host("localhost".to_owned())
            .port(8080)
            .build()
            .test_start()
            .await
            .unwrap();
        db_client.add_new_employee(EmployeeData{name: "Test Employee".to_owned(), salary: 100}).await.unwrap();
        let request = actix_web::test::TestRequest::post()
            .uri("/employee/increase?name=Test%20Employee&percentage=0")
            .to_request();
        let response = app.call(request).await.unwrap();
        let response_status = response.status();
        assert_eq!(response_status, StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    #[serial]
    async fn test_employee_increase_salary_failing_3(){
        set_env_vars();
        let db_client = DBClient::new_test().await.unwrap();
        let app = Server::builder()
            .host("localhost".to_owned())
            .port(8080)
            .build()
            .test_start()
            .await
            .unwrap();
        db_client.add_new_employee(EmployeeData{name: "Test Employee".to_owned(), salary: 100}).await.unwrap();
        let request = actix_web::test::TestRequest::post()
            .uri("/employee/increase?name=Tust%20Mmployee&percentage=-31")
            .to_request();
        let response = app.call(request).await.unwrap();
        let response_status = response.status();
        assert_eq!(response_status, StatusCode::BAD_REQUEST);
    }
}
