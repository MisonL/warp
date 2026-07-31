use settings::{SupportedPlatforms, SyncToCloud, define_settings_group};
pub use warp_localization::AppLanguage;

define_settings_group!(LanguageSettings, settings: [
    app_language: AppLanguageSetting {
        type: AppLanguage,
        default: AppLanguage::default(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        surface: settings::SettingSurfaces::GUI,
        private: false,
        storage_key: "AppLanguage",
        toml_path: "appearance.interface.language",
        description: "The display language used by Warp.",
    },
]);
