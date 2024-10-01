pub mod Texture2D;
mod Resources;
pub mod Sprite;
pub mod Object;
pub mod Application;
pub mod Material;
mod AsyncOperation;
pub mod GameObject;
pub mod Texture;
pub mod RenderTexture;
pub mod Graphics;
pub mod Behaviour;
pub mod Component;

#[cfg(target_os = "windows")]
pub mod QualitySettings;
#[cfg(target_os = "windows")]
pub mod Screen;

pub const HideFlags_DontUnloadUnusedAsset: i32 = 32;

pub const TextureFormat_RGBA32: i32 = 4;

pub const FullScreenMode_ExclusiveFullScreen: i32 = 0;
pub const FullScreenMode_FullScreenWindow: i32 = 1;

pub fn init() {
    get_assembly_image_or_return!(image, "UnityEngine.CoreModule.dll");

    Texture2D::init(image);
    Resources::init(image);
    Sprite::init(image);
    Object::init(image);
    Application::init(image);
    Material::init(image);
    AsyncOperation::init(image);
    GameObject::init(image);
    Texture::init(image);
    RenderTexture::init(image);
    Graphics::init(image);
    Behaviour::init(image);
    Component::init(image);

    #[cfg(target_os = "windows")]
    {
        QualitySettings::init(image);
        Screen::init(image);
    }
}