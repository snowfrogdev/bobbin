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
extern gold

# Commands (trigger game effects)
extern give_item(name, count)
extern play_sound(name)

Hello, {player_name}!

- Ask about wares
    set met_merchant = true
    I have potions and scrolls.
    - Buy potion (10 gold)
        if gold >= 10
            give_item("potion", 1)
            play_sound("purchase")
            You purchase the potion.
        else
            You can't afford that.
    - Never mind
        Come back anytime.

- Leave
    if met_merchant
        Safe travels, friend!
    else
        Goodbye, stranger.
```

## Using in Godot

```gdscript
# Create a runtime with host state and commands
var runtime = Bobbin.create(
    "res://dialogue/intro.bobbin",
    {},  # saved_variables (for restoring from save games)
    {    # host_state (extern variables)
        "player_name": "Hero",
        "gold": 100,
    },
    {    # commands (game effect handlers)
        "give_item": func(args): inventory.add(args[0], int(args[1])),
        "play_sound": func(args): AudioManager.play(args[0]),
    }
)

while runtime.has_more():
    if runtime.is_waiting_for_choice():
        var choices = runtime.current_choices()
        # Show choices to player, get their selection...
        runtime.select_choice(selection)
    else:
        print(runtime.current_line())
        runtime.advance()
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
