use std::{fs, os::raw::c_void, path::{Path, PathBuf}, process, sync::{atomic::{self, AtomicBool, AtomicI32}, Arc}};
use arc_swap::ArcSwap;
use fnv::FnvHashMap;
use once_cell::sync::OnceCell;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{gui_impl, hachimi_impl, il2cpp};

use super::{game::Game, plurals, template, template_filters, tl_repo, utils, Error, Interceptor};

pub struct Hachimi {
    // Hooking stuff
    pub interceptor: Interceptor,
    pub hooking_finished: AtomicBool,

    // Localized data
    pub localized_data: ArcSwap<LocalizedData>,
    pub tl_updater: Arc<tl_repo::Updater>,

    // Shared properties
    pub game: Game,
    pub config: ArcSwap<Config>,
    pub template_parser: template::Parser,

    /// -1 = default
    pub target_fps: AtomicI32
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
                    error!("Init failed: {}", e);
                    return false;
                }
            };

            super::log::init(&instance);

            info!("Hachimi v{}", env!("CARGO_PKG_VERSION"));
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
            interceptor: Interceptor::default(),
            hooking_finished: AtomicBool::new(false),

            localized_data: ArcSwap::new(Arc::new(LocalizedData::new(&config, &game.data_dir)?)),
            tl_updater: Arc::default(),

            game,
            template_parser: template::Parser::new(&template_filters::LIST),

            target_fps: AtomicI32::new(config.target_fps.unwrap_or(-1)),

            config: ArcSwap::new(Arc::new(config))
        })
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

    pub fn reload_localized_data(&self) {
        if self.tl_updater.progress().is_some() {
            warn!("Update in progress, not reloading localized data");
            return;
        }
        let new_data = match LocalizedData::new(&self.config.load(), &self.game.data_dir) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to reload localized data: {}", e);
                return;
            }
        };
        self.localized_data.store(Arc::new(new_data));
    }

    pub fn find_symbol_by_name(&self, module: &str, symbol: &str) -> Result<*mut c_void, Error> {
        dobby_rs::resolve_symbol(module, symbol).ok_or(Error::SymbolNotFound(module.to_owned(), symbol.to_owned()))
    }

    pub fn on_dlopen(&self, filename: &str, handle: usize) -> bool {
        // return false, wait for il2cpp hook to finish and call unhook once (see below)
        if self.hooking_finished.load(atomic::Ordering::Relaxed) { return false; }

        if hachimi_impl::is_il2cpp_lib(filename) {
            info!("Got il2cpp handle");
            il2cpp::symbols::set_handle(handle);
            false
        }
        else if hachimi_impl::is_criware_lib(filename) {
            // Dobby calls dlopen while it's hooking, set this flag first
            self.hooking_finished.store(true, atomic::Ordering::Relaxed);

            info!("GameAssembly finished loading");
            il2cpp::symbols::init();
            il2cpp::hook::init();
            if !self.config.load().disable_gui {
                gui_impl::init();
            }
            true
        }
        else {
            false
        }
    }

    pub fn get_data_path<P: AsRef<Path>>(&self, rel_path: P) -> PathBuf {
        self.game.data_dir.join(rel_path)
    }
}

fn default_serde_instance<'a, T: Deserialize<'a>>() -> Option<T> {
    let empty_data = std::iter::empty::<((), ())>();
    let empty_deserializer = serde::de::value::MapDeserializer::<_, serde::de::value::Error>::new(empty_data);
    T::deserialize(empty_deserializer).ok()
}

#[derive(Deserialize, Clone, Serialize)]
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
    pub disable_auto_update_check: bool
}

impl Config {
    fn default_open_browser_url() -> String { "https://www.google.com/".to_owned() }
    fn default_virtual_res_mult() -> f32 { 1.0 }
}

impl Default for Config {
    fn default() -> Self {
        default_serde_instance().expect("default instance")
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
        let path: Option<PathBuf>;
        let config: LocalizedDataConfig = if let Some(ld_dir) = &config.localized_data_dir {
            let ld_path = Path::new(data_dir).join(ld_dir);
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
        if let Some(assets_path) = &self.assets_path {
            Some(assets_path.join(rel_path))
        }
        else {
            None
        }
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

    pub plural_form: Option<String>,
    pub ordinal_form: Option<String>,
    #[serde(default)] pub ordinal_types: Vec<String>,
    pub text_wrapper: Option<i32>,
    pub line_width_multiplier: Option<f32>,

    // UNIMPLEMENTED
    #[serde(default)]
    pub story_adjust_length: bool,
    #[serde(default = "LocalizedDataConfig::default_story_cps")]
    pub story_cps: i32,

    // RESERVED
    #[serde(default)]
    pub _debug: i32
}

impl LocalizedDataConfig {
    fn default_story_cps() -> i32 { 28 }
}

impl Default for LocalizedDataConfig {
    fn default() -> Self {
        default_serde_instance().expect("default instance")
    }
}