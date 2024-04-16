mod render_hook;
mod input_hook;
mod input;
mod d3d11_backup;
mod d3d11_painter;

pub fn init() {
    render_hook::init();
    input_hook::init();
}