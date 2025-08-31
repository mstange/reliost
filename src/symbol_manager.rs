use std::sync::Arc;

use samply_quota_manager::QuotaManager;
use wholesym::{SymbolManager, SymbolManagerConfig};

use crate::configuration::{QuotaSettings, Settings};
use crate::symbol_manager_observer::QuotaManagingSymbolManagerObserver;

#[tracing::instrument(name = "Create symbol manager", skip_all)]
pub fn create_symbol_manager_and_quota_manager(
    settings: Settings,
) -> (SymbolManager, Option<QuotaManager>) {
    let config = create_symbol_manager_config(&settings);
    let quota_manager = create_quota_manager(&settings);

    let quota_manager_notifiers = match &quota_manager {
        Some(qm) => {
            let notifier = qm.notifier();

            // Enforce size and age limit now
            notifier.trigger_eviction_if_needed();

            vec![notifier]
        }
        None => {
            tracing::warn!(
                "No quota manager configured! Check [quota] in configuration/base.toml."
            );
            tracing::warn!(
                "Without a quota manager, downloaded files will accumulate without bound."
            );
            vec![]
        }
    };

    let mut symbol_manager = SymbolManager::with_config(config);
    let observer = QuotaManagingSymbolManagerObserver::new(quota_manager_notifiers);
    symbol_manager.set_observer(Some(Arc::new(observer)));
    (symbol_manager, quota_manager)
}

fn create_symbol_manager_config(settings: &Settings) -> SymbolManagerConfig {
    let mut config = SymbolManagerConfig::default();
    if let Some(symbols) = settings.symbols.as_ref() {
        if let Some(breakpad) = symbols.breakpad.as_ref() {
            if breakpad.servers.is_empty() {
                config = config.breakpad_symbol_dir(&breakpad.cache_dir);
            } else {
                for server_url in &breakpad.servers {
                    config = config.breakpad_symbol_server(server_url, breakpad.cache_dir.clone());
                }
            }
            if let Some(symindex_dir) = &breakpad.symindex_dir {
                config = config.breakpad_symindex_cache_dir(symindex_dir);
            }
        }
        if let Some(windows) = symbols.windows.as_ref() {
            for server_url in &windows.servers {
                config = config.windows_symbol_server(server_url, windows.cache_dir.clone());
            }
        }
    }
    config
}

fn create_quota_manager(settings: &Settings) -> Option<QuotaManager> {
    let QuotaSettings {
        managed_dir,
        db_path,
        size_limit,
        age_limit,
    } = settings.quota.as_ref()?.clone();

    if let Err(e) = std::fs::create_dir_all(&managed_dir) {
        panic!("Could not create quota managed directory {managed_dir:?}: {e}");
    }

    let quota_manager = match QuotaManager::new(&managed_dir, &db_path) {
        Ok(quota_manager) => quota_manager,
        Err(e) => {
            panic!("Could not create QuotaManager with symbol cache database {db_path:?}: {e}");
        }
    };

    quota_manager.set_max_total_size(size_limit);
    quota_manager.set_max_age(age_limit.map(|d| d.as_secs()));
    Some(quota_manager)
}
