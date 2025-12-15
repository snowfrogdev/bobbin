use bobbin_runtime::Runtime;
use godot::classes::{
    Engine, FileAccess, IResourceFormatLoader, IResourceFormatSaver, IScriptExtension,
    IScriptLanguageExtension, Resource, ResourceFormatLoader, ResourceFormatSaver, ResourceLoader,
    ResourceSaver, Script, ScriptExtension, ScriptLanguage, ScriptLanguageExtension,
    file_access::ModeFlags, script_language::ScriptNameCasing,
};
use godot::meta::RawPtr;
use godot::prelude::*;

struct BobbinExtension;

/// Find the registered Bobbin language by iterating through Engine's script languages
fn find_bobbin_language() -> Option<Gd<ScriptLanguage>> {
    let mut engine = Engine::singleton();
    let count = engine.get_script_language_count();
    for i in 0..count {
        if let Some(lang) = engine.get_script_language(i) {
            // Try to cast to our BobbinLanguage type - if it succeeds, this is our language
            if lang.clone().try_cast::<BobbinLanguage>().is_ok() {
                return Some(lang);
            }
        }
    }
    None
}

#[gdextension]
unsafe impl ExtensionLibrary for BobbinExtension {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            // Register the scripting language
            let language = Gd::from_init_fn(|base| BobbinLanguage { base });
            Engine::singleton().register_script_language(&language);

            // Register the resource loader for .bobbin files
            let loader = Gd::from_init_fn(|base| BobbinLoader { base });
            ResourceLoader::singleton().add_resource_format_loader(&loader);

            // Register the resource saver for .bobbin files
            let saver = Gd::from_init_fn(|base| BobbinSaver { base });
            ResourceSaver::singleton().add_resource_format_saver(&saver);
        }
    }
}

// =============================================================================
// BobbinLanguage - ScriptLanguageExtension (minimal)
// =============================================================================

#[derive(GodotClass)]
#[class(tool, init, base=ScriptLanguageExtension)]
pub struct BobbinLanguage {
    base: Base<ScriptLanguageExtension>,
}

#[godot_api]
impl IScriptLanguageExtension for BobbinLanguage {
    // --- Identity ---
    fn get_name(&self) -> GString {
        "Bobbin".into()
    }
    fn get_type(&self) -> GString {
        "BobbinScript".into()
    }
    fn get_extension(&self) -> GString {
        "bobbin".into()
    }
    fn get_recognized_extensions(&self) -> PackedStringArray {
        let mut arr = PackedStringArray::new();
        arr.push(&GString::from("bobbin"));
        arr
    }
    fn preferred_file_name_casing(&self) -> ScriptNameCasing {
        ScriptNameCasing::SNAKE_CASE
    }

    // --- Lifecycle ---
    fn init_ext(&mut self) {}
    fn finish(&mut self) {}
    fn frame(&mut self) {}
    fn thread_enter(&mut self) {}
    fn thread_exit(&mut self) {}

    // --- Script creation ---
    fn create_script(&self) -> Option<Gd<Object>> {
        let script = Gd::from_init_fn(|base| BobbinScript {
            base,
            source_code: GString::new(),
        });
        Some(script.upcast())
    }
    fn make_template(
        &self,
        _template: GString,
        _class_name: GString,
        _base_class_name: GString,
    ) -> Option<Gd<Script>> {
        // Return a new empty BobbinScript when creating a new file
        let script = Gd::from_init_fn(|base| BobbinScript {
            base,
            source_code: GString::from("// New Bobbin script\n"),
        });
        Some(script.upcast())
    }
    fn get_built_in_templates(&self, _object: StringName) -> Array<Dictionary> {
        Array::new()
    }
    fn is_using_templates(&mut self) -> bool {
        false
    }

    // --- Language features ---
    fn get_reserved_words(&self) -> PackedStringArray {
        PackedStringArray::new()
    }
    fn is_control_flow_keyword(&self, _keyword: GString) -> bool {
        false
    }
    fn get_comment_delimiters(&self) -> PackedStringArray {
        let mut arr = PackedStringArray::new();
        arr.push(&GString::from("//"));
        arr
    }
    fn get_string_delimiters(&self) -> PackedStringArray {
        let mut arr = PackedStringArray::new();
        arr.push(&GString::from("\" \""));
        arr
    }
    fn supports_builtin_mode(&self) -> bool {
        false
    }
    fn supports_documentation(&self) -> bool {
        false
    }
    fn can_inherit_from_file(&self) -> bool {
        false
    }
    fn has_named_classes(&self) -> bool {
        false
    }
    fn can_make_function(&self) -> bool {
        false
    }

    // --- Code editing ---
    fn validate(
        &self,
        _script: GString,
        _path: GString,
        _validate_functions: bool,
        _validate_errors: bool,
        _validate_warnings: bool,
        _validate_safe_lines: bool,
    ) -> Dictionary {
        let mut dict = Dictionary::new();
        dict.set("valid", true);
        dict
    }
    fn validate_path(&self, _path: GString) -> GString {
        GString::new()
    }
    fn find_function(&self, _function: GString, _code: GString) -> i32 {
        -1
    }
    fn make_function(
        &self,
        _class_name: GString,
        _function_name: GString,
        _function_args: PackedStringArray,
    ) -> GString {
        GString::new()
    }
    fn complete_code(
        &self,
        _code: GString,
        _path: GString,
        _owner: Option<Gd<Object>>,
    ) -> Dictionary {
        let mut dict = Dictionary::new();
        dict.set("result", 0i32); // CodeCompletionKind::NONE
        dict.set("call_hint", GString::new());
        dict.set("force", false);
        dict
    }
    fn lookup_code(
        &self,
        _code: GString,
        _symbol: GString,
        _path: GString,
        _owner: Option<Gd<Object>>,
    ) -> Dictionary {
        // Godot 4.3 requires all six keys to be present
        let mut dict = Dictionary::new();
        dict.set("result", 7i32); // Error::ERR_UNAVAILABLE = 7 (no result found)
        dict.set("type", 0i32); // LOOKUP_RESULT_SCRIPT_LOCATION
        dict.set("script", Variant::nil());
        dict.set("class_name", GString::new());
        dict.set("class_path", GString::new());
        dict.set("location", -1i32);
        dict
    }
    fn auto_indent_code(&self, code: GString, _from_line: i32, _to_line: i32) -> GString {
        code
    }

    // --- External editor ---
    fn open_in_external_editor(
        &mut self,
        _script: Option<Gd<Script>>,
        _line: i32,
        _column: i32,
    ) -> godot::global::Error {
        godot::global::Error::ERR_UNAVAILABLE
    }
    fn overrides_external_editor(&mut self) -> bool {
        false
    }

    // --- Global constants ---
    fn add_global_constant(&mut self, _name: StringName, _value: Variant) {}
    fn add_named_global_constant(&mut self, _name: StringName, _value: Variant) {}
    fn remove_named_global_constant(&mut self, _name: StringName) {}

    // --- Debugging ---
    fn debug_get_error(&self) -> GString {
        GString::new()
    }
    fn debug_get_stack_level_count(&self) -> i32 {
        0
    }
    fn debug_get_stack_level_line(&self, _level: i32) -> i32 {
        0
    }
    fn debug_get_stack_level_function(&self, _level: i32) -> GString {
        GString::new()
    }
    fn debug_get_stack_level_source(&self, _level: i32) -> GString {
        GString::new()
    }
    fn debug_get_stack_level_locals(
        &mut self,
        _level: i32,
        _max_subitems: i32,
        _max_depth: i32,
    ) -> Dictionary {
        Dictionary::new()
    }
    fn debug_get_stack_level_members(
        &mut self,
        _level: i32,
        _max_subitems: i32,
        _max_depth: i32,
    ) -> Dictionary {
        Dictionary::new()
    }
    unsafe fn debug_get_stack_level_instance_rawptr(
        &mut self,
        _level: i32,
    ) -> RawPtr<*mut std::ffi::c_void> {
        unsafe { RawPtr::new(std::ptr::null_mut()) }
    }
    fn debug_get_globals(&mut self, _max_subitems: i32, _max_depth: i32) -> Dictionary {
        Dictionary::new()
    }
    fn debug_parse_stack_level_expression(
        &mut self,
        _level: i32,
        _expression: GString,
        _max_subitems: i32,
        _max_depth: i32,
    ) -> GString {
        GString::new()
    }
    fn debug_get_current_stack_info(&mut self) -> Array<Dictionary> {
        Array::new()
    }

    // --- Reloading ---
    fn reload_all_scripts(&mut self) {
        // Godot handles iteration - each script's reload() is called separately
    }
    fn reload_tool_script(&mut self, script: Option<Gd<Script>>, _soft_reload: bool) {
        if let Some(s) = script {
            if let Ok(mut bobbin_script) = s.try_cast::<BobbinScript>() {
                let _ = bobbin_script.bind_mut().reload(true);
            }
        }
    }

    // --- Public API info ---
    fn get_public_functions(&self) -> Array<Dictionary> {
        Array::new()
    }
    fn get_public_constants(&self) -> Dictionary {
        Dictionary::new()
    }
    fn get_public_annotations(&self) -> Array<Dictionary> {
        Array::new()
    }

    // --- Profiling ---
    fn profiling_start(&mut self) {}
    fn profiling_stop(&mut self) {}
    fn profiling_set_save_native_calls(&mut self, _enable: bool) {}
    unsafe fn profiling_get_accumulated_data_rawptr(
        &mut self,
        _info_array: RawPtr<*mut godot::classes::native::ScriptLanguageExtensionProfilingInfo>,
        _info_max: i32,
    ) -> i32 {
        0
    }
    unsafe fn profiling_get_frame_data_rawptr(
        &mut self,
        _info_array: RawPtr<*mut godot::classes::native::ScriptLanguageExtensionProfilingInfo>,
        _info_max: i32,
    ) -> i32 {
        0
    }

    // --- Global class handling ---
    fn handles_global_class_type(&self, type_: GString) -> bool {
        type_ == GString::from("BobbinScript")
    }
    fn get_global_class_name(&self, _path: GString) -> Dictionary {
        Dictionary::new()
    }
}

// =============================================================================
// BobbinScript - ScriptExtension (holds source code for .bobbin files)
// =============================================================================

#[derive(GodotClass)]
#[class(tool, init, base=ScriptExtension)]
pub struct BobbinScript {
    base: Base<ScriptExtension>,
    #[var]
    pub source_code: GString,
}

#[godot_api]
impl IScriptExtension for BobbinScript {
    // --- Source code ---
    fn get_source_code(&self) -> GString {
        self.source_code.clone()
    }

    fn set_source_code(&mut self, code: GString) {
        self.source_code = code;
    }

    fn has_source_code(&self) -> bool {
        !self.source_code.is_empty()
    }

    // --- Editor ---
    fn editor_can_reload_from_file(&mut self) -> bool {
        true
    }

    // --- Script info ---
    fn can_instantiate(&self) -> bool {
        false
    }
    fn get_base_script(&self) -> Option<Gd<Script>> {
        None
    }
    fn get_global_name(&self) -> StringName {
        StringName::default()
    }
    fn inherits_script(&self, _script: Gd<Script>) -> bool {
        false
    }
    fn get_instance_base_type(&self) -> StringName {
        StringName::from("RefCounted")
    }

    // --- Instances ---
    unsafe fn instance_create_rawptr(
        &self,
        _for_object: Gd<Object>,
    ) -> RawPtr<*mut std::ffi::c_void> {
        unsafe { RawPtr::new(std::ptr::null_mut()) }
    }
    unsafe fn placeholder_instance_create_rawptr(
        &self,
        _for_object: Gd<Object>,
    ) -> RawPtr<*mut std::ffi::c_void> {
        unsafe { RawPtr::new(std::ptr::null_mut()) }
    }
    fn instance_has(&self, _object: Gd<Object>) -> bool {
        false
    }

    // --- Reloading ---
    fn reload(&mut self, _keep_state: bool) -> godot::global::Error {
        // Get file path from base
        let path = self.base().get_path();
        if path.is_empty() {
            return godot::global::Error::OK; // No file, nothing to reload
        }

        // Read source from disk
        let Some(file) = FileAccess::open(&path, ModeFlags::READ) else {
            godot_error!("BobbinScript::reload: Failed to open {}", path);
            return godot::global::Error::ERR_FILE_CANT_OPEN;
        };
        let content = file.get_as_text();

        // Update source_code
        self.source_code = content;

        godot::global::Error::OK
    }
    fn update_exports(&mut self) {}

    // --- Documentation ---
    fn get_documentation(&self) -> Array<Dictionary> {
        Array::new()
    }

    // --- Methods ---
    fn has_method(&self, _method: StringName) -> bool {
        false
    }
    fn has_static_method(&self, _method: StringName) -> bool {
        false
    }
    fn get_method_info(&self, _method: StringName) -> Dictionary {
        Dictionary::new()
    }
    fn get_script_method_list(&self) -> Array<Dictionary> {
        Array::new()
    }

    // --- Properties ---
    fn has_property_default_value(&self, _property: StringName) -> bool {
        false
    }
    fn get_property_default_value(&self, _property: StringName) -> Variant {
        Variant::nil()
    }
    fn get_script_property_list(&self) -> Array<Dictionary> {
        Array::new()
    }
    fn get_member_line(&self, _member: StringName) -> i32 {
        -1
    }
    fn get_constants(&self) -> Dictionary {
        Dictionary::new()
    }
    fn get_members(&self) -> Array<StringName> {
        Array::new()
    }

    // --- Signals ---
    fn has_script_signal(&self, _signal: StringName) -> bool {
        false
    }
    fn get_script_signal_list(&self) -> Array<Dictionary> {
        Array::new()
    }

    // --- Flags ---
    fn is_tool(&self) -> bool {
        false
    }
    fn is_valid(&self) -> bool {
        true
    }
    fn is_placeholder_fallback_enabled(&self) -> bool {
        false
    }

    // --- Language ---
    fn get_language(&self) -> Option<Gd<ScriptLanguage>> {
        find_bobbin_language()
    }

    // --- RPC ---
    fn get_rpc_config(&self) -> Variant {
        Variant::nil()
    }
}

// =============================================================================
// BobbinLoader - ResourceFormatLoaderExtension (loads .bobbin files from disk)
// =============================================================================

#[derive(GodotClass)]
#[class(tool, init, base=ResourceFormatLoader)]
pub struct BobbinLoader {
    base: Base<ResourceFormatLoader>,
}

#[godot_api]
impl IResourceFormatLoader for BobbinLoader {
    fn get_recognized_extensions(&self) -> PackedStringArray {
        let mut arr = PackedStringArray::new();
        arr.push(&GString::from("bobbin"));
        arr
    }

    fn handles_type(&self, type_: StringName) -> bool {
        type_ == StringName::from("BobbinScript") || type_ == StringName::from("Script")
    }

    fn get_resource_type(&self, path: GString) -> GString {
        if path.to_string().ends_with(".bobbin") {
            GString::from("BobbinScript")
        } else {
            GString::new()
        }
    }

    fn exists(&self, path: GString) -> bool {
        FileAccess::file_exists(&path)
    }

    fn load(
        &self,
        path: GString,
        _original_path: GString,
        _use_sub_threads: bool,
        cache_mode: i32,
    ) -> Variant {
        // Cache mode constants from Godot:
        // CACHE_MODE_IGNORE = 0
        // CACHE_MODE_REUSE = 1
        // CACHE_MODE_REPLACE = 2
        // CACHE_MODE_IGNORE_DEEP = 3
        // CACHE_MODE_REPLACE_DEEP = 4
        const CACHE_MODE_IGNORE: i32 = 0;
        const CACHE_MODE_REUSE: i32 = 1;
        const CACHE_MODE_REPLACE: i32 = 2;
        const CACHE_MODE_IGNORE_DEEP: i32 = 3;
        const CACHE_MODE_REPLACE_DEEP: i32 = 4;

        // Read file content
        let file = match FileAccess::open(&path, ModeFlags::READ) {
            Some(f) => f,
            None => {
                godot_error!("BobbinLoader: Failed to open file: {}", path);
                return Variant::nil();
            }
        };

        let content = file.get_as_text();

        // Create BobbinScript with the content
        let mut script = Gd::from_init_fn(|base| BobbinScript {
            base,
            source_code: content,
        });

        // Handle path/caching based on cache_mode (following lua-gdextension pattern)
        match cache_mode {
            CACHE_MODE_IGNORE | CACHE_MODE_IGNORE_DEEP => {
                // Don't set path - resource won't be cached
            }
            CACHE_MODE_REUSE => {
                // Normal load - set path for caching
                script.set_path(&path);
            }
            CACHE_MODE_REPLACE | CACHE_MODE_REPLACE_DEEP => {
                // Replace existing cached resource
                script.take_over_path(&path);
            }
            _ => {
                // Unknown mode - default to set_path
                script.set_path(&path);
            }
        }

        Variant::from(script)
    }
}

// =============================================================================
// BobbinSaver - ResourceFormatSaverExtension (saves .bobbin files to disk)
// =============================================================================

#[derive(GodotClass)]
#[class(tool, init, base=ResourceFormatSaver)]
pub struct BobbinSaver {
    base: Base<ResourceFormatSaver>,
}

#[godot_api]
impl IResourceFormatSaver for BobbinSaver {
    fn get_recognized_extensions(&self, _resource: Option<Gd<Resource>>) -> PackedStringArray {
        let mut arr = PackedStringArray::new();
        arr.push(&GString::from("bobbin"));
        arr
    }

    fn recognize(&self, resource: Option<Gd<Resource>>) -> bool {
        if let Some(res) = resource {
            // Check if this is a BobbinScript
            res.try_cast::<BobbinScript>().is_ok()
        } else {
            false
        }
    }

    fn save(
        &mut self,
        resource: Option<Gd<Resource>>,
        path: GString,
        _flags: u32,
    ) -> godot::global::Error {
        let Some(res) = resource else {
            godot_error!("BobbinSaver: No resource provided");
            return godot::global::Error::ERR_INVALID_PARAMETER;
        };

        let Ok(script) = res.try_cast::<BobbinScript>() else {
            godot_error!("BobbinSaver: Resource is not a BobbinScript");
            return godot::global::Error::ERR_INVALID_PARAMETER;
        };

        // Get the source code from the script
        let source_code = script.bind().source_code.clone();

        // Open file for writing
        let Some(mut file) = FileAccess::open(&path, ModeFlags::WRITE) else {
            godot_error!("BobbinSaver: Failed to open file for writing: {}", path);
            return godot::global::Error::ERR_FILE_CANT_WRITE;
        };

        // Write the source code
        file.store_string(&source_code);

        godot::global::Error::OK
    }
}

#[derive(GodotClass)]
#[class(base=RefCounted, no_init)]
pub struct BobbinRuntime {
    base: Base<RefCounted>,
    inner: Runtime,
}

#[godot_api]
impl BobbinRuntime {
    #[func]
    fn from_string(content: GString) -> Option<Gd<Self>> {
        match Runtime::new(&content.to_string()) {
            Ok(runtime) => Some(Gd::from_init_fn(|base| Self {
                base,
                inner: runtime,
            })),
            Err(e) => {
                godot_error!("Failed to load bobbin script: {:?}", e);
                None
            }
        }
    }

    #[func]
    fn advance(&mut self) {
        self.inner.advance();
    }

    #[func]
    fn current_line(&self) -> GString {
        GString::from(self.inner.current_line())
    }

    #[func]
    fn has_more(&self) -> bool {
        self.inner.has_more()
    }

    #[func]
    fn is_waiting_for_choice(&self) -> bool {
        self.inner.is_waiting_for_choice()
    }

    #[func]
    fn current_choices(&self) -> PackedStringArray {
        let choices = self.inner.current_choices();
        let mut arr = PackedStringArray::new();
        for choice in choices {
            arr.push(&GString::from(choice.as_str()));
        }
        arr
    }

    #[func]
    fn select_choice(&mut self, index: i32) {
        self.inner.select_choice(index as usize);
    }
}
