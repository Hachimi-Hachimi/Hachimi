pub mod Text;
pub mod CanvasScaler;

pub fn init() {
    get_assembly_image_or_return!(image, "UnityEngine.UI.dll");
    
    Text::init(image);
    CanvasScaler::init(image);
}