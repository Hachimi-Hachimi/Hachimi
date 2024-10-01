pub mod TextGenerator;
pub mod Font;

#[repr(i32)]
pub enum TextAnchor {
    UpperLeft,
    UpperCenter,
    UpperRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    LowerLeft,
    LowerCenter,
    LowerRight
}

pub fn init() {
    get_assembly_image_or_return!(image, "UnityEngine.TextRenderingModule.dll");

    TextGenerator::init(image);
    Font::init(image);
}