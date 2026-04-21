use crate::{
    db,
    error::UiResult,
    models::*,
};
use actix_web::{get, web, HttpRequest};
use minijinja::Environment;
use sqlx::PgPool;

fn is_htmx_request(req: &HttpRequest) -> bool {
    req.headers()
        .get("HX-Request")
        .and_then(|v| v.to_str().ok())
        == Some("true")
}

#[get("/ui")]
pub async fn dashboard(
    pool: web::Data<PgPool>,
    env: web::Data<Environment<'static>>,
) -> UiResult {
    UiResult(async move {
        let total_laptops = db::laptops::count_laptops(&pool).await?;
        let total_users = db::users::count_users(&pool).await?;
        let status_counts = db::laptops::count_laptops_by_status(&pool).await?;
        let recent_laptops = db::laptops::get_recent_laptops(&pool, 5).await?;

        super::render_template(
            &env,
            "pages/dashboard.html",
            minijinja::context! {
                page_id => "dashboard",
                total_laptops,
                total_users,
                status_counts,
                recent_laptops,
            },
        )
    }.await)
}

#[get("/ui/laptops")]
pub async fn laptops_page(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    env: web::Data<Environment<'static>>,
    query: web::Query<LaptopListQuery>,
) -> UiResult {
    UiResult(async move {
        let page = clamp_page(query.page);
        let per_page = clamp_per_page(query.per_page);
        let status = query.status.clone();
        let status_str = status.as_ref().map(|s| s.to_string());
        let result = db::laptops::get_all_laptops_with_assignee(&pool, status, page, per_page).await?;

        if is_htmx_request(&req) {
            return super::render_template(
                &env,
                "partials/laptop_table.html",
                minijinja::context! {
                    laptops => result.data,
                    page => result.page,
                    total_pages => result.total_pages,
                    status_filter => status_str,
                },
            );
        }

        super::render_template(
            &env,
            "pages/laptops.html",
            minijinja::context! {
                page_id => "laptops",
            },
        )
    }.await)
}

#[get("/ui/users")]
pub async fn users_page(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    env: web::Data<Environment<'static>>,
    query: web::Query<PaginationParams>,
) -> UiResult {
    UiResult(async move {
        let page = clamp_page(query.page);
        let per_page = clamp_per_page(query.per_page);
        let result = db::users::get_all_users(&pool, page, per_page).await?;

        if is_htmx_request(&req) {
            return super::render_template(
                &env,
                "partials/user_table.html",
                minijinja::context! {
                    users => result.data,
                    page => result.page,
                    total_pages => result.total_pages,
                },
            );
        }

        super::render_template(
            &env,
            "pages/users.html",
            minijinja::context! {
                page_id => "users",
            },
        )
    }.await)
}
