use crate::{db, error::AppError, models::*};
use actix_web::{HttpResponse, get, web, HttpRequest};
use minijinja::Environment;
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct LaptopListQuery {
    pub status: Option<LaptopStatus>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(serde::Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

fn is_htmx_request(req: &HttpRequest) -> bool {
    req.headers()
        .get("HX-Request")
        .and_then(|v| v.to_str().ok())
        == Some("true")
}

fn render_template(
    env: &Environment<'static>,
    name: &str,
    ctx: minijinja::Value,
) -> Result<HttpResponse, AppError> {
    let tmpl = env
        .get_template(name)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    let html = tmpl
        .render(ctx)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

#[get("/ui")]
pub async fn dashboard(
    pool: web::Data<PgPool>,
    env: web::Data<Environment<'static>>,
) -> Result<HttpResponse, AppError> {
    let total_laptops = db::laptops::count_laptops(&pool).await?;
    let total_users = db::users::count_users(&pool).await?;
    let status_counts = db::laptops::count_laptops_by_status(&pool).await?;

    render_template(
        &env,
        "pages/dashboard.html",
        minijinja::context! {
            page_id => "dashboard",
            total_laptops,
            total_users,
            status_counts,
        },
    )
}

#[get("/ui/laptops")]
pub async fn laptops_page(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    env: web::Data<Environment<'static>>,
    query: web::Query<LaptopListQuery>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(DEFAULT_PAGE);
    let per_page = query.per_page.unwrap_or(DEFAULT_PER_PAGE).min(MAX_PER_PAGE);
    let status = query.status.clone();
    let result = db::laptops::get_all_laptops_with_assignee(&pool, status, page, per_page).await?;

    if is_htmx_request(&req) {
        return render_template(
            &env,
            "partials/laptop_table.html",
            minijinja::context! {
                laptops => result.data,
                page => result.page,
                total_pages => result.total_pages,
            },
        );
    }

    render_template(
        &env,
        "pages/laptops.html",
        minijinja::context! {
            page_id => "laptops",
        },
    )
}

#[get("/ui/users")]
pub async fn users_page(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    env: web::Data<Environment<'static>>,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(DEFAULT_PAGE);
    let per_page = query.per_page.unwrap_or(DEFAULT_PER_PAGE).min(MAX_PER_PAGE);
    let result = db::users::get_all_users(&pool, page, per_page).await?;

    if is_htmx_request(&req) {
        return render_template(
            &env,
            "partials/user_table.html",
            minijinja::context! {
                users => result.data,
                page => result.page,
                total_pages => result.total_pages,
            },
        );
    }

    render_template(
        &env,
        "pages/users.html",
        minijinja::context! {
            page_id => "users",
        },
    )
}