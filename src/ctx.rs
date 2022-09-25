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
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
//! App data: redis cache, database connections, etc.
use std::str::FromStr;
use std::sync::Arc;
use std::thread;

use argon2_creds::{Config, ConfigBuilder, PasswordPolicy};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::settings::Settings;

/// App data
pub struct Ctx {
    /// credential management configuration
    pub creds: Config,
    /// app settings
    pub settings: Settings,

    /// HTTP client
    pub http: Client,
    verify_path: String,
    pub captcha_path: String,
}

impl Ctx {
    pub fn get_creds() -> Config {
        ConfigBuilder::default()
            .username_case_mapped(true)
            .profanity(true)
            .blacklist(true)
            .password_policy(PasswordPolicy::default())
            .build()
            .unwrap()
    }
    #[cfg(not(tarpaulin_include))]
    /// create new instance of app data
    pub async fn new(s: &Settings) -> Arc<Self> {
        let creds = Self::get_creds();
        let c = creds.clone();

        #[allow(unused_variables)]
        let init = thread::spawn(move || {
            log::info!("Initializing credential manager");
            c.init();
            log::info!("Initialized credential manager");
        });

        let mut verify_path = s.captcha.mcaptcha_url.clone();
        verify_path.set_path("/api/v1/pow/siteverify");
        let verify_path = verify_path.into();

        let mut captcha_path = s.captcha.mcaptcha_url.clone();
        captcha_path.set_path("/widget");
        captcha_path.set_query(Some(&format!("sitekey={}", s.captcha.sitekey)));

        let captcha_path = captcha_path.into();
        let data = Ctx {
            creds,
            settings: s.clone(),
            http: Client::new(),
            verify_path,
            captcha_path,
        };
        #[cfg(not(debug_assertions))]
        init.join().unwrap();
        Arc::new(data)
    }

    pub async fn verify_token(&self, token: &str) -> bool {
        let payload = CaptchaVerfiyPayload::from_ctx(&self, &token);
        let res: CaptchaVerifyResp = self
            .http
            .post(&self.verify_path)
            .json(&payload)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        res.valid
    }
}

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
struct CaptchaVerifyResp {
    valid: bool,
}

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
struct CaptchaVerfiyPayload<'a> {
    token: &'a str,
    key: &'a str,
    secret: &'a str,
}

impl<'a> CaptchaVerfiyPayload<'a> {
    fn from_ctx(ctx: &'a Ctx, token: &'a str) -> Self {
        Self {
            key: &ctx.settings.captcha.sitekey,
            token,
            secret: &ctx.settings.captcha.account_secret,
        }
    }
}
