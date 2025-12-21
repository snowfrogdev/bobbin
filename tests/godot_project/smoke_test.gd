extends SceneTree

func _init():
	print("=== Bobbin Smoke Test ===")

	# Test 1: Create runtime from string
	var runtime = BobbinRuntime.from_string("Hello, World!")
	if runtime == null:
		printerr("FAIL: from_string() returned null")
		quit(1)
	print("PASS: from_string() created runtime")

	# Test 2: Advance and get line
	runtime.advance()
	var line = runtime.current_line()
	if line != "Hello, World!":
		printerr("FAIL: current_line() returned: " + line)
		quit(1)
	print("PASS: current_line() returned correct text")

	# Test 3: Check has_more
	if runtime.has_more():
		printerr("FAIL: has_more() should be false")
		quit(1)
	print("PASS: has_more() returned false")

	# Test 4: Variable operations
	var runtime2 = BobbinRuntime.from_string("save counter = 42")
	runtime2.advance()
	var val = runtime2.get_variable("counter")
	if val != 42:
		printerr("FAIL: get_variable() returned: " + str(val))
		quit(1)
	print("PASS: get_variable() returned correct value")

	print("=== All smoke tests passed ===")
	quit(0)
