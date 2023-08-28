# Описание:
Тестовое задание от Wildberries

Написать веб-приложение с тремя эндпоинтами:
- PUT /employee/add?name={Имя работника}&salary={Зарплата работника}
- GET /employee/salary?name={Имя работника}
- POST /employee/increase?name={Имя работника}&percentage{Процент увеличения зарплаты}

# Как запускать?
1) docker compose up

На порту 8080 откроется приложение, в котором можно использовать упомянутые выше эндпоинты.