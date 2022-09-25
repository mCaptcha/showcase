/*
 * Copyright (C) 2022  Aravinth Manivannan <realaravinth@batsense.net>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::env;
use std::sync::Arc;

use actix_web::{
    error::InternalError, http::StatusCode, middleware as actix_middleware, web::JsonConfig, App,
    HttpServer,
};
use log::info;

mod ctx;
mod routes;
mod settings;

pub use crate::ctx::Ctx;
pub use settings::Settings;

pub const COMPILED_DATE: &str = env!("COMPILED_DATE");
pub const GIT_COMMIT_HASH: &str = env!("GIT_HASH");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const PKG_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

pub const CACHE_AGE: u32 = 604800;

pub type ArcCtx = Arc<crate::ctx::Ctx>;
pub type AppCtx = actix_web::web::Data<ArcCtx>;

#[cfg(not(tarpaulin_include))]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "info");

    pretty_env_logger::init();
    info!(
        "{}: {}.\nFor more information, see: {}\nBuild info:\nVersion: {} commit: {}",
        PKG_NAME, PKG_DESCRIPTION, PKG_HOMEPAGE, VERSION, GIT_COMMIT_HASH
    );

    let settings = Settings::new().unwrap();
    println!("{:?}", settings.captcha);
    let ctx = ctx::Ctx::new(&settings).await;
    let ctx = AppCtx::new(ctx);

    let ip = settings.server.get_ip();
    println!("Starting server on: http://{ip}");

    HttpServer::new(move || {
        App::new()
            .wrap(actix_middleware::Logger::default())
            .wrap(
                actix_middleware::DefaultHeaders::new()
                    .add(("Permissions-Policy", "interest-cohort=()")),
            )
            .wrap(actix_middleware::Compress::default())
            .app_data(ctx.clone())
            .wrap(actix_middleware::NormalizePath::new(
                actix_middleware::TrailingSlash::Trim,
            ))
            .configure(routes::services)
            .app_data(get_json_err())
    })
    .bind(&ip)
    .unwrap()
    .run()
    .await?;

    Ok(())
}

#[cfg(not(tarpaulin_include))]
pub fn get_json_err() -> JsonConfig {
    JsonConfig::default().error_handler(|err, _| {
        //debug!("JSON deserialization error: {:?}", &err);
        InternalError::new(err, StatusCode::BAD_REQUEST).into()
    })
}
