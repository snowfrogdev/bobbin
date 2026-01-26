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

	# Test 5: Arithmetic expression
	var runtime3 = BobbinRuntime.from_string("Result: {10 + 5}")
	runtime3.advance()
	var line3 = runtime3.current_line()
	if line3 != "Result: 15":
		printerr("FAIL: arithmetic expression returned: " + line3)
		quit(1)
	print("PASS: arithmetic expression")

	# Test 6: Comparison expression
	var runtime4 = BobbinRuntime.from_string("Check: {10 > 5}")
	runtime4.advance()
	var line4 = runtime4.current_line()
	if line4 != "Check: true":
		printerr("FAIL: comparison expression returned: " + line4)
		quit(1)
	print("PASS: comparison expression")

	# Test 7: Logical expression
	var runtime5 = BobbinRuntime.from_string("Logic: {true and false}")
	runtime5.advance()
	var line5 = runtime5.current_line()
	if line5 != "Logic: false":
		printerr("FAIL: logical expression returned: " + line5)
		quit(1)
	print("PASS: logical expression")

	# Test 8: Conditional execution (if true)
	var runtime6 = BobbinRuntime.from_string("if true\n    Yes!")
	runtime6.advance()
	var line6 = runtime6.current_line()
	if line6 != "Yes!":
		printerr("FAIL: conditional (if true) returned: " + line6)
		quit(1)
	print("PASS: conditional execution (if true)")

	# Test 9: Conditional execution (if false, else)
	var runtime7 = BobbinRuntime.from_string("if false\n    No!\nelse\n    Yes!")
	runtime7.advance()
	var line7 = runtime7.current_line()
	if line7 != "Yes!":
		printerr("FAIL: conditional (if/else) returned: " + line7)
		quit(1)
	print("PASS: conditional execution (if/else)")

	# Test 10: Expression in set statement
	var runtime8 = BobbinRuntime.from_string("temp x = 10\nset x = x + 5\nValue: {x}")
	runtime8.advance()
	var line8 = runtime8.current_line()
	if line8 != "Value: 15":
		printerr("FAIL: expression in set returned: " + line8)
		quit(1)
	print("PASS: expression in set statement")

	print("=== All smoke tests passed ===")
	quit(0)
