pub mod Texture2D;
mod Resources;
pub mod Sprite;
pub mod Object;
pub mod Application;
pub mod Material;
#[cfg(target_os = "windows")]
pub mod QualitySettings;

pub const HideFlags_DontUnloadUnusedAsset: i32 = 32;

pub const TextureFormat_RGBA32: i32 = 4;

pub fn init() {
    get_assembly_image_or_return!(image, "UnityEngine.CoreModule.dll");

    Texture2D::init(image);
    Resources::init(image);
    Sprite::init(image);
    Object::init(image);
    Application::init(image);
    Material::init(image);
    #[cfg(target_os = "windows")]
    QualitySettings::init(image);
}