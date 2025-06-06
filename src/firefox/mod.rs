mod installations;
pub mod resource;
mod xulstore;
mod policies;

pub use installations::installations;

use crate::{firefox_common, logging::success, ARGS};
use tracing::{debug_span, instrument, warn};
use crate::browsers::Installation;

#[instrument(level = "debug")]
pub fn debloat(installation: &Installation) -> color_eyre::Result<()> {
    let mut custom_overrides = vec![
        include_str!("../../snippets/firefox_common/betterfox_extra"),
        include_str!("../../snippets/firefox/extra"),
    ];

    if ARGS.get().unwrap().vertical_tabs {
        custom_overrides.push(include_str!("../../snippets/firefox/vert_tabs"));
    }

    if ARGS.get().unwrap().search_suggestions {
        custom_overrides.push(include_str!("../../snippets/firefox_common/search_suggestions"));
    }

    let profiles = firefox_common::debloat(
        installation,
        resource::get_better_fox_user_js()?,
        &custom_overrides.join("\n")
    )?;
    
    if !ARGS.get().unwrap().vertical_tabs {
        return Ok(());
    }

    for profile in profiles {
        let span = debug_span!("Updating xulstore", %profile);
        let _enter = span.enter();

        match xulstore::xulstore(&profile.path) {
            Ok(()) => success(&format!("Updated xulstore.json for {profile}")),
            Err(why) => warn!(err = %why, "Failed to update xulstore.json for {profile}"),
        }
    }
    
    if !ARGS.get().unwrap().create_policies {
        return Ok(());
    }
    
    if installation.app_folders.is_empty() {
        warn!("No app folders found for Firefox, skipping creating policies");
        return Ok(());
    }
    
    for folder in &installation.app_folders {
        if let Err(why) = policies::create_policies_file(folder) {
            warn!(err = %why, "Failed to create policies");
            return Ok(());
        }
    }

    Ok(())
}
