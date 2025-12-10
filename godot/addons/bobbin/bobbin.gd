class_name Bobbin

# Hardcoded dialog (temporary - will come from interpreter later)
static var _lines: Array[String] = [
	"Long ago, in a kingdom by the sea...",
	"There lived a young wanderer with no name.",
	"They say she came from the eastern mountains.",
	"But no one truly knew her story.",
	"Until now."
]

static var _index: int = -1


# --- Queries (return data, don't change state) ---

static func current_line() -> String:
	if _index < 0 or _index >= _lines.size():
		return ""
	return _lines[_index]


static func has_more() -> bool:
	return _index + 1 < _lines.size()


# --- Commands (change state, return nothing) ---

static func advance() -> void:
	if not has_more():
		assert(false, "Bobbin.advance() called when no more lines")
		return
	_index += 1


static func reset() -> void:
	_index = -1
