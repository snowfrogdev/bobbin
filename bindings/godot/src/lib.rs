use bobbin_runtime::{HostState, Runtime, Value, VariableStorage};
use godot::classes::{
    Engine, FileAccess, IResourceFormatLoader, IResourceFormatSaver, IScriptExtension,
    IScriptLanguageExtension, Os, Resource, ResourceFormatLoader, ResourceFormatSaver,
    ResourceLoader, ResourceSaver, Script, ScriptExtension, ScriptLanguage,
    ScriptLanguageExtension, SceneTree, Timer,
    file_access::ModeFlags, resource_loader::CacheMode, script_language::ScriptNameCasing,
};

// NOTE: EditorSyntaxHighlighter is currently broken in gdext - virtual methods
// like _get_name() are not dispatched to Rust implementations. See:
// https://github.com/godot-rust/gdext/issues/XXXX (to be filed)
// For now, we rely on get_reserved_words() in ScriptLanguageExtension which
// provides basic keyword highlighting via the Standard highlighter.
use godot::meta::RawPtr;
use godot::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

struct BobbinExtension;

// =============================================================================
// Storage and Host State Implementations
// =============================================================================

/// In-memory implementation of VariableStorage for Godot.
/// Thread-safe via RwLock.
struct MemoryStorage {
    values: RwLock<HashMap<String, Value>>,
}

impl MemoryStorage {
    fn new() -> Self {
        Self {
            values: RwLock::new(HashMap::new()),
        }
    }

    /// Get all variables as a copy of the internal map.
    fn get_all(&self) -> HashMap<String, Value> {
        self.values.read().unwrap().clone()
    }
}

impl VariableStorage for MemoryStorage {
    fn get(&self, name: &str) -> Option<Value> {
        self.values.read().unwrap().get(name).cloned()
    }

    fn set(&self, name: &str, value: Value) {
        self.values.write().unwrap().insert(name.to_string(), value);
    }

    fn initialize_if_absent(&self, name: &str, default: Value) {
        self.values
            .write()
            .unwrap()
            .entry(name.to_string())
            .or_insert(default);
    }

    fn contains(&self, name: &str) -> bool {
        self.values.read().unwrap().contains_key(name)
    }
}

/// Host state implementation backed by a HashMap.
/// Thread-safe via RwLock. Game can update values at any time.
struct VarDictionaryHostState {
    values: RwLock<HashMap<String, Value>>,
}

impl VarDictionaryHostState {
    fn from_dictionary(dict: &VarDictionary) -> Self {
        let mut values = HashMap::new();
        for key in dict.keys_array().iter_shared() {
            if let Ok(name) = key.try_to::<GString>() {
                if let Some(val) = dict.get(key.clone()) {
                    if let Some(value) = variant_to_value(&val) {
                        values.insert(name.to_string(), value);
                    }
                }
            }
        }
        Self {
            values: RwLock::new(values),
        }
    }

    /// Update a host variable (called by game).
    fn update(&self, name: &str, value: Value) {
        self.values.write().unwrap().insert(name.to_string(), value);
    }
}

impl HostState for VarDictionaryHostState {
    fn lookup(&self, name: &str) -> Option<Value> {
        self.values.read().unwrap().get(name).cloned()
    }
}

// =============================================================================
// Value Conversion Helpers
// =============================================================================

/// Convert Godot Variant to Bobbin Value.
fn variant_to_value(v: &Variant) -> Option<Value> {
    match v.get_type() {
        VariantType::STRING => Some(Value::String(v.to::<GString>().to_string())),
        VariantType::INT => Some(Value::Number(v.to::<i64>() as f64)),
        VariantType::FLOAT => Some(Value::Number(v.to::<f64>())),
        VariantType::BOOL => Some(Value::Bool(v.to::<bool>())),
        _ => None,
    }
}

/// Convert Bobbin Value to Godot Variant.
fn value_to_variant(v: &Value) -> Variant {
    match v {
        Value::String(s) => Variant::from(GString::from(s.as_str())),
        Value::Number(n) => {
            if n.fract() == 0.0 {
                Variant::from(*n as i64)
            } else {
                Variant::from(*n)
            }
        }
        Value::Bool(b) => Variant::from(*b),
    }
}

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
    fn get_built_in_templates(&self, _object: StringName) -> Array<VarDictionary> {
        Array::new()
    }
    fn is_using_templates(&mut self) -> bool {
        false
    }

    // --- Language features ---
    fn get_reserved_words(&self) -> PackedStringArray {
        let mut arr = PackedStringArray::new();
        arr.push(&GString::from("temp"));
        arr.push(&GString::from("save"));
        arr.push(&GString::from("set"));
        arr.push(&GString::from("extern"));
        arr.push(&GString::from("true"));
        arr.push(&GString::from("false"));
        arr
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
        script: GString,
        _path: GString,
        _validate_functions: bool,
        _validate_errors: bool,
        _validate_warnings: bool,
        _validate_safe_lines: bool,
    ) -> VarDictionary {
        #[cfg(feature = "editor-tooling")]
        {
            use bobbin_syntax::{validate, LineIndex};

            let source = script.to_string();
            let diagnostics = validate(&source);

            let mut dict = VarDictionary::new();
            // Always set all expected fields (matching GDScript's validate return)
            dict.set("functions", Array::<GString>::new());
            dict.set("warnings", Array::<VarDictionary>::new());
            dict.set("safe_lines", PackedInt32Array::new());

            if diagnostics.is_empty() {
                dict.set("valid", true);
                dict.set("errors", Array::<VarDictionary>::new());
            } else {
                dict.set("valid", false);
                let line_index = LineIndex::new(&source);
                let mut errors = Array::<VarDictionary>::new();
                for diag in &diagnostics {
                    let mut error = VarDictionary::new();
                    if let Some(label) = diag.primary_label() {
                        let pos = line_index.line_col(label.span.start);
                        let line = (pos.line + 1) as i32;
                        let column = (pos.column + 1) as i32;
                        error.set("line", line);
                        error.set("column", column);
                    } else {
                        error.set("line", 1i32);
                        error.set("column", 1i32);
                    }
                    error.set("message", GString::from(diag.message.as_str()));
                    errors.push(&error);
                }
                dict.set("errors", errors);
            }
            dict
        }

        #[cfg(not(feature = "editor-tooling"))]
        {
            let _ = script; // Silence unused variable warning
            let _ = path;
            let mut dict = VarDictionary::new();
            dict.set("valid", true);
            dict
        }
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
    ) -> VarDictionary {
        let mut dict = VarDictionary::new();
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
    ) -> VarDictionary {
        // Godot 4.3 requires all six keys to be present
        let mut dict = VarDictionary::new();
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
    ) -> VarDictionary {
        VarDictionary::new()
    }
    fn debug_get_stack_level_members(
        &mut self,
        _level: i32,
        _max_subitems: i32,
        _max_depth: i32,
    ) -> VarDictionary {
        VarDictionary::new()
    }
    unsafe fn debug_get_stack_level_instance_rawptr(
        &mut self,
        _level: i32,
    ) -> RawPtr<*mut std::ffi::c_void> {
        unsafe { RawPtr::new(std::ptr::null_mut()) }
    }
    fn debug_get_globals(&mut self, _max_subitems: i32, _max_depth: i32) -> VarDictionary {
        VarDictionary::new()
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
    fn debug_get_current_stack_info(&mut self) -> Array<VarDictionary> {
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
    fn get_public_functions(&self) -> Array<VarDictionary> {
        Array::new()
    }
    fn get_public_constants(&self) -> VarDictionary {
        VarDictionary::new()
    }
    fn get_public_annotations(&self) -> Array<VarDictionary> {
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
    fn get_global_class_name(&self, _path: GString) -> VarDictionary {
        VarDictionary::new()
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
    fn get_documentation(&self) -> Array<VarDictionary> {
        Array::new()
    }

    // --- Methods ---
    fn has_method(&self, _method: StringName) -> bool {
        false
    }
    fn has_static_method(&self, _method: StringName) -> bool {
        false
    }
    fn get_method_info(&self, _method: StringName) -> VarDictionary {
        VarDictionary::new()
    }
    fn get_script_method_list(&self) -> Array<VarDictionary> {
        Array::new()
    }

    // --- Properties ---
    fn has_property_default_value(&self, _property: StringName) -> bool {
        false
    }
    fn get_property_default_value(&self, _property: StringName) -> Variant {
        Variant::nil()
    }
    fn get_script_property_list(&self) -> Array<VarDictionary> {
        Array::new()
    }
    fn get_member_line(&self, _member: StringName) -> i32 {
        -1
    }
    fn get_constants(&self) -> VarDictionary {
        VarDictionary::new()
    }
    fn get_members(&self) -> Array<StringName> {
        Array::new()
    }

    // --- Signals ---
    fn has_script_signal(&self, _signal: StringName) -> bool {
        false
    }
    fn get_script_signal_list(&self) -> Array<VarDictionary> {
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
    storage: Arc<MemoryStorage>,
    host: Arc<VarDictionaryHostState>,
    inner: Runtime,

    // Hot reload support (debug builds only)
    source_path: Option<GString>,  // None if created via from_string()
    last_modified: u64,            // File modification timestamp
    poll_timer: Option<Gd<Timer>>, // Self-managed polling timer
}

#[godot_api]
impl BobbinRuntime {
    /// Create runtime from script content without host state.
    #[func]
    fn from_string(content: GString) -> Option<Gd<Self>> {
        // Use empty host state (no extern variables)
        Self::from_string_with_host(content, VarDictionary::new())
    }

    /// Create runtime with host state Dictionary.
    #[func]
    fn from_string_with_host(content: GString, host_state: VarDictionary) -> Option<Gd<Self>> {
        let storage = Arc::new(MemoryStorage::new());
        let host = Arc::new(VarDictionaryHostState::from_dictionary(&host_state));

        let storage_dyn: Arc<dyn VariableStorage> = storage.clone();
        let host_dyn: Arc<dyn HostState> = host.clone();

        match Runtime::new(&content.to_string(), storage_dyn, host_dyn) {
            Ok(runtime) => Some(Gd::from_init_fn(|base| Self {
                base,
                storage,
                host,
                inner: runtime,
                source_path: None,
                last_modified: 0,
                poll_timer: None,
            })),
            Err(e) => {
                godot_error!(
                    "Failed to create runtime:\n{}",
                    e.render("<script>", &content.to_string())
                );
                None
            }
        }
    }

    /// Create runtime from a .bobbin file path.
    #[func]
    fn from_file(path: GString) -> Option<Gd<Self>> {
        Self::from_file_with_host(path, VarDictionary::new())
    }

    /// Create runtime from a .bobbin file path with host state.
    #[func]
    fn from_file_with_host(path: GString, host_state: VarDictionary) -> Option<Gd<Self>> {
        // Load BobbinScript resource
        let Some(resource) = ResourceLoader::singleton()
            .load_ex(&path)
            .type_hint("BobbinScript")
            .done()
        else {
            godot_error!("BobbinRuntime::from_file: Failed to load {}", path);
            return None;
        };

        let Ok(script) = resource.try_cast::<BobbinScript>() else {
            godot_error!("BobbinRuntime::from_file: {} is not a BobbinScript", path);
            return None;
        };

        let source = script.bind().get_source_code().to_string();
        let storage = Arc::new(MemoryStorage::new());
        let host = Arc::new(VarDictionaryHostState::from_dictionary(&host_state));

        let storage_dyn: Arc<dyn VariableStorage> = storage.clone();
        let host_dyn: Arc<dyn HostState> = host.clone();

        match Runtime::new(&source, storage_dyn, host_dyn) {
            Ok(runtime) => {
                // Get initial modification time and setup hot reload (debug builds only)
                let (source_path, last_modified) = if Os::singleton().is_debug_build() {
                    let modified = FileAccess::get_modified_time(&path);
                    (Some(path), modified)
                } else {
                    (None, 0)
                };

                let mut instance = Gd::from_init_fn(|base| Self {
                    base,
                    storage,
                    host,
                    inner: runtime,
                    source_path,
                    last_modified,
                    poll_timer: None,
                });

                // Start hot reload polling (debug only, requires scene tree)
                instance.bind_mut().start_hot_reload();

                Some(instance)
            }
            Err(e) => {
                godot_error!(
                    "Failed to create runtime:\n{}",
                    e.render(&path.to_string(), &source)
                );
                None
            }
        }
    }

    // =========================================================================
    // Hot Reload
    // =========================================================================

    #[signal]
    fn reloaded();

    #[signal]
    fn reload_failed(error_message: GString);

    /// Reload with new source code. Preserves save variables.
    #[func]
    fn reload(&mut self, new_source: GString) -> bool {
        let source_str = new_source.to_string();
        let path_str = self
            .source_path
            .as_ref()
            .map(|p| p.to_string())
            .unwrap_or_else(|| "<script>".to_string());

        let storage_dyn: Arc<dyn VariableStorage> = self.storage.clone();
        let host_dyn: Arc<dyn HostState> = self.host.clone();

        match Runtime::new(&source_str, storage_dyn, host_dyn) {
            Ok(new_runtime) => {
                self.inner = new_runtime;
                self.base_mut()
                    .emit_signal(&StringName::from("reloaded"), &[]);
                true
            }
            Err(e) => {
                let error_msg = e.render(&path_str, &source_str);
                godot_error!("Hot reload failed:\n{}", error_msg);
                self.base_mut().emit_signal(
                    &StringName::from("reload_failed"),
                    &[Variant::from(GString::from(error_msg.as_str()))],
                );
                false
            }
        }
    }

    /// Check if source file changed and reload if needed.
    /// Called automatically by the internal Timer. Can also be called manually.
    #[func]
    fn check_for_reload(&mut self) {
        // Skip in release builds
        if !Os::singleton().is_debug_build() {
            return;
        }

        // Skip if no source path (created via from_string)
        let Some(path) = &self.source_path else {
            return;
        };

        // Check modification time
        let current_modified = FileAccess::get_modified_time(path);
        if current_modified == self.last_modified {
            return; // No change
        }

        // File changed - reload
        self.last_modified = current_modified;

        // Load fresh BobbinScript via ResourceLoader (bypasses cache)
        let Some(resource) = ResourceLoader::singleton()
            .load_ex(path)
            .type_hint("BobbinScript")
            .cache_mode(CacheMode::REPLACE)
            .done()
        else {
            godot_error!("Hot reload: Failed to load {}", path);
            return;
        };

        let Ok(script) = resource.try_cast::<BobbinScript>() else {
            godot_error!("Hot reload: {} is not a BobbinScript", path);
            return;
        };

        // Reload with new source
        let new_source = script.bind().get_source_code();
        godot_print!("Hot reload: Reloading {}", path);
        self.reload(new_source);
    }

    /// Start the hot reload polling timer (debug builds only).
    /// Called automatically by from_file(). No-op if already started or in release.
    #[func]
    fn start_hot_reload(&mut self) {
        // Skip in release builds or if no source path
        if !Os::singleton().is_debug_build() || self.source_path.is_none() {
            return;
        }

        // Skip if timer already exists
        if self.poll_timer.is_some() {
            return;
        }

        // Create and configure timer
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(0.5);
        timer.set_one_shot(false);
        timer.set_autostart(true); // Start automatically when added to tree

        // Connect timeout signal to our polling method
        let callable = self.base().callable(&StringName::from("check_for_reload"));
        timer.connect(&StringName::from("timeout"), &callable);

        // Add timer to scene tree so it can tick
        // Use call_deferred to avoid "busy setting up children" error during _ready()
        if let Some(tree) = Engine::singleton()
            .get_main_loop()
            .and_then(|ml| ml.try_cast::<SceneTree>().ok())
        {
            if let Some(mut root) = tree.get_root() {
                root.call_deferred("add_child", &[timer.to_variant()]);
                self.poll_timer = Some(timer);
            } else {
                godot_warn!("Hot reload: Could not access scene root, polling disabled");
                timer.free();
            }
        } else {
            godot_warn!("Hot reload: Could not access scene tree, polling disabled");
            timer.free();
        }
    }

    /// Stop hot reload polling and clean up timer.
    #[func]
    fn stop_hot_reload(&mut self) {
        if let Some(mut timer) = self.poll_timer.take() {
            timer.stop();
            if timer.is_inside_tree() {
                timer.queue_free();
            }
        }
    }

    #[func]
    fn advance(&mut self) {
        if let Err(e) = self.inner.advance() {
            godot_error!("advance failed: {}", e);
        }
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
        if let Err(e) = self.inner.select_choice(index as usize) {
            godot_error!("select_choice failed: {}", e);
        }
    }

    /// Get a save variable value.
    #[func]
    fn get_variable(&self, name: GString) -> Variant {
        match self.storage.get(&name.to_string()) {
            Some(value) => value_to_variant(&value),
            None => Variant::nil(),
        }
    }

    /// Set a save variable value.
    #[func]
    fn set_variable(&self, name: GString, value: Variant) {
        if let Some(val) = variant_to_value(&value) {
            self.storage.set(&name.to_string(), val);
        }
    }

    /// Get all save variables as VarDictionary.
    #[func]
    fn get_all_variables(&self) -> VarDictionary {
        let mut dict = VarDictionary::new();
        for (key, value) in self.storage.get_all() {
            dict.set(GString::from(key.as_str()), value_to_variant(&value));
        }
        dict
    }

    /// Update a host variable (game state changed).
    #[func]
    fn update_host_variable(&self, name: GString, value: Variant) {
        if let Some(val) = variant_to_value(&value) {
            self.host.update(&name.to_string(), val);
        }
    }
}
