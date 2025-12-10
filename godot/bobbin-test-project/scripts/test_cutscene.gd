class_name TestCutscene
extends Control

signal cutscene_finished

@export var continue_text: String = "[ Click or press any key to continue ]"

@onready var _dialog_label: RichTextLabel = $DialogPanel/MarginContainer/VBoxContainer/DialogLabel
@onready var _continue_indicator: Label = $DialogPanel/MarginContainer/VBoxContainer/ContinueIndicator

var _is_active: bool = false


func _ready() -> void:
	_continue_indicator.text = continue_text
	_continue_indicator.hide()
	_start_cutscene()


func _input(event: InputEvent) -> void:
	if not _is_active:
		return

	var should_advance := false

	if event is InputEventMouseButton:
		should_advance = event.pressed and event.button_index == MOUSE_BUTTON_LEFT
	elif event is InputEventKey:
		should_advance = event.pressed and not event.echo

	if should_advance:
		_on_advance_requested()
		get_viewport().set_input_as_handled()


func _start_cutscene() -> void:
	Bobbin.reset()
	Bobbin.advance()
	_show_current_line()
	_is_active = true


func _on_advance_requested() -> void:
	if Bobbin.has_more():
		Bobbin.advance()
		_show_current_line()
	else:
		_finish_cutscene()


func _show_current_line() -> void:
	_dialog_label.text = Bobbin.current_line()
	_continue_indicator.show()


func _finish_cutscene() -> void:
	_is_active = false
	_continue_indicator.hide()
	cutscene_finished.emit()
	print("Cutscene finished!")
