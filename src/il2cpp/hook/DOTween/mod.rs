mod TweenManager;

pub fn init() {
    get_assembly_image_or_return!(image, "DOTween.dll");

    TweenManager::init(image);
}