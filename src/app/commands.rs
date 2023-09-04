mod ckey;
mod config;
mod help;
mod verification;
mod whitelist;

pub use ckey::{ckey, ckey_get_ctx, ckey_unset_ctx, ckey_unwhitelist_ctx, ckey_whitelist_ctx};
pub use config::{
    config, config_greeting_message_ctx, config_rejected_message_ctx, config_verified_message_ctx,
};
pub use help::{help, license, source, version};
pub use verification::{
    verification, verification_clear_ctx, verification_greet_ctx, verification_status_ctx,
};
pub use whitelist::whitelist;

pub fn commands() -> Vec<poise::Command<crate::app::Data, crate::app::Error>> {
    vec![
        ckey(),
        config(),
        help(),
        license(),
        source(),
        version(),
        verification(),
        whitelist(),
        ckey_get_ctx(),
        config_greeting_message_ctx(),
        config_rejected_message_ctx(),
        config_verified_message_ctx(),
    ]
}
