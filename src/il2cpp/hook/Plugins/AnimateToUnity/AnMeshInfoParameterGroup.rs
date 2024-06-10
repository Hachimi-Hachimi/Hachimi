use std::{collections::hash_map, path::Path, ptr::null_mut, sync::Mutex};

use fnv::FnvHashMap;
use once_cell::sync::Lazy;

use crate::{
    core::Hachimi,
    il2cpp::{
        hook::UnityEngine_CoreModule::{Material, Object, Texture2D},
        symbols::{get_field_from_name, get_field_object_value, get_method_addr, GCHandle},
        types::*
    }
};

static mut GET_TEXTURESETNAME_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_TextureSetName, GET_TEXTURESETNAME_ADDR, *mut Il2CppString, this: *mut Il2CppObject);

static mut GET_MESHPARAMETER_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_MeshParameter, GET_MESHPARAMETER_ADDR, *mut Il2CppObject, this: *mut Il2CppObject);

static mut _MESHINFOPARAMETERTABLE_FIELD: *mut FieldInfo = null_mut();
pub fn get__meshInfoParameterTable(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { _MESHINFOPARAMETERTABLE_FIELD })
}

// Cleaned up in the Resources::UnloadUnusedAssets hook
pub static PROCESSED_TEXTURES: Lazy<Mutex<FnvHashMap<usize, GCHandle>>> = Lazy::new(|| Mutex::default());

// DEV NOTE: The texture names can be found in AnMeshParameter asset bundles (which usually has "flash" in their path),
// in `AnMeshInfoParameter._textureName`. The name is a bit misleading because it works a bit more like sprites,
// different texture names can share the same texture. They're called "texture set" internally.
type _GetMaterialFn = extern "C" fn(
    this: *mut Il2CppObject, texture_name: *mut Il2CppString, shader_type: i32, stencil_ref: i32,
    base_stencil_ref: i32, stencil_compare_func: i32, use_custom_mesh: bool, material: *mut *mut Il2CppObject
) -> bool;
extern "C" fn _GetMaterial(
    this: *mut Il2CppObject, texture_name: *mut Il2CppString, shader_type: i32, stencil_ref: i32,
    base_stencil_ref: i32, stencil_compare_func: i32, use_custom_mesh: bool, material_: *mut *mut Il2CppObject
) -> bool {
    let res = get_orig_fn!(_GetMaterial, _GetMaterialFn)(this, texture_name, shader_type, stencil_ref,
        base_stencil_ref, stencil_compare_func, use_custom_mesh, material_);
    if !res { return res; }

    let material = unsafe { *material_ };
    let texture = Material::get_mainTexture(material);

    match PROCESSED_TEXTURES.lock().unwrap().entry(texture as usize) {
        hash_map::Entry::Occupied(_) => {
            return res;
        },
        hash_map::Entry::Vacant(e) => {
            e.insert(GCHandle::new_weak_ref(texture, false));
        }
    }

    let texture_set_name = get_TextureSetName(this);
    let texture_set_name_utf16 = unsafe { (*texture_set_name).to_utf16str() };
    
    // Try to load a replacement
    let amp = get_MeshParameter(this);
    let amp_name = unsafe { (*Object::get_name(amp)).to_utf16str() };
    let texture_set_filename = texture_set_name_utf16.to_string() + ".png";
    // an_texture_sets/[amp_name]/[texture_set_name].png
    let mut rel_path = Path::new("an_texture_sets").join(&amp_name.to_string());
    rel_path.push(&texture_set_filename);

    let localized_data = Hachimi::instance().localized_data.load();
    if let Some(path) = localized_data.get_assets_path(&rel_path) {
        Texture2D::load_image_file(texture, &path, true);
    }

    res
}

pub fn init(Plugins: *const Il2CppImage) {
    get_class_or_return!(Plugins, AnimateToUnity, AnMeshInfoParameterGroup);

    let _GetMaterial_addr = get_method_addr(AnMeshInfoParameterGroup, cstr!("_GetMaterial"), 7);

    new_hook!(_GetMaterial_addr, _GetMaterial);

    unsafe {
        GET_TEXTURESETNAME_ADDR = get_method_addr(AnMeshInfoParameterGroup, cstr!("get_TextureSetName"), 0);
        GET_MESHPARAMETER_ADDR = get_method_addr(AnMeshInfoParameterGroup, cstr!("get_MeshParameter"), 0);
        _MESHINFOPARAMETERTABLE_FIELD = get_field_from_name(AnMeshInfoParameterGroup, cstr!("_meshInfoParameterTable"));
    }
}