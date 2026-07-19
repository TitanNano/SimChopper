use std::collections::VecDeque;
use std::io::Read;

use case::CaseExt;
use godot::builtin::{GString, PackedStringArray, StringName, Variant};
use godot::classes::class_macros::private::class_macros::private::class_macros::meta::ToGodot;
use godot::classes::file_access::ModeFlags;
use godot::classes::{
    ClassDb, FileAccess, IResourceFormatLoader, IResourceFormatSaver, Object, Resource,
};
use godot::global;
use godot::obj::{Gd, Singleton};
use godot::prelude::{godot_api, GodotClass};
use godot::tools::GFile;
use num::ToPrimitive;
use serde::Deserializer;

use crate::util::logger;

#[derive(GodotClass)]
#[class(base = ResourceFormatLoader, init)]
pub struct TomlResourceLoader {}

#[godot_api]
impl IResourceFormatLoader for TomlResourceLoader {
    fn get_recognized_extensions(&self) -> PackedStringArray {
        let mut extensions = PackedStringArray::new();

        extensions.push("toml");

        extensions
    }

    fn recognize_path(&self, path: GString, _type: StringName) -> bool {
        logger::debug!(
            "checking resource path {} for toml support! Ext: {}",
            path,
            path.get_extension()
        );

        path.get_extension() == "toml"
    }

    fn handles_type(&self, ty: StringName) -> bool {
        ClassDb::singleton().is_parent_class(&ty, "Resource")
    }

    fn get_resource_type(&self, _path: GString) -> GString {
        GString::from("Resource")
    }
    fn exists(&self, path: GString) -> bool {
        FileAccess::file_exists(&path)
    }
    fn load(
        &self,
        path: GString,
        _original_path: GString,
        _use_sub_threads: bool,
        _cache_mode: i32,
    ) -> Variant {
        logger::debug!("loading TOML resource...");
        let mut file = match GFile::open(&path, ModeFlags::READ) {
            Ok(file) => file,
            Err(err) => {
                logger::error!("failed to open file: {}", err);
                return Variant::nil();
            }
        };

        let mut buf = String::with_capacity(file.length().to_usize().unwrap());
        let result = file.read_to_string(&mut buf);

        if let Err(err) = result {
            logger::error!("Failed to read toml file into buffer: {}", err);
            return Variant::nil();
        }

        let toml = match toml::from_str::<toml::Table>(&buf) {
            Ok(value) => value,
            Err(err) => {
                logger::error!("TOML parse error: {}", err);
                return Variant::nil();
            }
        };

        logger::debug!("parsed toml contents!");
        if toml.len() != 1 {
            logger::error!("TOML file is expected to have a single top-level table!");
            return Variant::nil();
        }

        let Some((table_name, toml::Value::Table(contents))) = toml.into_iter().next() else {
            logger::error!("TOML file is expected to have a single top-level table!");
            return Variant::nil();
        };

        let class = table_name.to_camel();

        logger::debug!("deserializing toml class: {}", class);

        if !ClassDb::singleton().class_exists(&class) {
            logger::error!("toml file uses unknown class {}", class);
            return Variant::nil();
        }

        let instance = ClassDb::singleton().instantiate(&class).to::<Gd<Object>>();

        logger::debug!("created instance for toml class!");

        let mut dyn_instance = match instance.try_dynify::<dyn SerializeTomlClass>() {
            Ok(inst) => inst,
            Err(inst) => {
                logger::error!("class {class} does not implement toml deserialization");
                if !inst.is_class("RefCounted") {
                    inst.free();
                }
                return Variant::nil();
            }
        };

        let result = dyn_instance.dyn_bind_mut().deserialize(contents);

        if let Some(err) = result {
            logger::error!("failed to deserialize TOML class {class}: {}", err);
            if !dyn_instance.is_class("RefCounted") {
                dyn_instance.free();
            }
            return Variant::nil();
        }

        dyn_instance.to_variant()
    }
}

pub trait SerializeTomlClass {
    fn deserialize(&mut self, der: toml::Table)
        -> Option<<toml::Table as Deserializer<'_>>::Error>;

    fn serialize(&self) -> Result<toml::Table, toml::ser::Error>;
}

#[derive(GodotClass)]
#[class(init, base = ResourceFormatSaver)]
pub struct TomlResourceSaver;

#[godot_api]
impl IResourceFormatSaver for TomlResourceSaver {
    fn save(
        &mut self,
        resource: Option<Gd<Resource>>,
        path: GString,
        _flags: u32,
    ) -> global::Error {
        let Some(resource) = resource else {
            return global::Error::ERR_INVALID_PARAMETER;
        };

        let serializable = match resource.try_dynify::<dyn SerializeTomlClass>() {
            Ok(val) => val,
            Err(obj) => {
                logger::error!(
                    "class {} does not implement TOML serialization!",
                    obj.get_class()
                );
                return global::Error::ERR_INVALID_PARAMETER;
            }
        };

        let table = match serializable.dyn_bind().serialize() {
            Ok(table) => table,
            Err(err) => {
                logger::error!("failed to serialize resource: {}", err);
                return global::Error::ERR_INVALID_DATA;
            }
        };

        let mut envelope = toml::Table::new();

        envelope.insert(
            serializable.get_class().to_string().to_snake(),
            toml::Value::Table(table),
        );

        let mut file = match GFile::open(&path, ModeFlags::WRITE) {
            Ok(file) => file,
            Err(err) => {
                logger::error!("Failed to write file: {path} - {err}");
                return global::Error::ERR_FILE_CANT_WRITE;
            }
        };

        let document = format_document(&envelope);

        if let Err(err) = file.write_gstring(&document) {
            logger::error!("Failed to store setting file: {err}");
            return global::Error::ERR_FILE_CANT_WRITE;
        }

        global::Error::OK
    }

    fn recognize(&self, resource: Option<Gd<Resource>>) -> bool {
        let Some(res) = resource else {
            return false;
        };

        res.try_dynify::<dyn SerializeTomlClass>().is_ok()
    }

    fn get_recognized_extensions(&self, resource: Option<Gd<Resource>>) -> PackedStringArray {
        let mut list = PackedStringArray::new();

        if !self.recognize(resource) {
            return list;
        }

        list.push("toml");
        list
    }

    fn recognize_path(&self, resource: Option<Gd<Resource>>, path: GString) -> bool {
        if !self.recognize(resource) {
            return false;
        }

        path.get_extension() == "toml"
    }
}
fn should_be_inline_table(table: &toml_edit::Table) -> bool {
    table.len() == 2 && table.contains_key("x") && table.contains_key("y")
}

fn should_not_be_inline_table(table: &toml_edit::InlineTable) -> bool {
    table.len() != 2 || !(table.contains_key("x") && table.contains_key("y"))
}

fn format_document(table: &toml::Table) -> String {
    let mut serializer =
        toml_edit::ser::to_document(&table).expect("should be able to convert table to document");
    let mut queue: VecDeque<_> = serializer.iter_mut().collect();

    while let Some((key, item)) = queue.pop_front() {
        match item {
            toml_edit::Item::Table(table) if should_be_inline_table(table) => {
                *item = toml_edit::Item::Value(toml_edit::Value::InlineTable(
                    table.clone().into_inline_table(),
                ));
            }

            toml_edit::Item::Table(table) => {
                logger::debug!("table {key} is not inline!");
                queue.extend(table.iter_mut());
            }

            toml_edit::Item::Value(toml_edit::Value::InlineTable(table))
                if should_not_be_inline_table(table) =>
            {
                *item = toml_edit::Item::Table(table.clone().into_table());

                queue.extend(item.as_table_mut().unwrap().iter_mut());
            }
            toml_edit::Item::None
            | toml_edit::Item::Value(_)
            | toml_edit::Item::ArrayOfTables(_) => (),
        }
    }
    serializer.to_string()
}
