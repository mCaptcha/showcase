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
use std::path::Path;

use config::{Config, ConfigError, Environment, File};
use log::{debug, warn};
use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub port: u32,
    pub domain: String,
    pub ip: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Captcha {
    pub sitekey: String,
    pub mcaptcha_url: Url,
    pub account_secret: String,
}

impl Server {
    #[cfg(not(tarpaulin_include))]
    pub fn get_ip(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub debug: bool,
    pub server: Server,
    pub captcha: Captcha,
    pub source_code: String,
}

#[cfg(not(tarpaulin_include))]
impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        const CURRENT_DIR: &str = "./config/config.toml";
        const ETC: &str = "/etc/mcaptcha-showcase/config.toml";

        s.set("capatcha.enable_stats", true.to_string())
            .expect("unable to set capatcha.enable_stats default config");

        if let Ok(path) = env::var("MCAPTCHA_CONFIG") {
            s.merge(File::with_name(&path))?;
        } else if Path::new(CURRENT_DIR).exists() {
            // merging default config from file
            s.merge(File::with_name(CURRENT_DIR))?;
        } else if Path::new(ETC).exists() {
            s.merge(File::with_name(ETC))?;
        } else {
            log::warn!("configuration file not found");
        }

        s.merge(Environment::with_prefix("MCAPTCHA").separator("__"))?;

        check_url(&s);

        for (var, key) in [ ( "MCAPTCHA__CAPTCHA__SITEKEY", "captcha.sitekey"),
            ("MCAPTCHA__CAPTCHA__MCAPTCHA_URL", "captcha.mcaptcha_url"),
            ("MCAPTCHA__CAPTCHA__ACCOUNT__SECRET", "captcha.account_secret"  ),
        ].iter() {
        match env::var(var) {
            Ok(val) => {
                s.set(key, val).unwrap();
            }
            Err(e) => warn!("couldn't interpret env var: {var} to set key: {key}"),
        }
        }

        match env::var("PORT") {
            Ok(val) => {
                s.set("server.port", val).unwrap();
            }
            Err(e) => warn!("couldn't interpret PORT: {}", e),
        }

        match s.try_into() {
            Ok(val) => Ok(val),
            Err(e) => Err(ConfigError::Message(format!("\n\nError: {}. If it says missing fields, then please refer to https://github.com/mCaptcha/mcaptcha#configuration to learn more about how mcaptcha reads configuration\n\n", e))),
        }
    }
}

#[cfg(not(tarpaulin_include))]
fn check_url(s: &Config) {
    let url = s
        .get::<String>("source_code")
        .expect("Couldn't access source_code");

    Url::parse(&url).expect("Please enter a URL for source_code in settings");
}
