pub mod Text;

pub fn init() {
    get_assembly_image_or_return!(image, "UnityEngine.UI.dll");
    
    Text::init(image);
}