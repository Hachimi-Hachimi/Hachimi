mod MovieManager;

pub fn init() {
    get_assembly_image_or_return!(image, "Cute.Cri.Assembly.dll");

    MovieManager::init(image);
}