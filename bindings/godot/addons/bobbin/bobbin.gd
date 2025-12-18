class_name Bobbin


# =============================================================================
# Factory - Create independent runtime instances
# =============================================================================

## Create a new BobbinRuntime instance from a script path.
## Use this when you need multiple concurrent dialogs.
static func create(path: String) -> BobbinRuntime:
	return create_with_host(path, {})


## Create a new BobbinRuntime instance with host state.
## Host state provides extern variables that the dialogue can read.
static func create_with_host(path: String, host_state: Dictionary) -> BobbinRuntime:
	var script: BobbinScript = ResourceLoader.load(path, "BobbinScript")
	assert(script != null, "Bobbin.create_with_host() failed to load: " + path)
	if script == null:
		return null
	var runtime = BobbinRuntime.from_string_with_host(script.source_code, host_state)
	assert(runtime != null, "Bobbin.create_with_host() failed to parse: " + path)
	return runtime


# =============================================================================
# Default Runtime - Simple API for single-dialog games
# =============================================================================

static var _default: BobbinRuntime = null


# --- Commands (change state, return nothing) ---

## Start a dialog using the default runtime.
## For multiple concurrent dialogs, use create() instead.
static func start(path: String) -> void:
	_default = create(path)


## Start a dialog with host state using the default runtime.
## Host state provides extern variables that the dialogue can read.
static func start_with_host(path: String, host_state: Dictionary) -> void:
	_default = create_with_host(path, host_state)


static func advance() -> void:
	_default.advance()


static func select_choice(index: int) -> void:
	_default.select_choice(index)


# --- Queries (return data, don't change state) ---

static func current_line() -> String:
	return _default.current_line()


static func has_more() -> bool:
	return _default.has_more()


static func is_waiting_for_choice() -> bool:
	return _default.is_waiting_for_choice()


static func current_choices() -> PackedStringArray:
	return _default.current_choices()


# =============================================================================
# Variable Access - Read/write save variables and host state
# =============================================================================

## Get a save variable value from the default runtime.
## Returns null if the variable doesn't exist.
static func get_variable(name: String) -> Variant:
	if _default == null:
		return null
	return _default.get_variable(name)


## Set a save variable value on the default runtime.
static func set_variable(name: String, value: Variant) -> void:
	if _default != null:
		_default.set_variable(name, value)


## Get all save variables as a Dictionary.
static func get_all_variables() -> Dictionary:
	if _default == null:
		return {}
	return _default.get_all_variables()


## Update a host variable value on the default runtime.
## Use this when game state changes that dialogue needs to see.
static func update_host_variable(name: String, value: Variant) -> void:
	if _default != null:
		_default.update_host_variable(name, value)
