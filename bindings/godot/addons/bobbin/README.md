# Bobbin

A narrative scripting language for Godot.

## Writing Dialogue

```bobbin
# Save a persistent variable (survives save/load)
save met_merchant = false

# Temporary variable (runtime only)
temp mood = "friendly"

# Host variable (provided by your game)
extern player_name

Hello, {player_name}!

- Ask about wares
    set met_merchant = true
    I have potions and scrolls.

- Leave
    Safe travels!
```

## Using in Godot

```gdscript
# Create a runtime
var runtime = Bobbin.create("res://dialogue/intro.bobbin")

while runtime.has_more():
    if runtime.is_waiting_for_choice():
        var choices = runtime.current_choices()
        # Show choices to player, get their selection...
        runtime.select_choice(selection)
    else:
        print(runtime.current_line())
        runtime.advance()

# With host state (pass game variables to dialogue)
var runtime = Bobbin.create_with_host("res://dialogue/intro.bobbin", {
    "player_name": "Hero",
    "gold": 100
})
```

## Editor Settings

Bobbin uses **spaces for indentation** (tabs are not supported). Godot's script editor defaults to tabs.

To switch to spaces: **Edit → Indentation → Convert Indent to Spaces**

You can check the current indentation mode in the bottom-right corner of the editor.

## Web Export

Web builds require **multi-threading support** enabled in your export settings. Bobbin's WebAssembly binary uses threads.

In Godot's Export dialog, ensure "Thread Support" is enabled for your web export preset.

## macOS

macOS quarantines unsigned binaries downloaded from the internet. If Godot fails to load the addon, run this in Terminal from your project's `addons/bobbin/bin/` folder:

```bash
xattr -dr com.apple.quarantine *.dylib
```

This only affects developers during development. Games exported and properly signed for distribution will work without this step.

## License

See LICENSE.md. Please credit "Bobbin dialogue system by Snowfrog Studio" in your game credits.

## Links

- [GitHub Repository](https://github.com/snowfrogdev/bobbin)
- [Report Issues](https://github.com/snowfrogdev/bobbin/issues)
