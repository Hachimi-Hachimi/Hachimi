use std::path::Path;

use fnv::FnvHashMap;
use serde::Deserialize;
use widestring::Utf16Str;

use crate::{
	core::{ext::{StringExt, Utf16StringExt}, hachimi::AssetInfo, Hachimi},
	il2cpp::{
		api::{il2cpp_class_get_type, il2cpp_type_get_object},
		hook::{UnityEngine_AssetBundleModule::AssetBundle, UnityEngine_CoreModule::{Object, Texture2D}},
		symbols::{get_field_from_name, get_field_object_value, IList},
		types::*
	}
};

use super::{AnMeshInfoParameterGroup, AnMeshParameter, AnMeshParameterGroup, AnMotionParameter, AnMotionParameterGroup, AnObjectParameterBase, AnRootParameter, AnTextParameter};

static mut TYPE_OBJECT: *mut Il2CppObject = 0 as _;
pub fn type_object() -> *mut Il2CppObject {
    unsafe { TYPE_OBJECT }
}

// AnRootParameter
static mut _PARAMETER_FIELD: *mut FieldInfo = 0 as _;
pub fn get__parameter(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { _PARAMETER_FIELD })
}

// AnMeshParameterGroup
static mut _MESHPARAMETERGROUP_FIELD: *mut FieldInfo = 0 as _;
pub fn get__meshParameterGroup(this: *mut Il2CppObject) -> *mut Il2CppObject {
    get_field_object_value(this, unsafe { _MESHPARAMETERGROUP_FIELD })
}

#[derive(Deserialize)]
struct AnRootData {
	#[serde(default)]
	motion_parameter_list: FnvHashMap<i32, AnMotionParameterData>
}

#[derive(Deserialize)]
struct AnMotionParameterData {
	#[serde(default)]
	text_param_list: FnvHashMap<i32, AnTextParameterData>
}

#[derive(Deserialize)]
struct AnObjectParameterBaseData {
	position_offset: Option<Vector3_t>,
	scale: Option<Vector3_t>
}

#[derive(Deserialize)]
struct AnTextParameterData {
	text: Option<String>,

	#[serde(flatten)]
	base: AnObjectParameterBaseData
}

pub fn on_LoadAsset(bundle: *mut Il2CppObject, asset: &mut *mut Il2CppObject, name: &Utf16Str) {
	if !name.starts_with(AssetBundle::ASSET_PATH_PREFIX) {
        debug!("non-resource anroot: {}", name);
        return;
    }

    let base_path = name[AssetBundle::ASSET_PATH_PREFIX.len()..].path_basename();
    if !base_path.starts_with("uianimation/flash/") {
        debug!("bad path: {}", name);
        return;
    }
    let localized_data = Hachimi::instance().localized_data.load();
    let asset_info: AssetInfo<AnRootData> = localized_data.load_asset_info(&base_path.to_string());
    if !AssetBundle::check_asset_bundle_name(bundle, asset_info.metadata_ref()) {
        return;
    }

	let this = *asset;

	/*** Texture set replacement ***/
	let param_group = get__meshParameterGroup(this);
	let Some(param_list) = IList::new(AnMeshParameterGroup::get__meshParameterList(param_group)) else {
		return;
	};

	for param in param_list.iter() {
		let Some(group_list) = IList::new(AnMeshParameter::get__meshParameterGroupList(param)) else {
			return;
		};

		let amp_name = unsafe { (*Object::get_name(param)).to_utf16str() };
		let texture_sets_path = Path::new("an_texture_sets").join(&amp_name.to_string());

		for group in group_list.iter() {
			let texture = AnMeshInfoParameterGroup::get__textureSetColor(group);
			let texture_set_name = AnMeshInfoParameterGroup::get_TextureSetName(group);
			let texture_set_name_utf16 = unsafe { (*texture_set_name).to_utf16str() };
			
			// Try to load a replacement
			let texture_set_filename = texture_set_name_utf16.to_string() + ".png";
			let rel_path = texture_sets_path.join(texture_set_filename);

			let localized_data = Hachimi::instance().localized_data.load();
			if let Some(path) = localized_data.get_assets_path(&rel_path) {
				Texture2D::load_image_file(texture, &path, true);
			}
		}
	}

	/*** Asset data patches ***/
	if let Some(data) = asset_info.data {
		// quick escape!!!11
		if data.motion_parameter_list.is_empty() {
			return;
		}

		let root_param = get__parameter(this);
		let motion_param_group = AnRootParameter::get__motionParameterGroup(root_param);
		let Some(motion_param_list) = IList::new(AnMotionParameterGroup::get__motionParameterList(motion_param_group)) else {
			return;
		};

		for (i, motion_param_data) in data.motion_parameter_list.iter() {
			// quick escape!!!11
			if motion_param_data.text_param_list.is_empty() {
				continue;
			}

			let Some(motion_param) = motion_param_list.get(*i) else {
				continue;
			};

			let Some(text_param_list) = IList::new(AnMotionParameter::get__textParamList(motion_param)) else {
				continue;
			};
			
			for (j, text_param_data) in motion_param_data.text_param_list.iter() {
				let Some(text_param) = text_param_list.get(*j) else {
					continue;
				};

				if let Some(text) = &text_param_data.text {
					AnTextParameter::set__text(text_param, text.to_il2cpp_string());
				}

				if let Some(position_offset) = &text_param_data.base.position_offset {
					AnObjectParameterBase::set__positionOffset(text_param, position_offset);
				}
																	
				if let Some(scale) = &text_param_data.base.scale {
					AnObjectParameterBase::set__scale(text_param, scale);
				}
			}
		}
	}
}

pub fn init(Plugins: *const Il2CppImage) {
    get_class_or_return!(Plugins, AnimateToUnity, AnRoot);

    unsafe {
        TYPE_OBJECT = il2cpp_type_get_object(il2cpp_class_get_type(AnRoot));
		_PARAMETER_FIELD = get_field_from_name(AnRoot, cstr!("_parameter"));
		_MESHPARAMETERGROUP_FIELD = get_field_from_name(AnRoot, cstr!("_meshParameterGroup"));
    }
}