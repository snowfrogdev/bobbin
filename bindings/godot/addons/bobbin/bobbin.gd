class_name Bobbin

static var _runtime: BobbinRuntime = null


# --- Commands (change state, return nothing) ---

static func start(path: String) -> void:
	var script: BobbinScript = ResourceLoader.load(path, "BobbinScript")
	assert(script != null, "Bobbin.start() failed to load: " + path)
	if script == null:
		return
	_runtime = BobbinRuntime.from_string(script.source_code)
	assert(_runtime != null, "Bobbin.start() failed to parse: " + path)


static func advance() -> void:
	_runtime.advance()


# --- Queries (return data, don't change state) ---

static func current_line() -> String:
	return _runtime.current_line()


static func has_more() -> bool:
	return _runtime.has_more()
