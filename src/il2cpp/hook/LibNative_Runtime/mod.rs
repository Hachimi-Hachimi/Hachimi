pub mod Sqlite3;

pub fn init() {
    get_assembly_image_or_return!(image, "LibNative.Runtime.dll");

    Sqlite3::init(image);
}