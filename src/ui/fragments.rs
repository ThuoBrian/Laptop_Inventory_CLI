use crate::{
    db,
    error::{AppError, UiResult},
    models::*,
    validation,
};
use actix_web::{get, post, web, HttpResponse};
use minijinja::Environment;
use sqlx::PgPool;
use uuid::Uuid;

async fn render_laptop_table(
    pool: &PgPool,
    env: &Environment<'static>,
    status: Option<LaptopStatus>,
    page: i64,
) -> Result<String, AppError> {
    let result =
        db::laptops::get_all_laptops_with_assignee(pool, status.clone(), page, DEFAULT_PER_PAGE)
            .await?;
    let status_str = status.as_ref().map(|s| s.to_string()).unwrap_or_default();
    let tmpl = env
        .get_template("partials/laptop_table.html")
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    tmpl.render(minijinja::context! {
        laptops => result.data,
        page => result.page,
        total_pages => result.total_pages,
        status_filter => status_str,
    })
    .map_err(|e| AppError::BadRequest(e.to_string()))
}

fn laptop_table_oob(table_html: &str) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!(
            "<div hx-swap-oob=\"innerHTML:#laptop-table\">{table_html}</div>"
        ))
}

async fn render_user_table(
    pool: &PgPool,
    env: &Environment<'static>,
    page: i64,
) -> Result<String, AppError> {
    let result = db::users::get_all_users(pool, page, DEFAULT_PER_PAGE).await?;
    let tmpl = env
        .get_template("partials/user_table.html")
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    tmpl.render(minijinja::context! {
        users => result.data,
        page => result.page,
        total_pages => result.total_pages,
    })
    .map_err(|e| AppError::BadRequest(e.to_string()))
}

fn user_table_oob(table_html: &str) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!(
            "<div hx-swap-oob=\"innerHTML:#user-table\">{table_html}</div>"
        ))
}

// ── Laptop fragments ──────────────────────────────────────────

#[get("/ui/laptops/new-form")]
pub async fn new_laptop_form(
    env: web::Data<Environment<'static>>,
    query: web::Query<LaptopListQuery>,
) -> UiResult {
    UiResult(async move {
        let status_filter = query.status.as_ref().map(|s| s.to_string()).unwrap_or_default();
        let current_page = clamp_page(query.page);
        super::render_template(
            &env,
            "partials/laptop_form.html",
            minijinja::context! { status_filter, current_page },
        )
    }.await)
}

#[post("/ui/laptops/new")]
pub async fn create_laptop_form(
    pool: web::Data<PgPool>,
    env: web::Data<Environment<'static>>,
    query: web::Query<LaptopListQuery>,
    form: web::Form<CreateLaptop>,
) -> UiResult {
    UiResult(
        async move {
            let form = form.into_inner();
            let brand = validation::validate_required_string(&form.brand, "Brand")?;
            let model = validation::validate_required_string(&form.model, "Model")?;
            let serial_number =
                validation::validate_required_string(&form.serial_number, "Serial number")?;
            let validated = CreateLaptop {
                brand,
                model,
                serial_number,
                purchase_date: form.purchase_date,
            };
            db::laptops::create_laptop(&pool, validated).await?;
            let status = query.status.clone();
            let page = clamp_page(query.page);
            let table_html = render_laptop_table(&pool, &env, status, page).await?;
            Ok(laptop_table_oob(&table_html))
        }
        .await,
    )
}

#[get("/ui/laptops/{id}/edit-form")]
pub async fn edit_laptop_form(
    pool: web::Data<PgPool>,
    env: web::Data<Environment<'static>>,
    path: web::Path<Uuid>,
    query: web::Query<LaptopListQuery>,
) -> UiResult {
    UiResult(
        async move {
            let laptop = db::laptops::get_laptop_by_id(&pool, path.into_inner()).await?;
            let status_filter = query.status.as_ref().map(|s| s.to_string()).unwrap_or_default();
            let current_page = clamp_page(query.page);
            super::render_template(
                &env,
                "partials/laptop_form.html",
                minijinja::context! { laptop, status_filter, current_page },
            )
        }
        .await,
    )
}

#[post("/ui/laptops/{id}/edit")]
pub async fn update_laptop_form(
    pool: web::Data<PgPool>,
    env: web::Data<Environment<'static>>,
    path: web::Path<Uuid>,
    query: web::Query<LaptopListQuery>,
    form: web::Form<UpdateLaptop>,
) -> UiResult {
    UiResult(
        async move {
            let form = form.into_inner();
            let brand = validation::validate_optional_string(form.brand.as_ref(), "Brand")?;
            let model = validation::validate_optional_string(form.model.as_ref(), "Model")?;
            let serial_number =
                validation::validate_optional_string(form.serial_number.as_ref(), "Serial number")?;
            let validated = UpdateLaptop {
                brand,
                model,
                serial_number,
                status: form.status,
                purchase_date: form.purchase_date,
            };
            db::laptops::update_laptop(&pool, path.into_inner(), validated).await?;
            let status = query.status.clone();
            let page = clamp_page(query.page);
            let table_html = render_laptop_table(&pool, &env, status, page).await?;
            Ok(laptop_table_oob(&table_html))
        }
        .await,
    )
}

#[post("/ui/laptops/{id}/delete")]
pub async fn delete_laptop_form(
    pool: web::Data<PgPool>,
    env: web::Data<Environment<'static>>,
    path: web::Path<Uuid>,
    query: web::Query<LaptopListQuery>,
) -> UiResult {
    UiResult(
        async move {
            db::laptops::delete_laptop(&pool, path.into_inner()).await?;
            let status = query.status.clone();
            let page = clamp_page(query.page);
            let table_html = render_laptop_table(&pool, &env, status, page).await?;
            Ok(laptop_table_oob(&table_html))
        }
        .await,
    )
}

#[get("/ui/laptops/{id}/assign-form")]
pub async fn assign_laptop_form_get(
    pool: web::Data<PgPool>,
    env: web::Data<Environment<'static>>,
    path: web::Path<Uuid>,
    query: web::Query<LaptopListQuery>,
) -> UiResult {
    UiResult(
        async move {
            let laptop_id = path.into_inner();
            let users = db::users::get_users_for_dropdown(&pool).await?;
            let status_filter = query.status.as_ref().map(|s| s.to_string()).unwrap_or_default();
            let current_page = clamp_page(query.page);
            super::render_template(
                &env,
                "partials/assign_modal.html",
                minijinja::context! {
                    laptop_id => laptop_id.to_string(),
                    users,
                    status_filter,
                    current_page,
                },
            )
        }
        .await,
    )
}

#[post("/ui/laptops/{id}/assign")]
pub async fn assign_laptop_form(
    pool: web::Data<PgPool>,
    env: web::Data<Environment<'static>>,
    path: web::Path<Uuid>,
    query: web::Query<LaptopListQuery>,
    form: web::Form<AssignLaptop>,
) -> UiResult {
    UiResult(
        async move {
            db::laptops::assign_laptop(&pool, path.into_inner(), form.into_inner().user_id).await?;
            let status = query.status.clone();
            let page = clamp_page(query.page);
            let table_html = render_laptop_table(&pool, &env, status, page).await?;
            Ok(laptop_table_oob(&table_html))
        }
        .await,
    )
}

#[post("/ui/laptops/{id}/unassign")]
pub async fn unassign_laptop_form(
    pool: web::Data<PgPool>,
    env: web::Data<Environment<'static>>,
    path: web::Path<Uuid>,
    query: web::Query<LaptopListQuery>,
) -> UiResult {
    UiResult(
        async move {
            db::laptops::unassign_laptop(&pool, path.into_inner()).await?;
            let status = query.status.clone();
            let page = clamp_page(query.page);
            let table_html = render_laptop_table(&pool, &env, status, page).await?;
            Ok(laptop_table_oob(&table_html))
        }
        .await,
    )
}

// ── User fragments ────────────────────────────────────────────

#[get("/ui/users/new-form")]
pub async fn new_user_form(
    env: web::Data<Environment<'static>>,
    query: web::Query<PaginationParams>,
) -> UiResult {
    UiResult(async move {
        let current_page = clamp_page(query.page);
        super::render_template(
            &env,
            "partials/user_form.html",
            minijinja::context! { current_page },
        )
    }.await)
}

#[post("/ui/users/new")]
pub async fn create_user_form(
    pool: web::Data<PgPool>,
    env: web::Data<Environment<'static>>,
    query: web::Query<PaginationParams>,
    form: web::Form<CreateUser>,
) -> UiResult {
    UiResult(
        async move {
            let form = form.into_inner();
            let username = validation::validate_required_string(&form.username, "Username")?;
            let email = validation::validate_email(&form.email)?;
            let department = validation::validate_required_string(&form.department, "Department")?;
            let validated = CreateUser {
                username,
                email,
                department,
            };
            db::users::create_user(&pool, validated).await?;
            let page = clamp_page(query.page);
            let table_html = render_user_table(&pool, &env, page).await?;
            Ok(user_table_oob(&table_html))
        }
        .await,
    )
}

#[get("/ui/users/{id}/edit-form")]
pub async fn edit_user_form(
    pool: web::Data<PgPool>,
    env: web::Data<Environment<'static>>,
    path: web::Path<Uuid>,
    query: web::Query<PaginationParams>,
) -> UiResult {
    UiResult(
        async move {
            let user = db::users::get_user_by_id(&pool, path.into_inner()).await?;
            let current_page = clamp_page(query.page);
            super::render_template(
                &env,
                "partials/user_form.html",
                minijinja::context! { user, current_page },
            )
        }
        .await,
    )
}

#[post("/ui/users/{id}/edit")]
pub async fn update_user_form(
    pool: web::Data<PgPool>,
    env: web::Data<Environment<'static>>,
    path: web::Path<Uuid>,
    query: web::Query<PaginationParams>,
    form: web::Form<UpdateUser>,
) -> UiResult {
    UiResult(
        async move {
            let form = form.into_inner();
            let username =
                validation::validate_optional_string(form.username.as_ref(), "Username")?;
            let email = match form.email.as_ref() {
                Some(e) => Some(validation::validate_email(e)?),
                None => None,
            };
            let department =
                validation::validate_optional_string(form.department.as_ref(), "Department")?;
            let validated = UpdateUser {
                username,
                email,
                department,
            };
            db::users::update_user(&pool, path.into_inner(), validated).await?;
            let page = clamp_page(query.page);
            let table_html = render_user_table(&pool, &env, page).await?;
            Ok(user_table_oob(&table_html))
        }
        .await,
    )
}

#[post("/ui/users/{id}/delete")]
pub async fn delete_user_form(
    pool: web::Data<PgPool>,
    env: web::Data<Environment<'static>>,
    path: web::Path<Uuid>,
    query: web::Query<PaginationParams>,
) -> UiResult {
    UiResult(
        async move {
            db::users::delete_user(&pool, path.into_inner()).await?;
            let page = clamp_page(query.page);
            let table_html = render_user_table(&pool, &env, page).await?;
            Ok(user_table_oob(&table_html))
        }
        .await,
    )
}