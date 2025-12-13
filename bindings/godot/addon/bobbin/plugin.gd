@tool
extends EditorPlugin

const BobbinImporter = preload("res://addons/bobbin/bobbin_importer.gd")

var importer: EditorImportPlugin


func _enter_tree() -> void:
	importer = BobbinImporter.new()
	add_import_plugin(importer)


func _exit_tree() -> void:
	remove_import_plugin(importer)
	importer = null
