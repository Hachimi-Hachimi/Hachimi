use std::{fs, path::{Path, PathBuf}, process, sync::{atomic::{self, AtomicBool, AtomicI32}, Arc, Mutex, MutexGuard}};
use arc_swap::ArcSwap;
use fnv::FnvHashMap;
use once_cell::sync::OnceCell;
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{gui_impl, hachimi_impl, il2cpp::{self, hook::umamusume::GameSystem}};

use super::{game::Game, ipc, plurals, template, template_filters, tl_repo, utils, Error, Interceptor};

pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::default());

pub struct Hachimi {
    // Hooking stuff
    pub interceptor: Mutex<Interceptor>,
    pub hooking_finished: AtomicBool,

    // Localized data
    pub localized_data: ArcSwap<LocalizedData>,
    pub tl_updater: Arc<tl_repo::Updater>,

    // Shared properties
    pub game: Game,
    pub config: ArcSwap<Config>,
    pub template_parser: template::Parser,

    /// -1 = default
    pub target_fps: AtomicI32,

    #[cfg(target_os = "windows")]
    pub vsync_count: AtomicI32,

    #[cfg(target_os = "windows")]
    pub window_always_on_top: AtomicBool,

    #[cfg(target_os = "windows")]
    pub updater: Arc<crate::windows::updater::Updater>
}

static INSTANCE: OnceCell<Arc<Hachimi>> = OnceCell::new();

impl Hachimi {
    pub fn init() -> bool {
        if INSTANCE.get().is_some() {
            warn!("Hachimi should be initialized only once");
            return true;
        }

        let instance = match Self::new() {
            Ok(v) => v,
            Err(e) => {
                super::log::init(false); // early init to log error
                error!("Init failed: {}", e);
                return false;
            }
        };

        super::log::init(instance.config.load().debug_mode);

        info!("Hachimi {}", env!("HACHIMI_DISPLAY_VERSION"));
        instance.load_localized_data();

        INSTANCE.set(Arc::new(instance)).is_ok()
    }

    pub fn instance() -> Arc<Hachimi> {
        INSTANCE.get().unwrap_or_else(|| {
            error!("FATAL: Attempted to get Hachimi instance before initialization");
            process::exit(1);
        }).clone()
    }

    fn new() -> Result<Hachimi, Error> {
        let game = Game::init();
        let config = Self::load_config(&game.data_dir)?;

        Ok(Hachimi {
            interceptor: Mutex::default(),
            hooking_finished: AtomicBool::new(false),

            // Don't load localized data initially since it might fail, logging the error is not possible here
            localized_data: ArcSwap::new(Arc::default()),
            tl_updater: Arc::default(),

            game,
            template_parser: template::Parser::new(&template_filters::LIST),

            target_fps: AtomicI32::new(config.target_fps.unwrap_or(-1)),

            #[cfg(target_os = "windows")]
            vsync_count: AtomicI32::new(config.windows.vsync_count),

            #[cfg(target_os = "windows")]
            window_always_on_top: AtomicBool::new(config.windows.window_always_on_top),

            #[cfg(target_os = "windows")]
            updater: Arc::default(),

            config: ArcSwap::new(Arc::new(config))
        })
    }

    pub fn interceptor(&self) -> MutexGuard<'_, Interceptor> {
        self.interceptor.lock().unwrap()
    }

    fn load_config(data_dir: &Path) -> Result<Config, Error> {
        let config_path = data_dir.join("config.json");
        if fs::metadata(&config_path).is_ok() {
            let json = fs::read_to_string(&config_path)?;
            Ok(serde_json::from_str(&json)?)
        }
        else {
            Ok(Config::default())
        }
    }

    pub fn reload_config(&self) {
        let new_config = match Self::load_config(&self.game.data_dir) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to reload config: {}", e);
                return;
            }
        };
        self.config.store(Arc::new(new_config));
    }

    pub fn save_and_reload_config(&self, config: Config) -> Result<(), Error> {
        fs::create_dir_all(&self.game.data_dir)?;
        let config_path = self.get_data_path("config.json");
        utils::write_json_file(&config, &config_path)?;

        self.config.store(Arc::new(config));
        Ok(())
    }

    pub fn load_localized_data(&self) {
        if self.tl_updater.progress().is_some() {
            warn!("Update in progress, not loading localized data");
            return;
        }
        let new_data = match LocalizedData::new(&self.config.load(), &self.game.data_dir) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to load localized data: {}", e);
                return;
            }
        };
        self.localized_data.store(Arc::new(new_data));
    }

    pub fn on_dlopen(&self, filename: &str, handle: usize) -> bool {
        // Prevent double initialization
        if self.hooking_finished.load(atomic::Ordering::Relaxed) { return false; }

        if hachimi_impl::is_il2cpp_lib(filename) {
            info!("Got il2cpp handle");
            il2cpp::symbols::set_handle(handle);
            false
        }
        else if hachimi_impl::is_criware_lib(filename) {
            self.on_hooking_finished();
            true
        }
        else {
            false
        }
    }

    pub fn on_hooking_finished(&self) {
        self.hooking_finished.store(true, atomic::Ordering::Relaxed);

        info!("GameAssembly finished loading");
        il2cpp::symbols::init();
        il2cpp::hook::init();

        // By the time it finished hooking the game will have already finished initializing
        GameSystem::on_game_initialized();

        let config = self.config.load();
        if !config.disable_gui {
            gui_impl::init();
        }

        if config.enable_ipc {
            ipc::start_http(config.ipc_listen_all);
        }

        hachimi_impl::on_hooking_finished(self);
    }

    pub fn get_data_path<P: AsRef<Path>>(&self, rel_path: P) -> PathBuf {
        self.game.data_dir.join(rel_path)
    }

    pub fn run_auto_update_check(&self) {
        if !self.config.load().disable_auto_update_check {
            #[cfg(not(target_os = "windows"))]
            self.tl_updater.clone().check_for_updates();

            // Check for hachimi updates first, then translations
            // Don't auto check for tl updates if it's not up to date
            #[cfg(target_os = "windows")]
            self.updater.clone().check_for_updates(|new_update| {
                if !new_update {
                    Hachimi::instance().tl_updater.clone().check_for_updates();
                }
            });
        }
    }
}

fn default_serde_instance<'a, T: Deserialize<'a>>() -> Option<T> {
    let empty_data = std::iter::empty::<((), ())>();
    let empty_deserializer = serde::de::value::MapDeserializer::<_, serde::de::value::Error>::new(empty_data);
    T::deserialize(empty_deserializer).ok()
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub debug_mode: bool,
    #[serde(default)]
    pub translator_mode: bool,
    #[serde(default)]
    pub disable_gui: bool,
    pub localized_data_dir: Option<String>,
    pub target_fps: Option<i32>,
    #[serde(default = "Config::default_open_browser_url")]
    pub open_browser_url: String,
    #[serde(default = "Config::default_virtual_res_mult")]
    pub virtual_res_mult: f32,
    pub translation_repo_index: Option<String>,
    #[serde(default)]
    pub skip_first_time_setup: bool,
    #[serde(default)]
    pub disable_auto_update_check: bool,
    #[serde(default)]
    pub disable_translations: bool,
    #[serde(default = "Config::default_ui_scale")]
    pub ui_scale: f32,
    #[serde(default)]
    pub graphics_quality: crate::il2cpp::hook::umamusume::GraphicSettings::GraphicsQuality,
    #[serde(default = "Config::default_story_choice_auto_select_delay")]
    pub story_choice_auto_select_delay: f32,
    #[serde(default = "Config::default_story_tcps_multiplier")]
    pub story_tcps_multiplier: f32,
    #[serde(default)]
    pub enable_ipc: bool,
    #[serde(default)]
    pub ipc_listen_all: bool,
    #[serde(default)]
    pub force_allow_dynamic_camera: bool,
    #[serde(default)]
    pub live_theater_allow_same_chara: bool,
    pub sugoi_url: Option<String>,
    #[serde(default)]
    pub auto_translate_stories: bool,
    #[serde(default)]
    pub auto_translate_localize: bool,
    #[serde(default)]
    pub disable_skill_name_translation: bool,

    #[cfg(target_os = "windows")]
    #[serde(flatten)]
    pub windows: hachimi_impl::Config,

    #[cfg(target_os = "android")]
    #[serde(flatten)]
    pub android: hachimi_impl::Config
}

impl Config {
    fn default_open_browser_url() -> String { "https://www.google.com/".to_owned() }
    fn default_virtual_res_mult() -> f32 { 1.0 }
    fn default_ui_scale() -> f32 { 1.0 }
    fn default_story_choice_auto_select_delay() -> f32 { 0.75 }
    fn default_story_tcps_multiplier() -> f32 { 1.0 }
}

impl Default for Config {
    fn default() -> Self {
        default_serde_instance().expect("default instance")
    }
}

#[derive(Deserialize, Default, Clone)]
pub struct OsOption<T> {
    #[cfg(target_os = "android")]
    android: Option<T>,

    #[cfg(target_os = "windows")]
    windows: Option<T>
}

impl<T> OsOption<T> {
    pub fn as_ref(&self) -> Option<&T> {
        #[cfg(target_os = "android")]
        return self.android.as_ref();

        #[cfg(target_os = "windows")]
        return self.windows.as_ref();
    }
}

#[derive(Default)]
pub struct LocalizedData {
    pub config: LocalizedDataConfig,
    path: Option<PathBuf>,

    pub localize_dict: FnvHashMap<String, String>,
    pub hashed_dict: FnvHashMap<u64, String>,
    pub text_data_dict: FnvHashMap<i32, FnvHashMap<i32, String>>, // {"category": {"index": "text"}}
    pub character_system_text_dict: FnvHashMap<i32, FnvHashMap<i32, String>>, // {"character_id": {"voice_id": "text"}}
    pub race_jikkyo_comment_dict: FnvHashMap<i32, String>, // {"id": "text"}
    pub race_jikkyo_message_dict: FnvHashMap<i32, String>, // {"id": "text"}
    assets_path: Option<PathBuf>,

    pub plural_form: plurals::Resolver,
    pub ordinal_form: plurals::Resolver
}

impl LocalizedData {
    fn new(config: &Config, data_dir: &Path) -> Result<LocalizedData, Error> {
        if config.disable_translations {
            return Ok(LocalizedData::default());
        }

        let path: Option<PathBuf>;
        let config: LocalizedDataConfig = if let Some(ld_dir) = &config.localized_data_dir {
            let ld_path = Path::new(data_dir).join(ld_dir);

            // Create .nomedia
            #[cfg(target_os = "android")]
            { _ = fs::OpenOptions::new().create_new(true).write(true).open(ld_path.join(".nomedia")); }

            let ld_config_path = ld_path.join("config.json");
            path = Some(ld_path);

            if fs::metadata(&ld_config_path).is_ok() {
                let json = fs::read_to_string(&ld_config_path)?;
                serde_json::from_str(&json)?
            }
            else {
                warn!("Localized data config not found");
                LocalizedDataConfig::default()
            }
        }
        else {
            path = None;
            LocalizedDataConfig::default()
        };

        let plural_form = Self::parse_plural_form_or_default(&config.plural_form)?;
        let ordinal_form = Self::parse_plural_form_or_default(&config.ordinal_form)?;

        Ok(LocalizedData {
            localize_dict: Self::load_dict_static(&path, config.localize_dict.as_ref()).unwrap_or_default(),
            hashed_dict: Self::load_dict_static(&path, config.hashed_dict.as_ref()).unwrap_or_default(),
            text_data_dict: Self::load_dict_static(&path, config.text_data_dict.as_ref()).unwrap_or_default(),
            character_system_text_dict: Self::load_dict_static(&path, config.character_system_text_dict.as_ref()).unwrap_or_default(),
            race_jikkyo_comment_dict: Self::load_dict_static(&path, config.race_jikkyo_comment_dict.as_ref()).unwrap_or_default(),
            race_jikkyo_message_dict: Self::load_dict_static(&path, config.race_jikkyo_message_dict.as_ref()).unwrap_or_default(),
            assets_path: path.as_ref()
                .map(|p| config.assets_dir.as_ref()
                    .map(|dir| p.join(dir))
                )
                .unwrap_or_default(),

            plural_form,
            ordinal_form,

            config,
            path
        })
    }

    fn load_dict_static_ex<T: DeserializeOwned, P: AsRef<Path>>(ld_path_opt: &Option<PathBuf>, rel_path_opt: Option<P>, silent_fs_error: bool) -> Option<T> {
        let Some(ld_path) = ld_path_opt else {
            return None;
        };
        let Some(rel_path) = rel_path_opt else {
            return None;
        };

        let path = ld_path.join(rel_path);
        let json = match fs::read_to_string(&path) {
            Ok(v) => v,
            Err(e) => {
                if !silent_fs_error {
                    error!("Failed to read '{}': {}", path.display(), e);
                }
                return None;
            }
        };

        let dict = match serde_json::from_str::<T>(&json) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to parse '{}': {}", path.display(), e);
                return None;
            }
        };

        Some(dict)
    }

    fn load_dict_static<T: DeserializeOwned, P: AsRef<Path>>(ld_path_opt: &Option<PathBuf>, rel_path_opt: Option<P>) -> Option<T> {
        Self::load_dict_static_ex(ld_path_opt, rel_path_opt, false)
    }

    pub fn load_dict<T: DeserializeOwned, P: AsRef<Path>>(&self, rel_path_opt: Option<P>) -> Option<T> {
        Self::load_dict_static(&self.path, rel_path_opt)
    }

    pub fn load_assets_dict<T: DeserializeOwned, P: AsRef<Path>>(&self, rel_path_opt: Option<P>) -> Option<T> {
        Self::load_dict_static_ex(&self.assets_path, rel_path_opt, true)
    }

    fn parse_plural_form_or_default(opt: &Option<String>) -> Result<plurals::Resolver, Error> {
        if let Some(plural_form) = opt {
            Ok(plurals::Resolver::Expr(plurals::Ast::parse(plural_form)?))
        }
        else {
            Ok(plurals::Resolver::Function(|_| 0))
        }
    }

    pub fn get_assets_path<P: AsRef<Path>>(&self, rel_path: P) -> Option<PathBuf> {
        self.assets_path.as_ref().map(|p| p.join(rel_path))
    }

    pub fn get_data_path<P: AsRef<Path>>(&self, rel_path: P) -> Option<PathBuf> {
        self.path.as_ref().map(|p| p.join(rel_path))
    }

    pub fn load_asset_metadata<P: AsRef<Path>>(&self, rel_path: P) -> AssetMetadata {
        let mut path = rel_path.as_ref().to_owned();
        path.set_extension("json");
        self.load_assets_dict(Some(path)).unwrap_or_else(|| AssetInfo::<()>::default()).metadata()
    }

    pub fn load_asset_info<P: AsRef<Path>, T: DeserializeOwned>(&self, rel_path: P) -> AssetInfo<T> {
        let mut path = rel_path.as_ref().to_owned();
        path.set_extension("json");
        self.load_assets_dict(Some(path)).unwrap_or_else(|| AssetInfo::default())
    }
}

#[derive(Deserialize, Clone)]
pub struct LocalizedDataConfig {
    pub localize_dict: Option<String>,
    pub hashed_dict: Option<String>,
    pub text_data_dict: Option<String>,
    pub character_system_text_dict: Option<String>,
    pub race_jikkyo_comment_dict: Option<String>,
    pub race_jikkyo_message_dict: Option<String>,
    pub assets_dir: Option<String>,
    #[serde(default)]
    pub extra_asset_bundle: OsOption<String>,
    pub replacement_font_name: Option<String>,

    pub plural_form: Option<String>,
    pub ordinal_form: Option<String>,
    #[serde(default)]
    pub ordinal_types: Vec<String>,
    #[serde(default)]
    pub months: Vec<String>,
    pub month_text_format: Option<String>,

    #[serde(default)]
    pub use_text_wrapper: bool,
    // Predefined line widths are counts of cjk characters.
    // 1 cjk char = 2 columns, so setting this value to 2 replicates the default behaviour.
    pub line_width_multiplier: Option<f32>,

    #[serde(default)]
    pub auto_adjust_story_clip_length: bool,
    pub story_line_count_offset: Option<i32>,
    pub text_frame_line_spacing_multiplier: Option<f32>,
    pub text_frame_font_size_multiplier: Option<f32>,
    pub skill_list_item_desc_font_size_multiplier: Option<f32>,
    #[serde(default)]
    pub text_common_allow_overflow: bool,
    #[serde(default)]
    pub now_loading_comic_title_ellipsis: bool,

    #[serde(default)]
    pub remove_ruby: bool,
    pub character_note_top_gallery_button: Option<UITextConfig>,
    pub character_note_top_talk_gallery_button: Option<UITextConfig>,

    pub news_url: Option<String>,

    // RESERVED
    #[serde(default)]
    pub _debug: i32
}

#[derive(Deserialize, Clone)]
pub struct UITextConfig {
    pub text: Option<String>,
    pub font_size: Option<i32>,
    pub line_spacing: Option<f32>
}

impl Default for LocalizedDataConfig {
    fn default() -> Self {
        default_serde_instance().expect("default instance")
    }
}

#[derive(Deserialize)]
pub struct AssetInfo<T> {
    #[cfg(target_os = "android")]
    #[serde(default)]
    android: AssetMetadata,

    #[cfg(target_os = "windows")]
    #[serde(default)]
    windows: AssetMetadata,

    pub data: Option<T>
}

// Can't derive(Default), see rust-lang/rust#26925
impl<T> Default for AssetInfo<T> {
    fn default() -> Self {
        Self {
            #[cfg(target_os = "android")]
            android: Default::default(),

            #[cfg(target_os = "windows")]
            windows: Default::default(),

            data: None
        }
    }
}

impl<T> AssetInfo<T> {
    pub fn metadata(self) -> AssetMetadata {
        #[cfg(target_os = "android")]
        return self.android;

        #[cfg(target_os = "windows")]
        return self.windows;
    }

    pub fn metadata_ref(&self) -> &AssetMetadata {
        #[cfg(target_os = "android")]
        return &self.android;

        #[cfg(target_os = "windows")]
        return &self.windows;
    }
}

#[derive(Deserialize, Clone, Default)]
pub struct AssetMetadata {
    pub bundle_name: Option<String>
}