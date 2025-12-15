class_name TestCutscene
extends Control

signal cutscene_finished

@export var continue_text: String = "[ Click or press any key to continue ]"

@onready var _dialog_label: RichTextLabel = $DialogPanel/MarginContainer/VBoxContainer/DialogLabel
@onready var _continue_indicator: Label = $DialogPanel/MarginContainer/VBoxContainer/ContinueIndicator
@onready var _choices_container: VBoxContainer = $DialogPanel/MarginContainer/VBoxContainer/ChoicesContainer

var _is_active: bool = false
var _choice_buttons: Array[Button] = []
var _selected_choice_index: int = 0
var _last_line: String = ""


func _ready() -> void:
	_continue_indicator.text = continue_text
	_continue_indicator.hide()
	_choices_container.hide()
	_start_cutscene()


func _input(event: InputEvent) -> void:
	if not _is_active:
		return

	# Handle choice navigation with keyboard when choices are visible
	if Bobbin.is_waiting_for_choice():
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

	# Normal dialog advancement
	var should_advance := false

	if event is InputEventMouseButton:
		should_advance = event.pressed and event.button_index == MOUSE_BUTTON_LEFT
	elif event is InputEventKey:
		should_advance = event.pressed and not event.echo

	if should_advance:
		_on_advance_requested()
		get_viewport().set_input_as_handled()


func _start_cutscene() -> void:
	Bobbin.start("res://dialog/intro.bobbin")
	_show_current_content()
	_is_active = true


func _on_advance_requested() -> void:
	if Bobbin.has_more():
		Bobbin.advance()
		_show_current_content()
	else:
		_finish_cutscene()


func _show_current_content() -> void:
	if Bobbin.is_waiting_for_choice():
		_show_choices()
	else:
		_show_line()


func _show_line() -> void:
	_last_line = Bobbin.current_line()
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
	var choices := Bobbin.current_choices()
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
	Bobbin.select_choice(index)

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
