class_name Bobbin


# =============================================================================
# Factory - Create independent runtime instances
# =============================================================================

## Create a new BobbinRuntime instance from a script path.
## Use this when you need multiple concurrent dialogs.
static func create(path: String) -> BobbinRuntime:
	var script: BobbinScript = ResourceLoader.load(path, "BobbinScript")
	assert(script != null, "Bobbin.create() failed to load: " + path)
	if script == null:
		return null
	var runtime = BobbinRuntime.from_string(script.source_code)
	assert(runtime != null, "Bobbin.create() failed to parse: " + path)
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
