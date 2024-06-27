mod SafetyNet;
mod Device;

pub fn init() {
    get_assembly_image_or_return!(image, "Cute.Core.Assembly.dll");

    SafetyNet::init(image);
    Device::init(image);
}