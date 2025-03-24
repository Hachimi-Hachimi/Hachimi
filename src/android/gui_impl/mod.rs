mod render_hook;
pub mod input_hook;
pub mod keymap;

pub fn init() {
    render_hook::init();
    input_hook::init();
}