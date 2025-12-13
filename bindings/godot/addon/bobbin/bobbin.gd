class_name Bobbin

static var _runtime: BobbinRuntime = null


# --- Commands (change state, return nothing) ---

static func start(path: String) -> void:
	var file := FileAccess.open(path, FileAccess.READ)
	assert(file != null, "Bobbin.start() failed to open: " + path)
	if file == null:
		return
	var content := file.get_as_text()
	_runtime = BobbinRuntime.from_string(content)
	assert(_runtime != null, "Bobbin.start() failed to parse: " + path)


static func advance() -> void:
	_runtime.advance()


# --- Queries (return data, don't change state) ---

static func current_line() -> String:
	return _runtime.current_line()


static func has_more() -> bool:
	return _runtime.has_more()
