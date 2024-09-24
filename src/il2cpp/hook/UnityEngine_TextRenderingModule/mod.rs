pub mod TextGenerator;
pub mod Font;

pub fn init() {
    get_assembly_image_or_return!(image, "UnityEngine.TextRenderingModule.dll");

    TextGenerator::init(image);
    Font::init(image);
}