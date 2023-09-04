mod app;
mod config;

pub const SOURCE: &'static str = "https://github.com/SS13-Aeon/nightstation-verify";
pub const COPYRIGHT: &'static str = r#"Nightstation verification bot
Copyright (C) 2023  Nightstation contributors

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as
published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>."#;

pub use app::App;
pub use app::AppPaths;
pub type AppError = app::Error;
pub use config::AppConfig;
pub type AppConfigError = config::Error;
