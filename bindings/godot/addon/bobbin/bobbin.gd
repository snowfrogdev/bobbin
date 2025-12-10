class_name Bobbin

static var _interpreter: BobbinInterpreter = null


static func _get_interpreter() -> BobbinInterpreter:
	if _interpreter == null:
		_interpreter = BobbinInterpreter.new()
	return _interpreter


# --- Commands (change state, return nothing) ---

static func start(path: String) -> void:
	var file := FileAccess.open(path, FileAccess.READ)
	assert(file != null, "Bobbin.start() failed to open: " + path)
	if file == null:
		return
	var content := file.get_as_text()
	_get_interpreter().load_content(content)


static func advance() -> void:
	_get_interpreter().advance()


# --- Queries (return data, don't change state) ---

static func current_line() -> String:
	return _get_interpreter().current_line()


static func has_more() -> bool:
	return _get_interpreter().has_more()
