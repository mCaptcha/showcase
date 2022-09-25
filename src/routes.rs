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

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::AppCtx;

pub const ROUTES: routes::Routes = routes::Routes::new();
const INDEX_JS: &str = include_str!("./static/index.js");
const PROTECTED: &str = include_str!("./static/protected.html");
const SUCCESS: &str = include_str!("./static/success.html");

pub mod routes {

    pub struct Routes {
        pub index: &'static str,
        pub index_js: &'static str,
        pub success: &'static str,
    }

    impl Routes {
        pub const fn new() -> Self {
            let index = "/";
            let index_js = "/index.js";
            let success = "/success";
            Routes {
                index,
                index_js,
                success
            }
        }
    }
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(index_js);
    cfg.service(success_page);
    cfg.service(protected);
    cfg.service(protected_page);

}

#[actix_web_codegen_const_routes::get(path = "ROUTES.index_js")]
async fn index_js() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(INDEX_JS)
}

#[actix_web_codegen_const_routes::get(path = "ROUTES.index")]
async fn protected_page(ctx: AppCtx) -> impl Responder {
        let html = actix_web::http::header::ContentType::html();

        let page = PROTECTED.replace("MCAPTCHA_URL_REPLACEME", &ctx.captcha_path);
        HttpResponse::Ok().content_type(html).body(page)

}

#[actix_web_codegen_const_routes::get(path = "ROUTES.success")]
async fn success_page() -> impl Responder {
    let html = actix_web::http::header::ContentType::html();

    HttpResponse::Ok().content_type(html).body(SUCCESS)
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct RegisterProtected {
    pub name: String,
    pub comment: String,
    pub mcaptcha__token: String,
}

#[actix_web_codegen_const_routes::post(path = "ROUTES.index")]
async fn protected(payload: web::Form<RegisterProtected>, ctx: AppCtx) -> impl Responder {
    let payload = payload.into_inner();
    if !ctx.verify_token(&payload.mcaptcha__token).await {
        HttpResponse::BadRequest().body("Invalid Captcha")
    } else {
        HttpResponse::Found().insert_header((actix_web::http::header::LOCATION, ROUTES.success)).finish()
    }
}
