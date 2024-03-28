pub mod Enum;
pub mod File;

pub fn init() {
    get_assembly_image_or_return!(image, "mscorlib.dll");

    Enum::init(image);
    File::init(image);
}