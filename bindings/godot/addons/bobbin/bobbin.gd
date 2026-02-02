class_name Bobbin


## Create a new BobbinRuntime instance from a script path.
##
## Parameters:
## - path: Path to the .bobbin script file
## - saved_variables: Dictionary of save variable values to restore (optional)
## - host_state: Dictionary of extern variable values (optional)
## - commands: Dictionary of command_name -> Callable handlers (optional)
##
## Commands dictionary example:
##   {
##       "give_gold": func(args): player.gold += int(args[0]),
##       "play_sound": func(args): AudioManager.play(args[0]),
##   }
##
## Use this when you need multiple concurrent dialogs.
## Hot reload is enabled automatically in debug builds.
static func create(
	path: String,
	saved_variables: Dictionary = {},
	host_state: Dictionary = {},
	commands: Dictionary = {}
) -> BobbinRuntime:
	var runtime = BobbinRuntime.from_file_with_config(path, saved_variables, host_state, commands)
	if runtime == null:
		push_error("Bobbin.create() failed: " + path)
		return null
	return runtime


## Create a BobbinRuntime from a config dictionary.
##
## Config keys:
## - saved_variables: Dictionary of save variable values to restore
## - host_state: Dictionary of extern variable values
## - commands: Dictionary of command_name -> Callable handlers
##
## Example:
##   var runtime = Bobbin.create_from_config("res://dialogue/merchant.bobbin", {
##       "saved_variables": SaveManager.get_dialogue_vars("merchant"),
##       "host_state": {
##           "player_name": player.name,
##           "gold": player.gold,
##       },
##       "commands": {
##           "give_gold": func(args): player.gold += int(args[0]),
##           "play_sound": func(args): AudioManager.play(args[0]),
##       }
##   })
static func create_from_config(path: String, config: Dictionary = {}) -> BobbinRuntime:
	var saved_variables = config.get("saved_variables", {})
	var host_state = config.get("host_state", {})
	var commands = config.get("commands", {})
	return create(path, saved_variables, host_state, commands)


## DEPRECATED: Use create() with named parameters instead.
## This method exists for backwards compatibility.
static func create_with_host(path: String, host_state: Dictionary) -> BobbinRuntime:
	push_warning("Bobbin.create_with_host() is deprecated. Use Bobbin.create(path, {}, host_state) instead.")
	return create(path, {}, host_state, {})
