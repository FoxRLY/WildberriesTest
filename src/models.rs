use std::error::Error;
use std::fmt::Display;
use sqlx::FromRow;


// Полезные инструменты

#[derive(Debug)]
struct CustomError<'a>{
    msg: &'a str
}

impl<'a> Display for CustomError<'a>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg) 
    }
}

impl<'a> Error for CustomError<'a>{
}

fn check_name(name: &str) -> Result<(), Box<dyn Error>> {
    if name.trim().is_empty(){
        Err(CustomError{msg: "employee name cannot consist of whitespaces or have zero length"})?
    }
    Ok(())
}

fn check_salary(salary: i32) -> Result<(), Box<dyn Error>> {
    if salary <= 0 {
        Err(CustomError{msg: "employee's salary cannot be less than or equal to zero"})?;
    }
    Ok(())
}

fn check_percentage(percentage: i32) -> Result<(), Box<dyn Error>> {
    if percentage <= 0 {
        Err(CustomError{msg: "salary percentage increase cannot be less than or equal to zero"})?;
    }
    Ok(())
}


// Модели данных


/// Модель непроверенного имени сотрудника
///
/// Имя сотрудника, приходящее с эндпоинта и подлежащее проверке
#[derive(Debug, serde::Serialize, serde::Deserialize, FromRow, Clone)]
pub struct UncheckedEmployeeName{
    name: String,
}

impl UncheckedEmployeeName{
    /// Sanity-check для входящего имени
    ///
    /// Преобразует непроверенные данные в проверенные, поглощая объект.
    /// Проверка - имя не должно состоять из пробелов или иметь нулевую длину
    pub fn check(self) -> Result<EmployeeName, Box<dyn Error>> {
        check_name(&self.name)?;
        Ok(EmployeeName{name:self.name})
    }
}


/// Модель имени сотрудника
///
/// Проверенное значение имени сотрудника
#[derive(Debug, serde::Serialize, serde::Deserialize, FromRow, Clone)]
pub struct EmployeeName{
    pub name: String
}



/// Модель Непроверенной зарплаты сотрудника
///
/// Значение зарплаты сотрудника, приходящее с эндпоинта и подлежащее проверке
#[derive(Debug, serde::Serialize, serde::Deserialize, FromRow, Clone)]
pub struct UncheckedEmployeeSalary{
    amount: i32,
}

impl UncheckedEmployeeSalary{
    /// Sanity-check для значения зарплаты сотрудника
    ///
    /// Преобразует непроверенные данные в проверенные, поглощая объект
    /// Проверка - зарплата не может быть меньше либо равной нулю
    pub fn check(self) -> Result<EmployeeSalary, Box<dyn Error>> {
        check_salary(self.amount)?;
        Ok(EmployeeSalary{amount:self.amount})
    }
}


/// Модель Зарплаты сотрудника
///
/// Проверенное значение зарплаты сотрудника
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, FromRow)]
pub struct EmployeeSalary{
    pub amount: i32,
}

impl EmployeeSalary{
    /// Увеличить зарплату на процент
    ///
    /// Увеличивает зарплату на определенный процент с необходимыми проверками и делает возвращает
    /// старое значение
    pub fn increase_by_percentage(&mut self, percent: &SalaryMultiplier) -> Result<EmployeeSalary, Box<dyn Error>> {
        let old_salary = self.clone();
        let mut addition = old_salary.amount;
        addition = addition.checked_mul(percent.percentage)
            .ok_or(CustomError{msg:"employee's salary is too high to perform math operations"})?;
        addition = addition.checked_add(100)
            .ok_or(CustomError{msg:"employee's salary is too high to perform math operations"})?;
        addition = addition.checked_sub(1)
            .ok_or(CustomError{msg:"employee's salary is too low to perform math operations"})?;
        addition = addition.checked_div(100)
            .ok_or(CustomError{msg:"employee's salary is too low to perform math operations"})?;

        self.amount = self.amount.checked_add(addition)
            .ok_or(CustomError{msg:"employee's salary is too high to perform math operations"})?;
        check_salary(self.amount)?;
        Ok(old_salary)
    }
}



/// Модель Непроверенных данных о работнике
///
/// Данные, приходящие с эндпоинта и посдлежащие проверке
#[derive(Debug, serde::Serialize, serde::Deserialize, FromRow, Clone)]
pub struct UncheckedEmployeeData{
    name: String,
    salary: i32,
}

impl UncheckedEmployeeData{
    pub fn check(self) -> Result<EmployeeData, Box<dyn Error>> {
        check_name(&self.name)?;
        check_salary(self.salary)?;
        Ok(EmployeeData { name: self.name, salary: self.salary })
    }
}


/// Модель данных о работнике
///
/// Проверенное значение данных о работнике
#[derive(Debug, serde::Serialize, serde::Deserialize, FromRow, Clone)]
pub struct EmployeeData{
    pub name: String,
    pub salary: i32,
}


/// Модель Непроверенного процента повышения зарплаты
///
/// Процент повышения зарплаты, приходящий с эндпоинта и посдлежащий проверке
#[derive(Debug, serde::Serialize, serde::Deserialize, FromRow, Clone)]
pub struct UncheckedSalaryMultiplier{
    name: String,
    percentage: i32,
}

impl UncheckedSalaryMultiplier{
    /// Sanity-check для значения процента от зарплаты
    ///
    /// Преобразует непроверенные данные в проверенные, поглощая объект
    /// Проверка - процент не может быть равен нулю
    pub fn check(self) -> Result<SalaryMultiplier, Box<dyn Error>> {
        check_name(&self.name)?;
        check_percentage(self.percentage)?;
        Ok(SalaryMultiplier{percentage: self.percentage, name: self.name})
    }
}



/// Модель процента повышения зарплаты сотрудника
///
/// Проверенное значение зарплаты сотрудника
#[derive(Debug, serde::Serialize, serde::Deserialize, FromRow, Clone)]
pub struct SalaryMultiplier{
    pub name: String,
    pub percentage: i32,
}

impl SalaryMultiplier{
    pub fn get_name(&self) -> EmployeeName {
        EmployeeName{name: self.name.to_owned()}
    } 
}

#[cfg(test)]
mod tests{
    use super::{UncheckedEmployeeName, UncheckedEmployeeData, UncheckedEmployeeSalary, SalaryMultiplier, UncheckedSalaryMultiplier};

    #[test]
    fn employee_name_test(){
        let employee = UncheckedEmployeeName{name: "Владимир Евгеньевич Масленников".to_owned()};
        let employee = employee.check().unwrap();
        assert_eq!("Владимир Евгеньевич Масленников".to_owned(), employee.name);
        let employee = UncheckedEmployeeName{name: "Abdula Ibn Nurahmat".to_owned()};
        let employee = employee.check().unwrap();
        assert_eq!("Abdula Ibn Nurahmat".to_owned(), employee.name);
    }

    #[test]
    fn employee_name_test_failing(){
        if let Ok(val) = (UncheckedEmployeeName{name: "".to_owned()}).check(){
            panic!("Bad name somehow passed the check: {}", val.name)
        }
        if let Ok(val) = (UncheckedEmployeeName{name: "       ".to_owned()}).check(){
            panic!("Bad name somehow passed the check: {}", val.name)
        }
    }

    #[test]
    fn employee_data_test(){
        let employee = UncheckedEmployeeData{name:"Владимир Евгеньевич Масленников".to_owned(), salary: 5000};
        let employee = employee.check().unwrap();
        assert_eq!("Владимир Евгеньевич Масленников".to_owned(), employee.name);
        assert_eq!(5000, employee.salary);
    }

    #[test]
    fn employee_data_test_failing(){
        if let Ok(data) = (UncheckedEmployeeData{name:"Владимир Евгеньевич Масленников".to_owned(), salary: 0}).check() {
            panic!("Bad data somehow passed the check: {:?}", data);
        }
        if let Ok(data) = (UncheckedEmployeeData{name:"Владимир Евгеньевич Масленников".to_owned(), salary: -320}).check() {
            panic!("Bad data somehow passed the check: {:?}", data);
        }
        if let Ok(data) = (UncheckedEmployeeData{name:"".to_owned(), salary: 5000}).check() {
            panic!("Bad data somehow passed the check: {:?}", data);
        }
        if let Ok(data) = (UncheckedEmployeeData{name:"  ".to_owned(), salary: 5000}).check() {
            panic!("Bad data somehow passed the check: {:?}", data);
        }
        if let Ok(data) = (UncheckedEmployeeData{name:"         ".to_owned(), salary: 5000}).check() {
            panic!("Bad data somehow passed the check: {:?}", data);
        }
        if let Ok(data) = (UncheckedEmployeeData{name:"".to_owned(), salary: 0}).check() {
            panic!("Bad data somehow passed the check: {:?}", data);
        }
        if let Ok(data) = (UncheckedEmployeeData{name:"".to_owned(), salary: -250}).check() {
            panic!("Bad data somehow passed the check: {:?}", data);
        }
    }

    #[test]
    fn employee_salary_check_test(){
        let salary = UncheckedEmployeeSalary{amount: 500}.check().unwrap();
        assert_eq!(500, salary.amount);
    }

    #[test]
    fn employee_salary_increase_test(){
        let mut salary = UncheckedEmployeeSalary{amount: 100}.check().unwrap();
        let old_salary = salary.increase_by_percentage(&SalaryMultiplier{name: "Test Employee".to_owned(), percentage: 25}).unwrap();
        assert_eq!(100, old_salary.amount);
        assert_eq!(125, salary.amount);

        let mut salary = UncheckedEmployeeSalary{amount: 1000}.check().unwrap();
        let old_salary = salary.increase_by_percentage(&SalaryMultiplier{name: "Test Employee".to_owned(), percentage: 25}).unwrap();
        assert_eq!(1000, old_salary.amount);
        assert_eq!(1250, salary.amount);

        let mut salary = UncheckedEmployeeSalary{amount: 1000}.check().unwrap();
        let old_salary = salary.increase_by_percentage(&SalaryMultiplier{name: "Test Employee".to_owned(), percentage: 50}).unwrap();
        assert_eq!(1000, old_salary.amount);
        assert_eq!(1500, salary.amount);

        let mut salary = UncheckedEmployeeSalary{amount: 1000}.check().unwrap();
        let old_salary = salary.increase_by_percentage(&SalaryMultiplier{name: "Test Employee".to_owned(), percentage: 100}).unwrap();
        assert_eq!(1000, old_salary.amount);
        assert_eq!(2000, salary.amount);

        let mut salary = UncheckedEmployeeSalary{amount: 1000}.check().unwrap();
        let old_salary = salary.increase_by_percentage(&SalaryMultiplier{name: "Test Employee".to_owned(), percentage: 200}).unwrap();
        assert_eq!(1000, old_salary.amount);
        assert_eq!(3000, salary.amount);
    }

    #[test]
    fn employee_salary_increase_failing(){
        let mut salary = UncheckedEmployeeSalary{amount: 2147483647}.check().unwrap();
        if let Ok(val) = salary.increase_by_percentage(&SalaryMultiplier{name: "Test Employee".to_owned(), percentage: 100}){
            panic!("Impossible increase in salary was performed on value: {}", val.amount);
        }
    }

    #[test]
    fn salary_multipier_check_test(){
        let salary = UncheckedSalaryMultiplier{percentage: 100, name: "Test Employee".to_owned()}.check().unwrap();
        assert_eq!("Test Employee".to_owned(), salary.name);
        assert_eq!(100, salary.percentage);
    }

    #[test]
    fn salary_multiplier_check_test_failing(){
        if let Ok(val) = (UncheckedSalaryMultiplier{percentage: 0, name: "Test Employee".to_owned()}).check(){
            panic!("Bad data somehow passsed the check: {:?}", val);
        }
        if let Ok(val) = (UncheckedSalaryMultiplier{percentage: 20, name: "".to_owned()}).check(){
            panic!("Bad data somehow passsed the check: {:?}", val);
        }
        if let Ok(val) = (UncheckedSalaryMultiplier{percentage: -32, name: "Test Employee".to_owned()}).check(){
            panic!("Bad data somehow passsed the check: {:?}", val);
        }
        if let Ok(val) = (UncheckedSalaryMultiplier{percentage: 0, name: "".to_owned()}).check(){
            panic!("Bad data somehow passsed the check: {:?}", val);
        }
    }
}
