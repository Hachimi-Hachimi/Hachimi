pub mod TMP_FontAsset;
pub mod TMP_Text;

pub fn init() {
    get_assembly_image_or_return!(image, "Unity.TextMeshPro.dll");

    TMP_FontAsset::init(image);
    TMP_Text::init(image);
}