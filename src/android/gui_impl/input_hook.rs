#![allow(non_snake_case)]

use std::sync::atomic::{AtomicBool, Ordering};

use jni::{objects::{JMap, JObject}, sys::{jboolean, jint, JNI_TRUE}, JNIEnv};

use crate::core::{Error, Gui, Hachimi};

const ACTION_DOWN: jint = 0;
const ACTION_UP: jint = 1;
const ACTION_MOVE: jint = 2;
const ACTION_POINTER_DOWN: jint = 5;
const ACTION_POINTER_UP: jint = 6;
const ACTION_MASK: jint = 0xff;
const ACTION_POINTER_INDEX_MASK: jint = 0xff00;
const ACTION_POINTER_INDEX_SHIFT: jint = 8;

const KEYCODE_VOLUME_UP: jint = 24;
const KEYCODE_VOLUME_DOWN: jint = 25;

static VOLUME_UP_PRESSED: AtomicBool = AtomicBool::new(false);
static VOLUME_DOWN_PRESSED: AtomicBool = AtomicBool::new(false);

// static KEYBOARD_SHOWN: AtomicBool = AtomicBool::new(false);

type NativeInjectEventFn = extern "C" fn(env: JNIEnv, obj: JObject, input_event: JObject) -> jboolean;
extern "C" fn nativeInjectEvent(mut env: JNIEnv, obj: JObject, input_event: JObject) -> jboolean {
    let motion_event_class = env.find_class("android/view/MotionEvent").unwrap();
    let key_event_class = env.find_class("android/view/KeyEvent").unwrap();

    if env.is_instance_of(&input_event, &motion_event_class).unwrap() {
        // early return using atomic check to prevent mutex lock overhead
        if !Gui::is_consuming_input_atomic() {
            return get_orig_fn!(nativeInjectEvent, NativeInjectEventFn)(env, obj, input_event);
        }

        let Some(mut gui) = Gui::instance().map(|m| m.lock().unwrap()) else {
            return get_orig_fn!(nativeInjectEvent, NativeInjectEventFn)(env, obj, input_event);
        };

        let get_action_res = env.call_method(&input_event, "getAction", "()I", &[]).unwrap();
        let action = get_action_res.i().unwrap();
        let action_masked = action & ACTION_MASK;
        let pointer_index = (action & ACTION_POINTER_INDEX_MASK) >> ACTION_POINTER_INDEX_SHIFT;

        if pointer_index != 0 {
            return JNI_TRUE;
        }

        // borrowing egui's touch phase enum
        let phase = match action_masked {
            ACTION_DOWN | ACTION_POINTER_DOWN => egui::TouchPhase::Start,
            ACTION_MOVE => egui::TouchPhase::Move,
            ACTION_UP | ACTION_POINTER_UP => egui::TouchPhase::End,
            _ => return JNI_TRUE
        };

        // dumb and simple, no multi touch
        let real_x = env.call_method(&input_event, "getX", "()F", &[]).unwrap().f().unwrap();
        let real_y = env.call_method(&input_event, "getY", "()F", &[]).unwrap().f().unwrap();

        // SAFETY: view doesn't live past the lifetime of this function
        let view = get_view(unsafe { env.unsafe_clone() });
        let view_width = env.call_method(&view, "getWidth", "()I", &[]).unwrap().i().unwrap();
        let view_height = env.call_method(&view, "getHeight", "()I", &[]).unwrap().i().unwrap();
        let view_main_axis_size = if view_width < view_height { view_width } else { view_height };

        let ppp = gui.context.zoom_factor() * (view_main_axis_size as f32 / gui.prev_main_axis_size as f32);
        let x = real_x as f32 / ppp;
        let y = real_y as f32 / ppp;
        let pos = egui::Pos2 { x, y };

        match phase {
            egui::TouchPhase::Start => {
                gui.input.events.push(egui::Event::PointerMoved(pos));
                gui.input.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::default()
                });
            },
            egui::TouchPhase::Move => {
                gui.input.events.push(egui::Event::PointerMoved(pos));
            },
            egui::TouchPhase::End | egui::TouchPhase::Cancel => {
                gui.input.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::default()
                });
                gui.input.events.push(egui::Event::PointerGone);
            }
        }

        /* TODO
        let keyboard_shown = KEYBOARD_SHOWN.load(Ordering::Relaxed);
        let wants_keyboard_input = gui.context.wants_keyboard_input();
        if wants_keyboard_input && !keyboard_shown {
            show_soft_input(&mut env, true);
            KEYBOARD_SHOWN.store(true, Ordering::Relaxed);
        }
        else if !wants_keyboard_input && keyboard_shown {
            show_soft_input(&mut env, false);
            KEYBOARD_SHOWN.store(false, Ordering::Relaxed);
        }
        */

        return JNI_TRUE;
    }
    else if env.is_instance_of(&input_event, &key_event_class).unwrap() {
        let get_action_res = env.call_method(&input_event, "getAction", "()I", &[]).unwrap();
        let action = get_action_res.i().unwrap();

        let get_key_code_res = env.call_method(&input_event, "getKeyCode", "()I", &[]).unwrap();
        let key_code = get_key_code_res.i().unwrap();

        let pressed = action == ACTION_DOWN;
        let other_atomic = match key_code {
            KEYCODE_VOLUME_UP => {
                VOLUME_UP_PRESSED.store(pressed, Ordering::Relaxed);
                &VOLUME_DOWN_PRESSED
            }
            KEYCODE_VOLUME_DOWN => {
                VOLUME_DOWN_PRESSED.store(pressed, Ordering::Relaxed);
                &VOLUME_UP_PRESSED
            }
            _ => return get_orig_fn!(nativeInjectEvent, NativeInjectEventFn)(env, obj, input_event)
        };

        if pressed && other_atomic.load(Ordering::Relaxed) {
            let Some(mut gui) = Gui::instance().map(|m| m.lock().unwrap()) else {
                return get_orig_fn!(nativeInjectEvent, NativeInjectEventFn)(env, obj, input_event);
            };
            gui.toggle_menu();
        }
    }

    get_orig_fn!(nativeInjectEvent, NativeInjectEventFn)(env, obj, input_event)
}

fn get_view(mut env: JNIEnv) -> JObject<'_> {
    let activity_thread_class = env.find_class("android/app/ActivityThread").unwrap();
    let activity_thread = env
        .call_static_method(
            activity_thread_class,
            "currentActivityThread",
            "()Landroid/app/ActivityThread;",
            &[]
        )
        .unwrap()
        .l()
        .unwrap();
    let activities = env
        .get_field(activity_thread, "mActivities", "Landroid/util/ArrayMap;")
        .unwrap()
        .l()
        .unwrap();
    let activities_map = JMap::from_env(&mut env, &activities).unwrap();

    // Get the first activity in the map
    let (_, activity_record) = activities_map.iter(&mut env).unwrap().next(&mut env).unwrap().unwrap();
    let activity = env
        .get_field(activity_record, "activity", "Landroid/app/Activity;")
        .unwrap()
        .l()
        .unwrap();

    let jni_window = env
        .call_method(activity, "getWindow", "()Landroid/view/Window;", &[])
        .unwrap()
        .l()
        .unwrap();

    env
        .call_method(jni_window, "getDecorView", "()Landroid/view/View;", &[])
        .unwrap()
        .l()
        .unwrap()
}

/* TODO: incomplete code + currently not functional, view seems to be in touch mode
fn show_soft_input(env: &mut JNIEnv, show: bool) {
    let ctx_class = env.find_class("android/content/Context").unwrap();
    let ime = env
        .get_static_field(ctx_class, "INPUT_METHOD_SERVICE", "Ljava/lang/String;")
        .unwrap();

    let ime_manager = env
        .call_method(
            &activity,
            "getSystemService",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[ime.borrow()],
        )
        .unwrap()
        .l()
        .unwrap();

    if show {
        debug!("showing");
        let result = env.call_method(
            ime_manager,
            "showSoftInput",
            "(Landroid/view/View;I)Z",
            &[(&view).into(), 2i32.into()],
        )
        .unwrap()
        .z()
        .unwrap();
        debug!("res: {}", result);
    }
    else {
        debug!("hiding");
        let window_token = env
            .call_method(view, "getWindowToken", "()Landroid/os/IBinder;", &[])
            .unwrap();
        env.call_method(
            ime_manager,
            "hideSoftInputFromWindow",
            "(Landroid/os/IBinder;I)Z",
            &[window_token.borrow(), 0i32.into()],
        )
        .unwrap();
    }
}
*/

pub static mut NATIVE_INJECT_EVENT_ADDR: usize = 0;

fn init_internal() -> Result<(), Error> {
    let native_inject_event_addr = unsafe { NATIVE_INJECT_EVENT_ADDR };
    if native_inject_event_addr != 0 {
        info!("Hooking nativeInjectEvent");
        Hachimi::instance().interceptor.hook(unsafe { NATIVE_INJECT_EVENT_ADDR }, nativeInjectEvent as usize)?;
    }
    else {
        error!("native_inject_event_addr is null");
    }

    Ok(())
}

pub fn init() {
    init_internal().unwrap_or_else(|e| {
        error!("Init failed: {}", e);
    });
}