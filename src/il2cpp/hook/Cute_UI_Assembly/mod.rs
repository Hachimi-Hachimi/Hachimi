pub mod AtlasReference;

pub fn init() {
    get_assembly_image_or_return!(image, "Cute.UI.Assembly.dll");

    AtlasReference::init(image);
}