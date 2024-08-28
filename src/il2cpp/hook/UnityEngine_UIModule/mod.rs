pub mod Canvas;

pub fn init() {
    get_assembly_image_or_return!(image, "UnityEngine.UIModule.dll");
    
    Canvas::init(image);
}