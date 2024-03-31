pub mod Texture2D;
mod Resources;
pub mod Sprite;
pub mod Object;
pub mod Application;

pub const HideFlags_DontUnloadUnusedAsset: i32 = 32;

pub fn init() {
    get_assembly_image_or_return!(image, "UnityEngine.CoreModule.dll");

    Texture2D::init(image);
    Resources::init(image);
    Sprite::init(image);
    Object::init(image);
    Application::init(image);
}