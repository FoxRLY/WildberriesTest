use std::error::Error;
use std::fmt::Display;

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

// Модели данных


/// Модель непроверенного имени сотрудника
///
/// Имя сотрудника, приходящее с эндпоинта и подлежащее проверке
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UncheckedEmployeeName{
    name: String,
}

impl UncheckedEmployeeName{
    /// Sanity-check для входящего имени
    ///
    /// Преобразует непроверенные данные в проверенные, поглощая объект.
    /// Проверка - имя не должно состоять из пробелов или иметь нулевую длину
    pub fn check(self) -> Result<EmployeeName, Box<dyn Error>> {
        if self.name.trim().is_empty() {
            Err(CustomError{msg: "employee name cannot consist of whitespaces or have zero length"})?
        }
        Ok(EmployeeName{name:self.name})
    }
}


/// Модель имени сотрудника
///
/// Проверенное значение имени сотрудника
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EmployeeName{
    pub name: String
}



/// Модель Непроверенной зарплаты сотрудника
///
/// Значение зарплаты сотрудника, приходящее с эндпоинта и подлежащее проверке
pub struct UncheckedEmployeeSalary{
    amount: u32,
}

impl UncheckedEmployeeSalary{
    /// Sanity-check для значения зарплаты сотрудника
    ///
    /// Преобразует непроверенные данные в проверенные, поглощая объект
    /// Проверка - зарплата не может быть меньше либо равной нулю
    pub fn check(self) -> Result<EmployeeSalary, Box<dyn Error>> {
        if self.amount <= 0 {
            Err(CustomError{msg: "employee salary cannot be equal to zero"})?;
        }
        Ok(EmployeeSalary{amount:self.amount})
    }
}


/// Модель Зарплаты сотрудника
///
/// Проверенное значение зарплаты сотрудника
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct EmployeeSalary{
    pub amount: u32,
}

impl EmployeeSalary{
    /// Проверить значение на соответствие условиям
    fn check(&self) -> Result<(), CustomError> {
        if self.amount <= 0 {
            return Err(CustomError{msg: "employee's salary cannot be equal to zero in result of addition"});
        }
        Ok(())
    }
    
    /// Увеличить зарплату на процент
    ///
    /// Увеличивает зарплату на определенный процент с необходимыми проверками и делает возвращает
    /// старое значение
    pub fn increase_by_percentage(&mut self, percent: SalaryMultiplier) -> Result<EmployeeSalary, Box<dyn Error>> {
        let old_salary = self.clone();
        let mut addition = self.amount.checked_add(100)
            .ok_or(CustomError{msg:"employee's salary is too high to perform math operations"})?;
        addition = addition.checked_sub(1)
            .ok_or(CustomError{msg:"employee's salary is too low to perform math operations"})?;
        addition = addition.checked_div(100)
            .ok_or(CustomError{msg:"employee's salary is too low to perform math operations"})?;
        addition = addition.checked_mul(percent.percentage)
            .ok_or(CustomError{msg:"employee's salary is too high to perform math operations"})?;



        self.amount = self.amount.checked_add(addition)
            .ok_or(CustomError{msg:"employee's salary is too high to perform math operations"})?;
        self.check()?;
        Ok(old_salary)
    }
}



/// Модель Непроверенных данных о работнике
///
/// Данные, приходящие с эндпоинта и посдлежащие проверке
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UncheckedEmployeeData{
    name: String,
    salary: u32,
}

impl UncheckedEmployeeData{
    pub fn check(self) -> Result<EmployeeData, Box<dyn Error>> {
        if self.name.trim().is_empty(){
            Err(CustomError{msg: "employee's salary cannot be equal to zero"})?;
        }
        if self.salary <= 0 {
            Err(CustomError{msg: "employee's salary cannot be equal to zero"})?;
        }
        Ok(EmployeeData { name: self.name, salary: self.salary })
    }
}


/// Модель данных о работнике
///
/// Проверенное значение данных о работнике
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EmployeeData{
    pub name: String,
    pub salary: u32,
}


/// Модель Непроверенного процента повышения зарплаты
///
/// Процент повышения зарплаты, приходящий с эндпоинта и посдлежащий проверке
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UncheckedSalaryMultiplier{
    percentage: u32,
}

impl UncheckedSalaryMultiplier{
    /// Sanity-check для значения процента от зарплаты
    ///
    /// Преобразует непроверенные данные в проверенные, поглощая объект
    /// Проверка - процент не может быть равен нулю
    pub fn check(self) -> Result<SalaryMultiplier, Box<dyn Error>> {
        if self.percentage <= 0 {
            Err(CustomError{msg: "salary multiplier cannot be equal to zero"})?;
        }
        Ok(SalaryMultiplier{percentage:self.percentage})
    }
}



/// Модель Зарплаты сотрудника
///
/// Проверенное значение зарплаты сотрудника
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SalaryMultiplier{
    pub percentage: u32,
}
