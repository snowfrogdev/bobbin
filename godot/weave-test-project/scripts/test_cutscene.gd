class_name TestCutscene
extends Control

signal cutscene_finished

@export var dialog_lines: Array[String] = [
	"Long ago, in a kingdom by the sea...",
	"There lived a young wanderer with no name.",
	"They say she came from the eastern mountains.",
	"But no one truly knew her story.",
	"Until now."
]

@export var continue_text: String = "[ Click or press any key to continue ]"

@onready var _dialog_label: RichTextLabel = $DialogPanel/MarginContainer/VBoxContainer/DialogLabel
@onready var _continue_indicator: Label = $DialogPanel/MarginContainer/VBoxContainer/ContinueIndicator

var _current_index: int = -1
var _is_active: bool = false


func _ready() -> void:
	_continue_indicator.text = continue_text
	_continue_indicator.hide()
	_start_cutscene()


func _unhandled_input(event: InputEvent) -> void:
	if not _is_active:
		return

	var should_advance := false

	if event is InputEventMouseButton:
		should_advance = event.pressed and event.button_index == MOUSE_BUTTON_LEFT
	elif event is InputEventKey:
		should_advance = event.pressed and not event.echo

	if should_advance:
		_advance_dialog()
		get_viewport().set_input_as_handled()


func _start_cutscene() -> void:
	_current_index = -1
	_is_active = true
	_advance_dialog()


func _advance_dialog() -> void:
	_current_index += 1

	if _current_index >= dialog_lines.size():
		_finish_cutscene()
		return

	_dialog_label.text = dialog_lines[_current_index]
	_continue_indicator.show()


func _finish_cutscene() -> void:
	_is_active = false
	_continue_indicator.hide()
	cutscene_finished.emit()
	print("Cutscene finished!")
