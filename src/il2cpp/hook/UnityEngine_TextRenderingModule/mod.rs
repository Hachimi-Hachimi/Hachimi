pub mod TextGenerator;

pub fn init() {
    get_assembly_image_or_return!(image, "UnityEngine.TextRenderingModule.dll");
    
    TextGenerator::init(image);
}