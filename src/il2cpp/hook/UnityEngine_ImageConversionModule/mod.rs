pub mod ImageConversion;

pub fn init() {
    get_assembly_image_or_return!(image, "UnityEngine.ImageConversionModule.dll");
    
    ImageConversion::init(image);
}