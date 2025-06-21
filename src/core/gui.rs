use std::{borrow::Cow, ops::RangeInclusive, sync::{atomic::{self, AtomicBool}, Arc, Mutex}, thread, time::Instant};

use fnv::FnvHashSet;
use once_cell::sync::OnceCell;
use rust_i18n::t;

use crate::il2cpp::{
    hook::{
        umamusume::{CySpringController::SpringUpdateMode, GameSystem, GraphicSettings::GraphicsQuality, Localize},
        UnityEngine_CoreModule::Application
    },
    symbols::Thread
};

#[cfg(not(target_os = "windows"))]
use crate::il2cpp::hook::umamusume::WebViewManager;

#[cfg(target_os = "windows")]
use crate::il2cpp::hook::UnityEngine_CoreModule::QualitySettings;

use super::{hachimi::{self, Language}, http::AsyncRequest, tl_repo::{self, RepoInfo}, utils, Hachimi};

macro_rules! add_font {
    ($fonts:expr, $family_fonts:expr, $filename:literal) => {
        $fonts.font_data.insert(
            $filename.to_owned(),
            egui::FontData::from_static(include_bytes!(concat!("../../assets/fonts/", $filename)))
        );
        $family_fonts.push($filename.to_owned());
    };
}

type BoxedWindow = Box<dyn Window + Send + Sync>;
pub struct Gui {
    pub context: egui::Context,
    pub input: egui::RawInput,
    pub start_time: Instant,
    pub prev_main_axis_size: i32,
    last_fps_update: Instant,
    tmp_frame_count: u32,
    fps_text: String,

    show_menu: bool,

    splash_visible: bool,
    splash_tween: TweenInOutWithDelay,
    splash_sub_str: String,

    menu_visible: bool,
    menu_anim_time: Option<Instant>,
    menu_fps_value: i32,

    #[cfg(target_os = "windows")]
    menu_vsync_value: i32,

    pub update_progress_visible: bool,

    notifications: Vec<Notification>,
    windows: Vec<BoxedWindow>
}

const PIXELS_PER_POINT_RATIO: f32 = 3.0/1080.0;
const BACKGROUND_COLOR: egui::Color32 = egui::Color32::from_rgba_premultiplied(27, 27, 27, 220);
const TEXT_COLOR: egui::Color32 = egui::Color32::from_gray(170);

static INSTANCE: OnceCell<Mutex<Gui>> = OnceCell::new();
static IS_CONSUMING_INPUT: AtomicBool = AtomicBool::new(false);
static mut DISABLED_GAME_UIS: once_cell::unsync::Lazy<FnvHashSet<*mut crate::il2cpp::types::Il2CppObject>> =
    once_cell::unsync::Lazy::new(|| FnvHashSet::default());

impl Gui {
    // Call this from the render thread!
    pub fn instance_or_init(open_key_id: &str) -> &Mutex<Gui> {
        if let Some(instance) = INSTANCE.get() {
            return instance;
        }

        let hachimi = Hachimi::instance();
        let config = hachimi.config.load();

        let context = egui::Context::default();
        egui_extras::install_image_loaders(&context);

        context.set_fonts(Self::get_font_definitions());

        let mut style = egui::Style::default();
        style.spacing.button_padding = egui::Vec2::new(8.0, 5.0);
        style.interaction.selectable_labels = false;
        context.set_style(style);

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = BACKGROUND_COLOR;
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_COLOR);
        context.set_visuals(visuals);

        let mut fps_value = hachimi.target_fps.load(atomic::Ordering::Relaxed);
        if fps_value == -1 {
            fps_value = 30;
        }

        let mut windows: Vec<BoxedWindow> = Vec::new();
        if !config.skip_first_time_setup {
            windows.push(Box::new(FirstTimeSetupWindow::new()));
        }

        let now = Instant::now();
        let instance = Gui {
            context,
            input: egui::RawInput::default(),
            start_time: now,
            prev_main_axis_size: 1,
            last_fps_update: now,
            tmp_frame_count: 0,
            fps_text: "FPS: 0".to_string(),

            show_menu: false,

            splash_visible: true,
            splash_tween: TweenInOutWithDelay::new(0.8, 3.0, Easing::OutQuad),
            splash_sub_str: t!("splash_sub", open_key_str = t!(open_key_id)).into_owned(),

            menu_visible: false,
            menu_anim_time: None,
            menu_fps_value: fps_value,

            #[cfg(target_os = "windows")]
            menu_vsync_value: hachimi.vsync_count.load(atomic::Ordering::Relaxed),

            update_progress_visible: false,

            notifications: Vec::new(),
            windows
        };
        unsafe {
            INSTANCE.set(Mutex::new(instance)).unwrap_unchecked();

            // Doing auto update check here to ensure that the updater can access the gui
            hachimi.run_auto_update_check();

            INSTANCE.get().unwrap_unchecked()
        }
    }

    pub fn instance() -> Option<&'static Mutex<Gui>> {
        INSTANCE.get()
    }

    fn get_font_definitions() -> egui::FontDefinitions {
        let mut fonts = egui::FontDefinitions::default();
        let proportional_fonts = fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap();

        add_font!(fonts, proportional_fonts, "AlibabaPuHuiTi-3-45-Light.otf");
        add_font!(fonts, proportional_fonts, "NotoSans-Light.ttf");
        add_font!(fonts, proportional_fonts, "FontAwesome.otf");

        fonts
    }

    pub fn set_screen_size(&mut self, width: i32, height: i32) {
        let main_axis_size = if width < height { width } else { height };
        let pixels_per_point = main_axis_size as f32 * PIXELS_PER_POINT_RATIO;
        self.context.set_pixels_per_point(pixels_per_point);

        self.input.screen_rect = Some(egui::Rect {
            min: egui::Pos2::default(),
            max: egui::Pos2::new(
                width as f32 / self.context.pixels_per_point(),
                height as f32 / self.context.pixels_per_point()
            )
        });

        self.prev_main_axis_size = main_axis_size;
    }

    fn take_input(&mut self) -> egui::RawInput {
        self.input.time = Some(self.start_time.elapsed().as_secs_f64());
        self.input.take()
    }

    fn update_fps(&mut self) {
        let delta = self.last_fps_update.elapsed().as_secs_f64();
        if delta > 0.5 {
            let fps = (self.tmp_frame_count as f64 * (0.5 / delta) * 2.0).round();
            self.fps_text = t!("menu.fps_text", fps = fps).into_owned();
            self.tmp_frame_count = 1;
            self.last_fps_update = Instant::now();
        }
        else {
            self.tmp_frame_count += 1;
        }
    }

    pub fn run(&mut self) -> egui::FullOutput {
        self.update_fps();
        let input = self.take_input();

        self.context.begin_frame(input);
        
        if self.menu_visible { self.run_menu(); }
        if self.update_progress_visible { self.run_update_progress(); }

        self.run_windows();
        self.run_notifications();

        if self.splash_visible { self.run_splash(); }

        // Store this as an atomic value so the input thread can check it without locking the gui
        IS_CONSUMING_INPUT.store(self.is_consuming_input(), atomic::Ordering::Relaxed);

        self.context.end_frame()
    }

    const ICON_IMAGE: egui::ImageSource<'static> = egui::include_image!("../../assets/icon.png");
    fn icon<'a>() -> egui::Image<'a> {
        egui::Image::new(Self::ICON_IMAGE)
        .fit_to_exact_size(egui::Vec2::new(24.0, 24.0))
    }

    fn icon_2x<'a>() -> egui::Image<'a> {
        egui::Image::new(Self::ICON_IMAGE)
        .fit_to_exact_size(egui::Vec2::new(48.0, 48.0))
    }

    fn run_splash(&mut self) {
        let ctx = &self.context;

        let id = egui::Id::from("splash");
        let Some(tween_val) = self.splash_tween.run(ctx, id.with("tween")) else {
            self.splash_visible = false;
            return;
        };

        egui::Area::new(id)
        .fixed_pos(egui::Pos2 {
            x: -250.0 * (1.0 - tween_val),
            y: 16.0
        })
        .show(ctx, |ui| {
            egui::Frame::none()
            .fill(BACKGROUND_COLOR)
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add(Self::icon());
                    ui.heading("Hachimi");
                    ui.label(env!("HACHIMI_DISPLAY_VERSION"));
                });
                ui.label(&self.splash_sub_str);
            });
        });
    }

    fn run_menu(&mut self) {
        let ctx = &self.context;
        let hachimi = Hachimi::instance();
        let localized_data = hachimi.localized_data.load();
        let localize_dict_count = localized_data.localize_dict.len().to_string();
        let hashed_dict_count = localized_data.hashed_dict.len().to_string();

        let mut show_notification: Option<Cow<'_, str>> = None;
        let mut show_window: Option<BoxedWindow> = None;
        egui::SidePanel::left("hachimi_menu").show_animated(ctx, self.show_menu, |ui| {
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::TOP), |ui| {
                ui.horizontal(|ui| {
                    ui.add(Self::icon());
                    ui.heading(t!("hachimi"));
                    if ui.button(" \u{f29c} ").clicked() {
                        show_window = Some(Box::new(AboutWindow::new()));
                    }
                });
                ui.label(env!("HACHIMI_DISPLAY_VERSION"));
                if ui.button(t!("menu.close_menu")).clicked() {
                    self.show_menu = false;
                    self.menu_anim_time = None;
                }
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading(t!("menu.stats_heading"));
                    ui.label(&self.fps_text);
                    ui.label(t!("menu.localize_dict_entries", count = localize_dict_count));
                    ui.label(t!("menu.hashed_dict_entries", count = hashed_dict_count));
                    ui.separator();

                    ui.heading(t!("menu.config_heading"));
                    if ui.button(t!("menu.open_config_editor")).clicked() {
                        show_window = Some(Box::new(ConfigEditor::new()));
                    }
                    if ui.button(t!("menu.reload_config")).clicked() {
                        hachimi.reload_config();
                        show_notification = Some(t!("notification.config_reloaded"));
                    }
                    if ui.button(t!("menu.open_first_time_setup")).clicked() {
                        show_window = Some(Box::new(FirstTimeSetupWindow::new()));
                    }
                    ui.separator();

                    ui.heading(t!("menu.graphics_heading"));
                    ui.horizontal(|ui| {
                        ui.label(t!("menu.fps_label"));
                        let res = ui.add(egui::Slider::new(&mut self.menu_fps_value, 30..=240));
                        if res.lost_focus() || res.drag_stopped() {
                            hachimi.target_fps.store(self.menu_fps_value, atomic::Ordering::Relaxed);
                            Thread::main_thread().schedule(|| {
                                // doesnt matter which value's used here, hook will override it
                                Application::set_targetFrameRate(30);
                            });
                        }
                    });
                    #[cfg(target_os = "windows")]
                    {
                        use crate::windows::{utils::set_window_topmost, wnd_hook};

                        ui.horizontal(|ui| {
                            let prev_value = self.menu_vsync_value;

                            ui.label(t!("menu.vsync_label"));
                            Self::run_vsync_combo(ui, &mut self.menu_vsync_value);

                            if prev_value != self.menu_vsync_value {
                                hachimi.vsync_count.store(self.menu_vsync_value, atomic::Ordering::Relaxed);
                                Thread::main_thread().schedule(|| {
                                    QualitySettings::set_vSyncCount(1);
                                });
                            }
                        });
                        ui.horizontal(|ui| {
                            let mut value = hachimi.window_always_on_top.load(atomic::Ordering::Relaxed);

                            ui.label(t!("menu.stay_on_top"));
                            if ui.checkbox(&mut value, "").changed() {
                                hachimi.window_always_on_top.store(value, atomic::Ordering::Relaxed);
                                Thread::main_thread().schedule(|| {
                                    let topmost = Hachimi::instance().window_always_on_top.load(atomic::Ordering::Relaxed);
                                    unsafe { _ = set_window_topmost(wnd_hook::get_target_hwnd(), topmost); }
                                });
                            }
                        });
                    }
                    ui.separator();

                    ui.heading(t!("menu.translation_heading"));
                    if ui.button(t!("menu.reload_localized_data")).clicked() {
                        hachimi.load_localized_data();
                        show_notification = Some(t!("notification.localized_data_reloaded"));
                    }
                    if ui.button(t!("menu.check_for_updates")).clicked() {
                        hachimi.tl_updater.clone().check_for_updates(false);
                    }
                    if ui.button(t!("menu.check_for_updates_pedantic")).clicked() {
                        hachimi.tl_updater.clone().check_for_updates(true);
                    }
                    if hachimi.config.load().translator_mode {
                        if ui.button(t!("menu.dump_localize_dict")).clicked() {
                            Thread::main_thread().schedule(|| {
                                let data = Localize::dump_strings();
                                let dict_path = Hachimi::instance().get_data_path("localize_dump.json");
                                let mut gui = Gui::instance().unwrap().lock().unwrap();
                                if let Err(e) = utils::write_json_file(&data, dict_path) {
                                    gui.show_notification(&e.to_string())
                                }
                                else {
                                    gui.show_notification(&t!("notification.saved_localize_dump"))
                                }
                            })
                        }
                    }
                    ui.separator();

                    ui.heading(t!("menu.danger_zone_heading"));
                    ui.label(t!("menu.danger_zone_warning"));
                    if ui.button(t!("menu.soft_restart")).clicked() {
                        show_window = Some(Box::new(SimpleYesNoDialog::new(&t!("confirm_dialog_title"), &t!("soft_restart_confirm_content"), |ok| {
                            if !ok { return; }
                            Thread::main_thread().schedule(|| {
                                GameSystem::SoftwareReset(GameSystem::instance());
                            });
                        })));
                    }
                    #[cfg(not(target_os = "windows"))]
                    if ui.button(t!("menu.open_in_game_browser")).clicked() {
                        show_window = Some(Box::new(SimpleYesNoDialog::new(&t!("confirm_dialog_title"), &t!("in_game_browser_confirm_content"), |ok| {
                            if !ok { return; }
                            Thread::main_thread().schedule(|| {
                                WebViewManager::quick_open(&t!("browser_dialog_title"), &Hachimi::instance().config.load().open_browser_url);
                            });
                        })));
                    }
                    if ui.button(t!("menu.toggle_game_ui")).clicked() {
                        Thread::main_thread().schedule(Self::toggle_game_ui);
                    }
                });
            });
        });

        if !self.show_menu {
            if let Some(time) = self.menu_anim_time {
                if time.elapsed().as_secs_f32() >= ctx.style().animation_time {
                    self.menu_visible = false;
                }
            }
            else {
                self.menu_anim_time = Some(Instant::now());
            }
        }

        if let Some(content) = show_notification {
            self.show_notification(content.as_ref());
        }

        if let Some(window) = show_window {
            self.show_window(window);
        }
    }

    fn toggle_game_ui() {
        use crate::il2cpp::hook::{
            UnityEngine_CoreModule::{Object, Behaviour, GameObject},
            UnityEngine_UIModule::Canvas,
            Plugins::AnimateToUnity::AnRoot
        };

        let canvas_array = Object::FindObjectsOfType(Canvas::type_object(), true);
        let an_root_array = Object::FindObjectsOfType(AnRoot::type_object(), true);
        let canvas_iter = unsafe { canvas_array.as_slice().iter() };
        let an_root_iter = unsafe { an_root_array.as_slice().iter() };

        if unsafe { DISABLED_GAME_UIS.is_empty() } {
            for canvas in canvas_iter {
                if Behaviour::get_enabled(*canvas) {
                    Behaviour::set_enabled(*canvas, false);
                    unsafe { DISABLED_GAME_UIS.insert(*canvas); }
                }
            }
            for an_root in an_root_iter {
                let top_object = AnRoot::get__topObject(*an_root);
                if GameObject::get_activeSelf(top_object) {
                    GameObject::SetActive(top_object, false);
                    unsafe { DISABLED_GAME_UIS.insert(top_object); }
                }
            }
        }
        else {
            for canvas in canvas_iter {
                if unsafe { DISABLED_GAME_UIS.contains(canvas) } {
                    Behaviour::set_enabled(*canvas, true);
                }
            }
            for an_root in an_root_iter {
                let top_object = AnRoot::get__topObject(*an_root);
                if unsafe { DISABLED_GAME_UIS.contains(&top_object) } {
                    GameObject::SetActive(top_object, true);
                }
            }
            unsafe { DISABLED_GAME_UIS.clear(); }
        }
    }

    #[cfg(target_os = "windows")]
    fn run_vsync_combo(ui: &mut egui::Ui, value: &mut i32) {
        Self::run_combo(ui, "vsync_combo", value, &[
            (-1, &t!("default")),
            (0, &t!("off")),
            (1, &t!("on")),
            (2, "1/2"),
            (3, "1/3"),
            (4, "1/4")
        ]);
    }

    fn run_combo<T: PartialEq + Copy>(
        ui: &mut egui::Ui,
        id_child: impl std::hash::Hash,
        value: &mut T,
        choices: &[(T, &str)]
    ) -> bool {
        let mut selected = "Unknown";
        for choice in choices.iter() {
            if *value == choice.0 {
                selected = choice.1;
            }
        }

        let mut changed = false;
        egui::ComboBox::new(ui.id().with(id_child), "")
        .selected_text(selected)
        .show_ui(ui, |ui| {
            for choice in choices.iter() {
                changed |= ui.selectable_value(value, choice.0, choice.1).changed();
            }
        });

        changed
    }

    fn run_update_progress(&mut self) {
        let ctx = &self.context;
        let progress = Hachimi::instance().tl_updater.progress().unwrap_or_else(|| {
            // Assume that update is complete
            self.update_progress_visible = false;
            tl_repo::UpdateProgress::new(1, 1)
        });
        let ratio = progress.current as f32 / progress.total as f32;

        egui::Area::new("update_progress".into())
        .fixed_pos(egui::Pos2 {
            x: 4.0,
            y: 4.0
        })
        .show(ctx, |ui| {
            egui::Frame::none()
            .fill(BACKGROUND_COLOR)
            .inner_margin(egui::Margin::same(4.0))
            .rounding(4.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(t!("tl_updater.title"));
                    ui.add_space(26.0);
                    ui.label(format!("{:.2}%", ratio * 100.0));
                });
                ui.add(
                    egui::ProgressBar::new(ratio)
                    .desired_height(4.0)
                    .desired_width(140.0)
                );
                ui.label(
                    egui::RichText::new(t!("tl_updater.warning"))
                    .font(egui::FontId::proportional(10.0))
                );
            });
        });
    }

    fn run_notifications(&mut self) {
        let mut offset: f32 = -16.0;
        self.notifications.retain_mut(|n| n.run(&self.context, &mut offset));
    }

    fn run_windows(&mut self) {
        self.windows.retain_mut(|w| w.run(&self.context));
    }

    pub fn is_empty(&self) -> bool {
        !self.splash_visible && !self.menu_visible && !self.update_progress_visible &&
        self.notifications.is_empty() && self.windows.is_empty()
    }

    pub fn is_consuming_input(&self) -> bool {
        self.menu_visible || !self.windows.is_empty()
    }

    pub fn is_consuming_input_atomic() -> bool {
        IS_CONSUMING_INPUT.load(atomic::Ordering::Relaxed)
    }

    pub fn toggle_menu(&mut self) {
        self.show_menu = !self.show_menu;
        // Menu is always visible on show, but not immediately invisible on hide
        if self.show_menu {
            self.menu_visible = true;
        }
        else {
            self.menu_anim_time = None;
        }
    }

    pub fn show_notification(&mut self, content: &str) {
        self.notifications.push(Notification::new(content.to_owned()));
    }

    pub fn show_window(&mut self, window: BoxedWindow) {
        self.windows.push(window);
    }
}

struct TweenInOutWithDelay {
    tween_time: f32,
    delay_duration: f32,
    easing: Easing,

    started: bool,
    delay_start: Option<Instant>
}

enum Easing {
    //Linear,
    //InQuad,
    OutQuad
}

impl TweenInOutWithDelay {
    fn new(tween_time: f32, delay_duration: f32, easing: Easing) -> TweenInOutWithDelay {
        TweenInOutWithDelay {
            tween_time,
            delay_duration,
            easing,

            started: false,
            delay_start: None
        }
    }

    fn run(&mut self, ctx: &egui::Context, id: egui::Id) -> Option<f32> {
        let anim_dir = if let Some(start) = self.delay_start {
            // Hold animation at peak position until duration passes
            start.elapsed().as_secs_f32() < self.delay_duration
        }
        else {
            // On animation start, initialize to 0.0. Next calls will start tweening to 1.0
            let v = self.started;
            self.started = true;
            v
        };
        let tween_val = ctx.animate_bool_with_time(id, anim_dir, self.tween_time);

        // Switch on delay when animation hits peak (next call makes tween_val < 1.0)
        if tween_val == 1.0 && self.delay_start.is_none() {
            self.delay_start = Some(Instant::now());
        }
        // Check if everything's done
        else if tween_val == 0.0 && self.delay_start.is_some() {
            return None;
        }

        Some(
            match self.easing {
                //Easing::Linear => tween_val,
                //Easing::InQuad => tween_val * tween_val,
                Easing::OutQuad => 1.0 - (1.0 - tween_val) * (1.0 - tween_val)
            }
        )
    }
}

// quick n dirty random id generator
fn random_id() -> egui::Id {
    egui::Id::new(egui::epaint::ahash::RandomState::new().hash_one(0))
}

struct Notification {
    content: String,
    tween: TweenInOutWithDelay,
    id: egui::Id
}

impl Notification {
    fn new(content: String) -> Notification {
        Notification {
            content,
            tween: TweenInOutWithDelay::new(0.2, 3.0, Easing::OutQuad),
            id: random_id()
        }
    }

    const WIDTH: f32 = 150.0;
    fn run(&mut self, ctx: &egui::Context, offset: &mut f32) -> bool {
        let Some(tween_val) = self.tween.run(ctx, self.id.with("tween")) else {
            return false;
        };

        let frame_rect = egui::Area::new(self.id)
        .anchor(
            egui::Align2::RIGHT_BOTTOM,
            egui::Vec2::new(
                Self::WIDTH * (1.0 - tween_val),
                *offset
            )
        )
        .show(ctx, |ui| {
            egui::Frame::none()
            .fill(BACKGROUND_COLOR)
            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
            .show(ui, |ui| {
                ui.set_width(Self::WIDTH);
                ui.label(&self.content);
            }).response.rect
        }).inner;

        *offset -= 2.0 + frame_rect.height() * tween_val;
        true
    }
}

pub trait Window {
    fn run(&mut self, ctx: &egui::Context) -> bool;
}

// Shared window creation function
fn new_window<'a>(ctx: &egui::Context, title: impl Into<egui::WidgetText>) -> egui::Window<'a> {
    egui::Window::new(title)
    .pivot(egui::Align2::CENTER_CENTER)
    .fixed_pos(ctx.screen_rect().max / 2.0)
    .max_width(320.0)
    .max_height(250.0)
    .collapsible(false)
    .resizable(false)
}

fn simple_window_layout(ui: &mut egui::Ui, id: egui::Id, add_contents: impl FnOnce(&mut egui::Ui), add_buttons: impl FnOnce(&mut egui::Ui)) {
    add_contents(ui);
    egui::TopBottomPanel::bottom(id.with("bottom_panel"))
    .show_inside(ui, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), add_buttons)
    });
}

fn paginated_window_layout(ui: &mut egui::Ui, id: egui::Id, i: &mut usize, page_count: usize, add_page_content: impl FnOnce(&mut egui::Ui, usize) -> bool) -> bool {
    let allow_next = add_page_content(ui, *i);
    egui::TopBottomPanel::bottom(id.with("bottom_panel"))
    .show_inside(ui, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            let mut open = true;
            if *i < page_count - 1 {
                if allow_next && ui.button(t!("next")).clicked() {
                    *i += 1;
                }
            }
            else {
                if ui.button(t!("done")).clicked() {
                    open = false;
                }
            }
            if *i > 0 && ui.button(t!("previous")).clicked() {
                *i -= 1;
            }

            open
        }).inner
    }).inner
}

fn async_request_ui_content<T: Send + Sync + 'static>(ui: &mut egui::Ui, request: Arc<AsyncRequest<T>>, add_contents: impl FnOnce(&mut egui::Ui, &T)) {
    let Some(result) = &**request.result.load() else {
        if !request.running() {
            request.call();
        }
        ui.centered_and_justified(|ui| {
            ui.label(t!("loading_label"));
        });
        return;
    };

    match result {
        Ok(v) => add_contents(ui, v),
        Err(e) => {
            ui.centered_and_justified(|ui| {
                ui.label(e.to_string());
                if ui.button(t!("retry")).clicked() {
                    request.call();
                }
            });
        }
    }
}

pub struct SimpleYesNoDialog {
    title: String,
    content: String,
    callback: fn(bool),
    id: egui::Id
}

impl SimpleYesNoDialog {
    pub fn new(title: &str, content: &str, callback: fn(bool)) -> SimpleYesNoDialog {
        SimpleYesNoDialog {
            title: title.to_owned(),
            content: content.to_owned(),
            callback,
            id: random_id()
        }
    }
}

impl Window for SimpleYesNoDialog {
    fn run(&mut self, ctx: &egui::Context) -> bool {
        let mut open = true;
        let mut open2 = true;
        let mut result = false;

        new_window(ctx, &self.title)
        .id(self.id)
        .open(&mut open)
        .show(ctx, |ui| {
            simple_window_layout(ui, self.id,
                |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(&self.content);
                    });
                },
                |ui| {
                    if ui.button(t!("no")).clicked() {
                        open2 = false;
                    }
                    if ui.button(t!("yes")).clicked() {
                        result = true;
                        open2 = false;
                    }
                }
            );
        });

        if open && open2 {
            true
        }
        else {
            (self.callback)(result);
            false
        }
    }
}

pub struct SimpleOkDialog {
    title: String,
    content: String,
    callback: fn(),
    id: egui::Id
}

impl SimpleOkDialog {
    pub fn new(title: &str, content: &str, callback: fn()) -> SimpleOkDialog {
        SimpleOkDialog {
            title: title.to_owned(),
            content: content.to_owned(),
            callback,
            id: random_id()
        }
    }
}

impl Window for SimpleOkDialog {
    fn run(&mut self, ctx: &egui::Context) -> bool {
        let mut open = true;
        let mut open2 = true;

        new_window(ctx, &self.title)
        .id(self.id)
        .open(&mut open)
        .show(ctx, |ui| {
            simple_window_layout(ui, self.id,
                |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(&self.content);
                    });
                },
                |ui| {
                    if ui.button(t!("ok")).clicked() {
                        open2 = false;
                    }
                }
            );
        });

        if open && open2 {
            true
        }
        else {
            (self.callback)();
            false
        }
    }
}

struct ConfigEditor {
    config: hachimi::Config,
    id: egui::Id,
    current_tab: ConfigEditorTab
}

#[derive(Eq, PartialEq, Clone, Copy)]
enum ConfigEditorTab {
    General,
    Graphics,
    Gameplay
}

impl ConfigEditorTab {
    fn display_list() -> [(ConfigEditorTab, Cow<'static, str>); 3] {
        [
            (ConfigEditorTab::General, t!("config_editor.general_tab")),
            (ConfigEditorTab::Graphics, t!("config_editor.graphics_tab")),
            (ConfigEditorTab::Gameplay, t!("config_editor.gameplay_tab"))
        ]
    }
}

impl ConfigEditor {
    pub fn new() -> ConfigEditor {
        ConfigEditor {
            config: (**Hachimi::instance().config.load()).clone(),
            id: random_id(),
            current_tab: ConfigEditorTab::General
        }
    }

    fn option_slider<Num: egui::emath::Numeric>(ui: &mut egui::Ui, label: &str, value: &mut Option<Num>, range: RangeInclusive<Num>) {
        let mut checked = value.is_some();
        ui.label(label);
        ui.checkbox(&mut checked, t!("enable"));
        ui.end_row();

        if checked && value.is_none() {
            *value = Some(*range.start())
        }
        else if !checked && value.is_some() {
            *value = None;
        }

        if let Some(num) = value.as_mut() {
            ui.label("");
            ui.add(egui::Slider::new(num, range));
            ui.end_row();
        }
    }

    fn run_options_grid(config: &mut hachimi::Config, ui: &mut egui::Ui, tab: ConfigEditorTab) {
        match tab {
            ConfigEditorTab::General => {
                ui.label(t!("config_editor.language"));
                let lang_changed = Gui::run_combo(ui, "language", &mut config.language, Language::CHOICES);
                if lang_changed {
                    config.language.set_locale();
                }
                ui.end_row();

                ui.label(t!("config_editor.disable_overlay"));
                if ui.checkbox(&mut config.disable_gui, "").clicked() {
                    if config.disable_gui {
                        thread::spawn(|| {
                            Gui::instance().unwrap()
                            .lock().unwrap()
                            .show_window(Box::new(SimpleOkDialog::new(
                                &t!("warning"),
                                &t!("config_editor.disable_overlay_warning"),
                                || {}
                            )));
                        });
                    }
                }
                ui.end_row();

                ui.label(t!("config_editor.debug_mode"));
                ui.checkbox(&mut config.debug_mode, "");
                ui.end_row();

                ui.label(t!("config_editor.translator_mode"));
                ui.checkbox(&mut config.translator_mode, "");
                ui.end_row();

                ui.label(t!("config_editor.skip_first_time_setup"));
                ui.checkbox(&mut config.skip_first_time_setup, "");
                ui.end_row();

                ui.label(t!("config_editor.disable_auto_update_check"));
                ui.checkbox(&mut config.disable_auto_update_check, "");
                ui.end_row();

                ui.label(t!("config_editor.disable_translations"));
                ui.checkbox(&mut config.disable_translations, "");
                ui.end_row();

                ui.label(t!("config_editor.enable_ipc"));
                ui.checkbox(&mut config.enable_ipc, "");
                ui.end_row();

                ui.label(t!("config_editor.ipc_listen_all"));
                ui.checkbox(&mut config.ipc_listen_all, "");
                ui.end_row();

                ui.label(t!("config_editor.auto_translate_stories"));
                ui.checkbox(&mut config.auto_translate_stories, "");
                ui.end_row();

                ui.label(t!("config_editor.auto_translate_ui"));
                ui.checkbox(&mut config.auto_translate_localize, "");
                ui.end_row();
            },

            ConfigEditorTab::Graphics => {
                Self::option_slider(ui, &t!("config_editor.target_fps"), &mut config.target_fps, 30..=240);

                ui.label(t!("config_editor.virtual_resolution_multiplier"));
                ui.add(egui::Slider::new(&mut config.virtual_res_mult, 1.0..=4.0).step_by(0.1));
                ui.end_row();

                ui.label(t!("config_editor.ui_scale"));
                ui.add(egui::Slider::new(&mut config.ui_scale, 0.1..=10.0).step_by(0.05));
                ui.end_row();

                ui.label(t!("config_editor.ui_animation_scale"));
                ui.add(egui::Slider::new(&mut config.ui_animation_scale, 0.1..=10.0).step_by(0.1));
                ui.end_row();

                ui.label(t!("config_editor.graphics_quality"));
                Gui::run_combo(ui, "graphics_quality", &mut config.graphics_quality, &[
                    (GraphicsQuality::Default, &t!("default")),
                    (GraphicsQuality::Toon1280, "Toon1280"),
                    (GraphicsQuality::Toon1280x2, "Toon1280x2"),
                    (GraphicsQuality::Toon1280x4, "Toon1280x4"),
                    (GraphicsQuality::ToonFull, "ToonFull"),
                    (GraphicsQuality::Max, "Max")
                ]);
                ui.end_row();

                #[cfg(target_os = "windows")]
                {
                    use crate::windows::hachimi_impl::{FullScreenMode, ResolutionScaling};

                    ui.label(t!("config_editor.vsync"));
                    Gui::run_vsync_combo(ui, &mut config.windows.vsync_count);
                    ui.end_row();

                    ui.label(t!("config_editor.auto_full_screen"));
                    ui.checkbox(&mut config.windows.auto_full_screen, "");
                    ui.end_row();

                    ui.label(t!("config_editor.full_screen_mode"));
                    Gui::run_combo(ui, "full_screen_mode", &mut config.windows.full_screen_mode, &[
                        (FullScreenMode::ExclusiveFullScreen, &t!("config_editor.full_screen_mode_exclusive")),
                        (FullScreenMode::FullScreenWindow, &t!("config_editor.full_screen_mode_borderless"))
                    ]);
                    ui.end_row();

                    ui.label(t!("config_editor.block_minimize_in_full_screen"));
                    ui.checkbox(&mut config.windows.block_minimize_in_full_screen, "");
                    ui.end_row();

                    ui.label(t!("config_editor.resolution_scaling"));
                    Gui::run_combo(ui, "resolution_scaling", &mut config.windows.resolution_scaling, &[
                        (ResolutionScaling::Default, &t!("config_editor.resolution_scaling_default")),
                        (ResolutionScaling::ScaleToScreenSize, &t!("config_editor.resolution_scaling_ssize")),
                        (ResolutionScaling::ScaleToWindowSize, &t!("config_editor.resolution_scaling_wsize"))
                    ]);
                    ui.end_row();

                    ui.label(t!("config_editor.window_always_on_top"));
                    ui.checkbox(&mut config.windows.window_always_on_top, "");
                    ui.end_row();
                }
            },

            ConfigEditorTab::Gameplay => {
                ui.label(t!("config_editor.physics_update_mode"));
                Gui::run_combo(ui, "physics_update_mode", &mut config.physics_update_mode, &[
                    (None, &t!("default")),
                    (SpringUpdateMode::ModeNormal.into(), "ModeNormal"),
                    (SpringUpdateMode::Mode60FPS.into(), "Mode60FPS"),
                    (SpringUpdateMode::SkipFrame.into(), "SkipFrame"),
                    (SpringUpdateMode::SkipFramePostAlways.into(), "SkipFramePostAlways")
                ]);
                ui.end_row();

                ui.label(t!("config_editor.story_choice_auto_select_delay"));
                ui.add(egui::Slider::new(&mut config.story_choice_auto_select_delay, 0.1..=10.0).step_by(0.05));
                ui.end_row();

                ui.label(t!("config_editor.story_text_speed_multiplier"));
                ui.add(egui::Slider::new(&mut config.story_tcps_multiplier, 0.1..=10.0).step_by(0.1));
                ui.end_row();

                ui.label(t!("config_editor.force_allow_dynamic_camera"));
                ui.checkbox(&mut config.force_allow_dynamic_camera, "");
                ui.end_row();

                ui.label(t!("config_editor.live_theater_allow_same_chara"));
                ui.checkbox(&mut config.live_theater_allow_same_chara, "");
                ui.end_row();

                ui.label(t!("config_editor.disable_skill_name_translation"));
                ui.checkbox(&mut config.disable_skill_name_translation, "");
                ui.end_row();
            }
        }

        // Column widths workaround
        ui.horizontal(|ui| ui.add_space(100.0));
        ui.horizontal(|ui| ui.add_space(150.0));
        ui.end_row();
    }
}

impl Window for ConfigEditor {
    fn run(&mut self, ctx: &egui::Context) -> bool {
        let mut open = true;
        let mut open2 = true;
        let mut config = self.config.clone();

        new_window(ctx, t!("config_editor.title"))
        .id(self.id)
        .open(&mut open)
        .show(ctx, |ui| {
            simple_window_layout(ui, self.id,
                |ui| {
                    egui::ScrollArea::horizontal()
                    .id_source("tabs_scroll")
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let style = ui.style_mut();
                            style.spacing.button_padding = egui::vec2(8.0, 5.0);
                            style.spacing.item_spacing = egui::Vec2::ZERO;
                            let widgets = &mut style.visuals.widgets;
                            widgets.inactive.rounding = egui::Rounding::ZERO;
                            widgets.hovered.rounding = egui::Rounding::ZERO;
                            widgets.active.rounding = egui::Rounding::ZERO;

                            for (tab, label) in ConfigEditorTab::display_list() {
                                if ui.selectable_label(self.current_tab == tab, label.as_ref()).clicked() {
                                    self.current_tab = tab;
                                }
                            }
                        });
                    });

                    ui.add_space(4.0);

                    egui::ScrollArea::vertical()
                    .id_source("body_scroll")
                    .show(ui, |ui| {
                        egui::Frame::none()
                        .inner_margin(egui::Margin::symmetric(8.0, 0.0))
                        .show(ui, |ui| {
                            egui::Grid::new(self.id.with("options_grid"))
                            .striped(true)
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .show(ui, |ui| {
                                Self::run_options_grid(&mut config, ui, self.current_tab);
                            });
                        });
                    });
                },
                |ui| {
                    if ui.button(t!("cancel")).clicked() {
                        open2 = false;
                    }
                    if ui.button(t!("save")).clicked() {
                        save_and_reload_config(self.config.clone());
                        open2 = false;
                    }
                }
            );
        });

        self.config = config;

        open &= open2;
        if !open {
            let config_locale = Hachimi::instance().config.load().language.locale_str();
            if config_locale != &*rust_i18n::locale() {
                rust_i18n::set_locale(config_locale);
            }
        }

        open
    }
}

fn save_and_reload_config(config: hachimi::Config) {
    let notif = match Hachimi::instance().save_and_reload_config(config) {
        Ok(_) => t!("notification.config_saved").into_owned(),
        Err(e) => e.to_string()
    };

    // workaround since we can't get a mutable ref to the Gui and
    // locking the mutex on the current thread would cause a deadlock
    thread::spawn(move || {
        Gui::instance().unwrap()
        .lock().unwrap()
        .show_notification(&notif);
    });
}

struct FirstTimeSetupWindow {
    id: egui::Id,
    index_request: Arc<AsyncRequest<Vec<RepoInfo>>>,
    current_page: usize,
    current_tl_repo: usize
}

impl FirstTimeSetupWindow {
    fn new() -> FirstTimeSetupWindow {
        FirstTimeSetupWindow {
            id: random_id(),
            index_request: Arc::new(tl_repo::new_meta_index_request()),
            current_page: 0,
            current_tl_repo: 0
        }
    }
}

impl Window for FirstTimeSetupWindow {
    fn run(&mut self, ctx: &egui::Context) -> bool {
        let mut open = true;
        let mut page_open = true;

        new_window(ctx, t!("first_time_setup.title"))
        .id(self.id)
        .open(&mut open)
        .show(ctx, |ui| {
            page_open = paginated_window_layout(ui, self.id, &mut self.current_page, 3, |ui, i| {
                match i {
                    0 => {
                        ui.heading(t!("first_time_setup.welcome_heading"));
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label(t!("config_editor.language"));

                            let hachimi = Hachimi::instance();
                            let config = &**hachimi.config.load();
                            let mut language = config.language;
                            let lang_changed = Gui::run_combo(ui, "language", &mut language, Language::CHOICES);
                            if lang_changed {
                                let mut config = config.clone();
                                config.language = language;
                                save_and_reload_config(config);
                            }   
                        });
                        ui.label(t!("first_time_setup.welcome_content"));
                        true
                    }
                    1 => {
                        ui.heading(t!("first_time_setup.translation_repo_heading"));
                        ui.separator();
                        ui.label(t!("first_time_setup.select_translation_repo"));
                        ui.add_space(4.0);

                        let mut selected = false;
                        async_request_ui_content(ui, self.index_request.clone(), |ui, repo_list| {
                            selected = repo_list.get(self.current_tl_repo).is_some();
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                egui::Frame::none()
                                .inner_margin(egui::Margin::symmetric(8.0, 0.0))
                                .show(ui, |ui| {
                                    for (i, repo) in repo_list.iter().enumerate() {
                                        ui.radio_value(&mut self.current_tl_repo, i, &repo.name);
                                        if let Some(list) = &repo.contributors {
                                            ui.label(
                                                egui::RichText::new(t!("first_time_setup.contributors", list = list))
                                                    .small()
                                            );
                                        }
                                    }
                                });
                            });
                        });
                        selected
                    }
                    2 => {
                        ui.heading(t!("first_time_setup.complete_heading"));
                        ui.separator();
                        ui.label(t!("first_time_setup.complete_content"));
                        true
                    }
                    _ => false
                }
            });
        });

        let open_res = open && page_open;
        if !open_res {
            let hachimi = Hachimi::instance();
            let mut config = (**hachimi.config.load()).clone();
            config.skip_first_time_setup = true;

            if !page_open {
                let Some(res) = &**self.index_request.result.load() else {
                    return open_res;
                };

                let Ok(repo_list) = res else {
                    return open_res;
                };

                let Some(repo) = repo_list.get(self.current_tl_repo) else {
                    return open_res;
                };

                config.translation_repo_index = Some(repo.index.clone());
            }

            save_and_reload_config(config);

            if !page_open {
                hachimi.tl_updater.clone().check_for_updates(false);
            }
        }

        open_res
    }
}

struct AboutWindow {
    id: egui::Id
}

impl AboutWindow {
    fn new() -> AboutWindow {
        AboutWindow {
            id: random_id()
        }
    }
}

impl Window for AboutWindow {
    fn run(&mut self, ctx: &egui::Context) -> bool {
        let mut open = true;

        new_window(ctx, t!("about.title"))
        .id(self.id)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add(Gui::icon_2x());
                ui.vertical(|ui| {
                    ui.heading(t!("hachimi"));
                    ui.label(env!("HACHIMI_DISPLAY_VERSION"));
                });
            });
            ui.label(t!("about.copyright"));
            ui.horizontal(|ui| {
                if ui.button(t!("about.view_license")).clicked() {
                    thread::spawn(|| {
                        Gui::instance().unwrap()
                        .lock().unwrap()
                        .show_window(Box::new(LicenseWindow::new()));
                    });
                }
                #[cfg(target_os = "windows")]
                if ui.button(t!("about.check_for_updates")).clicked() {
                    Hachimi::instance().updater.clone().check_for_updates(|_| {});
                }
            });
        });

        open
    }
}

struct LicenseWindow {
    id: egui::Id
}

impl LicenseWindow {
    fn new() -> LicenseWindow {
        LicenseWindow {
            id: random_id()
        }
    }
}

impl Window for LicenseWindow {
    fn run(&mut self, ctx: &egui::Context) -> bool {
        let mut open = true;

        new_window(ctx, t!("license.title"))
        .id(self.id)
        .open(&mut open)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(include_str!("../../LICENSE"));
            });
        });

        open
    }
}

pub struct PersistentMessageWindow {
    id: egui::Id,
    title: String,
    content: String,
    show: Arc<AtomicBool>
}

impl PersistentMessageWindow {
    pub fn new(title: &str, content: &str, show: Arc<AtomicBool>) -> PersistentMessageWindow {
        PersistentMessageWindow {
            id: random_id(),
            title: title.to_owned(),
            content: content.to_owned(),
            show
        }
    }
}

impl Window for PersistentMessageWindow {
    fn run(&mut self, ctx: &egui::Context) -> bool {
        new_window(ctx, &self.title)
        .id(self.id)
        .show(ctx, |ui| {
            simple_window_layout(ui, self.id,
                |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(&self.content);
                    });
                },
                |_| {
                }
            );
        });

        self.show.load(atomic::Ordering::Relaxed)
    }
}