use async_trait::async_trait;
use sqlx::postgres::{PgPoolOptions, PgConnectOptions, Postgres};
use sqlx::Pool;
use mockall::automock;
use std::error::Error;
use std::env;
use crate::models::{EmployeeData, EmployeeName, EmployeeSalary, UncheckedEmployeeSalary, SalaryMultiplier};

#[automock]
#[async_trait]
pub trait DBClient: Send + Sync{
    async fn init_db(&self) -> Result<(), Box<dyn Error>>; 
    async fn init_db_clear(&self) -> Result<(), Box<dyn Error>>; 
    async fn get_employee_salary(&self, data: EmployeeName) -> Result<EmployeeSalary, Box<dyn Error>>; 
    async fn add_new_employee(&self, data: EmployeeData) -> Result<(), Box<dyn Error>>;
    async fn increase_employee_salary(&self, data: SalaryMultiplier) -> Result<EmployeeSalary, Box<dyn Error>>;
}

/// Обертка над клиентом базы данных
///
/// Подключается к постгресу с помощью переменных окружения
#[derive(Debug)]
pub struct DBClientPostgres{
    inner_client: Pool<Postgres> 
}

impl DBClientPostgres{
    /// Новое подключение к базе данных
    ///
    /// Использовать для основного подключения
    pub async fn new() -> Result<DBClientPostgres, Box<dyn Error>> {
        let options = PgConnectOptions::new()
            .host(&env::var("DB_CONTAINER_NAME").unwrap_or("localhost".to_owned()))
            .username(&env::var("DB_USERNAME").unwrap_or("username".to_owned()))
            .password(&env::var("DB_PASSWORD").unwrap_or("password".to_owned()))
            .database(&env::var("DB_NAME").unwrap_or("username".to_owned()))
            .port(5432);
        let client = PgPoolOptions::new()
            .max_connections(7)
            .connect_with(options)
            .await?;

        Ok(DBClientPostgres{inner_client: client})
    }
    
    pub async fn new_test() -> Result<DBClientPostgres, Box<dyn Error>> {
        let options = PgConnectOptions::new()
            .host(&env::var("DB_CONTAINER_NAME").unwrap_or("localhost".to_owned()))
            .username(&env::var("DB_USERNAME").unwrap_or("username".to_owned()))
            .password(&env::var("DB_PASSWORD").unwrap_or("password".to_owned()))
            .database(&env::var("DB_NAME").unwrap_or("username".to_owned()))
            .port(5432);
        let client = PgPoolOptions::new()
            .max_connections(7)
            .connect_with(options)
            .await?;

        Ok(DBClientPostgres{inner_client: client})
    }
}

#[async_trait]
impl DBClient for DBClientPostgres{
    /// Инициализация схемы БД без стирания предыдущих данных
    async fn init_db(&self) -> Result<(), Box<dyn Error>> {
        sqlx::query(r#"CREATE TABLE IF NOT EXISTS employees (
                    id SERIAL PRIMARY KEY,
                    name VARCHAR(255) NOT NULL,
                    salary INT NOT NULL
                    )"#)
        .execute(&self.inner_client)
        .await?;
        Ok(())
    }
    
    /// Инициализация схемы БД с удалением существующих данных
    async fn init_db_clear(&self) -> Result<(), Box<dyn Error>> {
        let mut tx = self.inner_client.begin().await?;
        sqlx::query(r#"CREATE TABLE IF NOT EXISTS employees (id SERIAL PRIMARY KEY, name VARCHAR(255) NOT NULL, salary INT NOT NULL)"#)
        .execute(&mut *tx)
        .await?;
        sqlx::query("TRUNCATE TABLE employees")
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    /// Получить зарплату работника
    ///
    /// Обращается к базе и возвращает проверенные данные о зарплате сотрудника
    async fn get_employee_salary(&self, data: EmployeeName) -> Result<EmployeeSalary, Box<dyn Error>> {
        let employee_salary_raw: UncheckedEmployeeSalary = sqlx::query_as(r#"SELECT salary AS amount FROM employees WHERE name = $1"#)
            .bind(data.name)
            .fetch_one(&self.inner_client)
            .await?;
        Ok(employee_salary_raw.check()?)
    }
    
    /// Добавить нового сотрудника
    ///
    /// Обращается к базе и добавляет в нее новые данные о сотруднике
    async fn add_new_employee(&self, data: EmployeeData) -> Result<(), Box<dyn Error>> {
        sqlx::query(r#"INSERT INTO employees(name, salary) VALUES ($1 , $2)"#)
        .bind(data.name)
        .bind(data.salary)
        .execute(&self.inner_client)
        .await?;
        Ok(())
    }

    /// Увеличить зарплату сотрудника
    ///
    /// Обращается к базе и изменяет значение зарплаты сотрудника с совпадающим именем
    /// Возвращает предыдущее значение зарплаты
    async fn increase_employee_salary(&self, data: SalaryMultiplier) -> Result<EmployeeSalary, Box<dyn Error>> {
        let mut tx = self.inner_client.begin().await?;
        let employee_salary_raw: UncheckedEmployeeSalary = sqlx::query_as(r#"SELECT salary AS amount FROM employees WHERE name = $1"#)
            .bind(&data.name)
            .fetch_one(&mut *tx)
            .await?;
        let mut employee_salary = employee_salary_raw.check()?;

        let old_employee_salary = employee_salary.increase_by_percentage(&data)?;
        sqlx::query(r#"UPDATE employees SET salary = $1 WHERE name = $2"#)
            .bind(employee_salary.amount)
            .bind(&data.name)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(old_employee_salary)
    }
}

#[cfg(test)]
mod tests{
    use serial_test::serial;
    use super::*;

    fn set_env_vars(){
        dotenv::dotenv().ok();
    }

    #[actix_web::test]
    #[serial]
    async fn test_client_init_ok(){
        set_env_vars();
        let client = DBClientPostgres::new_test().await.unwrap();
        client.init_db_clear().await.unwrap();
    }

    #[actix_web::test]
    #[serial]
    async fn test_employee_addition(){
        set_env_vars();
        let client = DBClientPostgres::new_test().await.unwrap();
        client.init_db_clear().await.unwrap();
        client.add_new_employee(EmployeeData{name: "Test Employee".to_owned(), salary: 5000}).await.unwrap();
    }

    #[actix_web::test]
    #[serial]
    async fn test_employee_salary_getter(){
        set_env_vars();
        let client = DBClientPostgres::new_test().await.unwrap();
        client.init_db_clear().await.unwrap();
        client.add_new_employee(EmployeeData{name: "Test Employee".to_owned(), salary: 5000}).await.unwrap();
        let salary = client.get_employee_salary(EmployeeName { name: "Test Employee".to_owned() }).await.unwrap();
        assert_eq!(5000, salary.amount);
    }

    #[actix_web::test]
    #[serial]
    async fn test_employee_salary_increase(){
        set_env_vars();
        let client = DBClientPostgres::new_test().await.unwrap();
        client.init_db_clear().await.unwrap();
        client.add_new_employee(EmployeeData{name: "Test Employee".to_owned(), salary: 100}).await.unwrap();
        let old_salary = client.increase_employee_salary(SalaryMultiplier { name: "Test Employee".to_owned(), percentage: 25 }).await.unwrap();
        assert_eq!(100, old_salary.amount);
        let salary = client.get_employee_salary(EmployeeName { name: "Test Employee".to_owned() }).await.unwrap();
        assert_eq!(125, salary.amount);
    }
}
