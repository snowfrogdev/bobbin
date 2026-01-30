class_name Bobbin


## Create a new BobbinRuntime instance from a script path.
## - saved_variables: Dictionary of save variable values to restore (optional)
## - host_state: Dictionary of extern variable values (optional)
##
## Use this when you need multiple concurrent dialogs.
## Hot reload is enabled automatically in debug builds.
static func create(
	path: String,
	saved_variables: Dictionary = {},
	host_state: Dictionary = {}
) -> BobbinRuntime:
	var runtime = BobbinRuntime.from_file_with_state(path, saved_variables, host_state)
	if runtime == null:
		push_error("Bobbin.create() failed: " + path)
		return null
	return runtime


## DEPRECATED: Use create() with named parameters instead.
## This method exists for backwards compatibility.
static func create_with_host(path: String, host_state: Dictionary) -> BobbinRuntime:
	push_warning("Bobbin.create_with_host() is deprecated. Use Bobbin.create(path, {}, host_state) instead.")
	return create(path, {}, host_state)
