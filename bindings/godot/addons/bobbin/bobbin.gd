class_name Bobbin


## Create a new BobbinRuntime instance from a script path.
## Use this when you need multiple concurrent dialogs.
static func create(path: String) -> BobbinRuntime:
	return create_with_host(path, {})


## Create a new BobbinRuntime instance with host state.
## Host state provides extern variables that the dialogue can read.
## Hot reload is enabled automatically in debug builds.
static func create_with_host(path: String, host_state: Dictionary) -> BobbinRuntime:
	var runtime = BobbinRuntime.from_file_with_host(path, host_state)
	assert(runtime != null, "Bobbin.create_with_host() failed: " + path)
	return runtime
