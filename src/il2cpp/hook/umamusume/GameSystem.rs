use crate::{core::Hachimi, il2cpp::{symbols::{get_method_addr, IEnumerator, MonoSingleton, MoveNextFn}, types::*}};

static mut CLASS: *mut Il2CppClass = 0 as _;
pub fn class() -> *mut Il2CppClass {
    unsafe { CLASS }
}

pub fn instance() -> *mut Il2CppObject {
    let Some(singleton) = MonoSingleton::new(class()) else {
        return 0 as _;
    };
    singleton.instance()
}

static mut SOFTWARERESET_ADDR: usize = 0;
impl_addr_wrapper_fn!(SoftwareReset, SOFTWARERESET_ADDR, (), this: *mut Il2CppObject);

pub fn on_game_initialized() {
    #[cfg(target_os = "windows")]
    super::UIManager::apply_ui_scale();
}

extern "C" fn InitializeGame_MoveNext(enumerator: *mut Il2CppObject) -> bool {
    let moved = get_orig_fn!(InitializeGame_MoveNext, MoveNextFn)(enumerator);
    if !moved {
        // Game has finished initializing
        on_game_initialized();
    }
    moved
}

type InitializeGameFn = extern "C" fn(this: *mut Il2CppObject) -> IEnumerator;
extern "C" fn InitializeGame(this: *mut Il2CppObject) -> IEnumerator {
    let enumerator = get_orig_fn!(InitializeGame, InitializeGameFn)(this);
    if Hachimi::instance().config.load().ui_scale == 1.0 { return enumerator; }

    if let Err(e) = enumerator.hook_move_next(InitializeGame_MoveNext) {
        error!("Failed to hook InitializeGame enumerator: {}", e);
    }

    enumerator
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, GameSystem);

    let InitializeGame_addr = get_method_addr(GameSystem, c"InitializeGame", 0);

    new_hook!(InitializeGame_addr, InitializeGame);

    unsafe {
        CLASS = GameSystem;
        SOFTWARERESET_ADDR = get_method_addr(GameSystem, c"SoftwareReset", 0);
    }
}