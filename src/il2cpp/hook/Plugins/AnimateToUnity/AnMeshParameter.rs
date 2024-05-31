use std::{collections::hash_map, path::Path, sync::Mutex};

use fnv::FnvHashMap;
use once_cell::sync::Lazy;
use widestring::Utf16Str;

use crate::{
    core::Hachimi, 
    il2cpp::{
        hook::UnityEngine_CoreModule::{HideFlags_DontUnloadUnusedAsset, Material, Object, Texture2D},
        symbols::{get_method_addr, IDictionary, IList},
        types::*
    }
};

use super::AnMeshInfoParameterGroup;

static mut GET_MESHPARAMETERGROUPLIST_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_MeshParameterGroupList, GET_MESHPARAMETERGROUPLIST_ADDR, *mut Il2CppObject, this: *mut Il2CppObject);

// *mut Il2CppObject(AnMeshParameter), Map<TextureSetName, *mut Il2CppObject(Texture2D)>
// We map the texture sets to the mesh parameter because it's a Unity object and its lifetime can be tracked.
// The textures are destroyed in the Resources::UnloadUnusedAssets hook.
pub static TEXTURE_SET_OVERRIDES: Lazy<
    Mutex<FnvHashMap<usize, FnvHashMap<&Utf16Str, Option<usize>>>>
> = Lazy::new(|| Mutex::default());

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
    
    let Some(parameter_group_list) = IList::new(get_MeshParameterGroupList(this)) else {
        return res;
    };

    // Find the texture set associated with this texture
    for parameter_group in parameter_group_list.iter() {
        let Some(mesh_info_parameter_table) = IDictionary::<*mut Il2CppString, *mut Il2CppObject>::new(
            AnMeshInfoParameterGroup::get__meshInfoParameterTable(parameter_group)
        ) else {
            return res;
        };

        if mesh_info_parameter_table.contains(texture_name) {
            // found!
            let texture_set_name = AnMeshInfoParameterGroup::get_TextureSetName(parameter_group);
            let texture_set_name_utf16 = unsafe { (*texture_set_name).to_utf16str() };
            let material = unsafe { *material_ };

            // Get override map entry or insert
            let mut overrides = TEXTURE_SET_OVERRIDES.lock().unwrap();
            let amp_overrides = match overrides.entry(this as usize) {
                hash_map::Entry::Occupied(e) => {
                    // Check if a replacement is already loaded or not found
                    if let Some(texture_override_opt) = e.get().get(texture_set_name_utf16) {
                        if let Some(texture_override) = texture_override_opt {
                            Material::set_mainTexture(material, *texture_override as *mut Il2CppObject);
                        }
                        break;
                    }
                    e.into_mut()
                },
                hash_map::Entry::Vacant(e) => e.insert(FnvHashMap::default())
            };

            // Try to load a replacement
            let amp_name = unsafe { (*Object::get_name(this)).to_utf16str() };
            let texture_set_filename = texture_set_name_utf16.to_string() + ".png";
            // an_texture_sets/[amp_name]/[texture_set_name].png
            let mut rel_path = Path::new("an_texture_sets").join(&amp_name.to_string());
            rel_path.push(&texture_set_filename);

            let localized_data = Hachimi::instance().localized_data.load();
            if let Some(path) = localized_data.get_assets_path(&rel_path) {
                if let Some(texture) = Texture2D::from_image_file(&path, false, true) {
                    // Tell Unity not to unload this dangling texture
                    Object::set_hideFlags(texture, HideFlags_DontUnloadUnusedAsset);

                    // Set the texture
                    Material::set_mainTexture(material, texture);

                    // Add it to the override map
                    amp_overrides.insert(texture_set_name_utf16, Some(texture as usize));
                    break;
                }
            }

            // Mark as not found
            amp_overrides.insert(texture_set_name_utf16, None);
            break;
        }
    }

    res
}

pub fn init(Plugins: *const Il2CppImage) {
    get_class_or_return!(Plugins, AnimateToUnity, AnMeshParameter);

    let _GetMaterial_addr = get_method_addr(AnMeshParameter, cstr!("_GetMaterial"), 7);

    new_hook!(_GetMaterial_addr, _GetMaterial);

    unsafe {
        GET_MESHPARAMETERGROUPLIST_ADDR = get_method_addr(AnMeshParameter, cstr!("get_MeshParameterGroupList"), 0);
    }
}