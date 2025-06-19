use std::os::raw::c_long;

use jni::{objects::JString, JNIEnv};
use once_cell::unsync::OnceCell;

use crate::{android::{game_impl, hook, zygisk::{internal::{api_table, module_abi}, AppSpecializeArgs, ServerSpecializeArgs}}, core::{game::Region, Hachimi}};

const ZYGISK_API_VERSION: c_long = 4;

pub struct Module {
    env: *mut jni::sys::JNIEnv,
    is_game: bool
}

impl Module {
    fn new(env: *mut jni::sys::JNIEnv) -> Self {
        Self {
            env,
            is_game: false
        }
    }
}

static mut PACKAGE_NAME: OnceCell<String> = OnceCell::new();
pub fn get_package_name() -> Option<&'static String> {
    unsafe { PACKAGE_NAME.get() }
}

unsafe extern "C" fn pre_app_specialize(this: *mut Module, args: *mut AppSpecializeArgs) {
    let mut env = unsafe { JNIEnv::from_raw((*this).env).unwrap() };
    let jstr = JString::from_raw(*(*args).nice_name);
    let java_str = env.get_string(&jstr).unwrap();
    let package_name = java_str.to_string_lossy();
    _ = PACKAGE_NAME.set(package_name.to_string());

    (*this).is_game = match game_impl::get_region(&package_name) {
        Region::Japan => true,
        _ => false
    };
}

unsafe extern "C" fn post_app_specialize(this: *mut Module, _args: *const AppSpecializeArgs) {
    if (*this).is_game {
        if !Hachimi::init() {
            return;
        }
        hook::init((*this).env);
    }
}

unsafe extern "C" fn pre_server_specialize(_this: *mut Module, _args: *mut ServerSpecializeArgs) {

}

unsafe extern "C" fn post_server_specialize(_this: *mut Module, _args: *const ServerSpecializeArgs) {

}

static mut MODULE: OnceCell<Module> = OnceCell::new();
static mut ABI: OnceCell<module_abi<Module>> = OnceCell::new();

#[no_mangle]
pub unsafe extern "C" fn zygisk_module_entry(api: *mut api_table<Module>, env: *mut jni::sys::JNIEnv) {
    let module = Module::new(env);
    if MODULE.set(module).is_err() { return; }

    let abi = module_abi {
        api_version: ZYGISK_API_VERSION,
        impl_: MODULE.get_mut().unwrap(),
        preAppSpecialize: Some(pre_app_specialize),
        postAppSpecialize: Some(post_app_specialize),
        preServerSpecialize: Some(pre_server_specialize),
        postServerSpecialize: Some(post_server_specialize)
    };
    if ABI.set(abi).is_err() { return; }

    (*api).registerModule.unwrap()(api, ABI.get_mut().unwrap());
}

#[no_mangle]
pub unsafe extern "C" fn zygisk_companion_entry(_arg1: ::std::os::raw::c_int) {

}