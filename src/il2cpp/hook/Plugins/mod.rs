pub mod AnimateToUnity;

pub fn init() {
    get_assembly_image_or_return!(image, "Plugins.dll");

    AnimateToUnity::init(image);
}