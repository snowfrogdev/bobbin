class_name TestCutscene
extends Control

signal cutscene_finished

@export var continue_text: String = "[ Click or press any key to continue ]"

@onready var _dialog_panel: PanelContainer = $DialogPanel
@onready var _dialog_label: RichTextLabel = $DialogPanel/MarginContainer/VBoxContainer/DialogLabel
@onready var _continue_indicator: Label = $DialogPanel/MarginContainer/VBoxContainer/ContinueIndicator
@onready var _choices_container: VBoxContainer = $DialogPanel/MarginContainer/VBoxContainer/ChoicesContainer
@onready var _status_label: Label = $StatusPanel/VBoxContainer/StatusLabel
@onready var _add_gold_btn: Button = $StatusPanel/VBoxContainer/GoldButtons/AddGoldButton
@onready var _remove_gold_btn: Button = $StatusPanel/VBoxContainer/GoldButtons/RemoveGoldButton
@onready var _add_rep_btn: Button = $StatusPanel/VBoxContainer/ReputationButtons/AddRepButton
@onready var _remove_rep_btn: Button = $StatusPanel/VBoxContainer/ReputationButtons/RemoveRepButton

# Host state - game-provided variables (extern in dialogue)
var _host_state: Dictionary = {
	"player_name": "Hero",
	"player_gold": 150
}

var _runtime: BobbinRuntime
var _is_active: bool = false
var _choice_buttons: Array[Button] = []
var _selected_choice_index: int = 0
var _last_line: String = ""


func _ready() -> void:
	_continue_indicator.text = continue_text
	_continue_indicator.hide()
	_choices_container.hide()

	# Connect button signals
	_add_gold_btn.pressed.connect(_on_add_gold_pressed)
	_remove_gold_btn.pressed.connect(_on_remove_gold_pressed)
	_add_rep_btn.pressed.connect(_on_add_rep_pressed)
	_remove_rep_btn.pressed.connect(_on_remove_rep_pressed)

	# Connect dialog panel for click-to-advance
	_dialog_panel.gui_input.connect(_on_dialog_panel_input)

	_start_cutscene()


func _unhandled_input(event: InputEvent) -> void:
	if not _is_active:
		return

	# Handle choice navigation with keyboard when choices are visible
	if _runtime.is_waiting_for_choice():
		if event is InputEventKey and event.pressed and not event.echo:
			match event.keycode:
				KEY_UP, KEY_W:
					_navigate_choice(-1)
					get_viewport().set_input_as_handled()
				KEY_DOWN, KEY_S:
					_navigate_choice(1)
					get_viewport().set_input_as_handled()
				KEY_ENTER, KEY_SPACE:
					_select_current_choice()
					get_viewport().set_input_as_handled()
				KEY_1, KEY_2, KEY_3, KEY_4, KEY_5, KEY_6, KEY_7, KEY_8, KEY_9:
					var index: int = event.keycode - KEY_1
					if index < _choice_buttons.size():
						_on_choice_selected(index)
						get_viewport().set_input_as_handled()
		return

	# Normal dialog advancement (keyboard only - mouse handled by dialog panel)
	if event is InputEventKey and event.pressed and not event.echo:
		_on_advance_requested()
		get_viewport().set_input_as_handled()


func _on_dialog_panel_input(event: InputEvent) -> void:
	if not _is_active or _runtime.is_waiting_for_choice():
		return

	if event is InputEventMouseButton:
		if event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
			_on_advance_requested()
			get_viewport().set_input_as_handled()


func _start_cutscene() -> void:
	_runtime = Bobbin.create_with_host("res://dialog/feature_showcase.bobbin", _host_state)

	# Connect hot reload signals (debug builds only)
	if OS.is_debug_build():
		_runtime.reloaded.connect(_on_dialogue_reloaded)
		_runtime.reload_failed.connect(_on_dialogue_reload_failed)

	_show_current_content()
	_is_active = true


func _on_dialogue_reloaded() -> void:
	print("Hot reload: Dialogue restarted from beginning")
	_show_current_content()


func _on_dialogue_reload_failed(error: String) -> void:
	push_error("Hot reload failed: " + error)


func _on_advance_requested() -> void:
	if _runtime.has_more():
		_runtime.advance()
		_show_current_content()
	else:
		_finish_cutscene()


func _show_current_content() -> void:
	_update_status_display()
	if _runtime.is_waiting_for_choice():
		_show_choices()
	else:
		_show_line()


func _update_status_display() -> void:
	var text := "=== Host State ===\n"
	text += "player_name: %s\n" % _host_state.get("player_name")
	text += "player_gold: %d\n" % _host_state.get("player_gold")
	text += "\n=== Save Variables ===\n"
	var vars := _runtime.get_all_variables()
	for key in vars:
		text += "%s: %s\n" % [key, str(vars[key])]
	_status_label.text = text


func _show_line() -> void:
	_last_line = _runtime.current_line()
	_dialog_label.text = _last_line
	_continue_indicator.show()
	_choices_container.hide()


func _show_choices() -> void:
	# Show the question line (the line shown before choices)
	_dialog_label.text = _last_line
	_continue_indicator.hide()
	_choices_container.show()

	# Clear existing buttons
	for button in _choice_buttons:
		button.queue_free()
	_choice_buttons.clear()

	# Create buttons for each choice
	var choices := _runtime.current_choices()
	for i in choices.size():
		var button := Button.new()
		button.text = choices[i]
		button.alignment = HORIZONTAL_ALIGNMENT_LEFT
		button.pressed.connect(_on_choice_selected.bind(i))
		button.focus_mode = Control.FOCUS_ALL
		button.mouse_filter = Control.MOUSE_FILTER_STOP
		_choices_container.add_child(button)
		_choice_buttons.append(button)

	# Focus first choice
	_selected_choice_index = 0
	if _choice_buttons.size() > 0:
		_choice_buttons[0].grab_focus()


func _navigate_choice(direction: int) -> void:
	if _choice_buttons.is_empty():
		return

	_selected_choice_index = wrapi(_selected_choice_index + direction, 0, _choice_buttons.size())
	_choice_buttons[_selected_choice_index].grab_focus()


func _select_current_choice() -> void:
	if _choice_buttons.is_empty():
		return
	_on_choice_selected(_selected_choice_index)


func _on_choice_selected(index: int) -> void:
	_runtime.select_choice(index)

	# Clear choice UI
	for button in _choice_buttons:
		button.queue_free()
	_choice_buttons.clear()

	# Continue showing content
	_show_current_content()


func _finish_cutscene() -> void:
	_is_active = false
	_continue_indicator.hide()
	_choices_container.hide()
	cutscene_finished.emit()
	print("Cutscene finished!")


# Button handlers for game-side variable updates
func _on_add_gold_pressed() -> void:
	_host_state["player_gold"] = _host_state.get("player_gold", 0) + 10
	_runtime.update_host_variable("player_gold", _host_state["player_gold"])
	_update_status_display()


func _on_remove_gold_pressed() -> void:
	_host_state["player_gold"] = max(0, _host_state.get("player_gold", 0) - 10)
	_runtime.update_host_variable("player_gold", _host_state["player_gold"])
	_update_status_display()


func _on_add_rep_pressed() -> void:
	var current_rep: int = _runtime.get_variable("reputation")
	_runtime.set_variable("reputation", current_rep + 10)
	_update_status_display()


func _on_remove_rep_pressed() -> void:
	var current_rep: int = _runtime.get_variable("reputation")
	_runtime.set_variable("reputation", max(0, current_rep - 10))
	_update_status_display()
