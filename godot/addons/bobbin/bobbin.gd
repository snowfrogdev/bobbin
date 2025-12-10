class_name Bobbin

static var _lines: Array[String] = []
static var _index: int = 0


# --- Commands (change state, return nothing) ---

static func start(path: String) -> void:
	var file := FileAccess.open(path, FileAccess.READ)
	assert(file != null, "Bobbin.start() failed to open: " + path)
	if file == null:
		return

	_lines.clear()
	while not file.eof_reached():
		var line := file.get_line()
		if line.strip_edges() != "":
			_lines.append(line)

	_index = 0


static func advance() -> void:
	if not has_more():
		assert(false, "Bobbin.advance() called when no more lines")
		return
	_index += 1


# --- Queries (return data, don't change state) ---

static func current_line() -> String:
	if _index < 0 or _index >= _lines.size():
		return ""
	return _lines[_index]


static func has_more() -> bool:
	return _index + 1 < _lines.size()
