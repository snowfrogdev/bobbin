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
# Simple API (single dialogue)
Bobbin.start("res://dialogue/intro.bobbin")

while Bobbin.has_more():
    if Bobbin.is_waiting_for_choice():
        var choices = Bobbin.current_choices()
        # Show choices to player, get their selection...
        Bobbin.select_choice(selection)
    else:
        print(Bobbin.current_line())
        Bobbin.advance()

# With host state (pass game variables to dialogue)
Bobbin.start_with_host("res://dialogue/intro.bobbin", {
    "player_name": "Hero",
    "gold": 100
})

# Create independent runtimes for multiple concurrent dialogues
var runtime = Bobbin.create("res://dialogue/npc.bobbin")
```

## Editor Settings

Bobbin uses **spaces for indentation** (tabs are not supported). Godot's script editor defaults to tabs.

To switch to spaces: **Edit → Indentation → Convert Indent to Spaces**

You can check the current indentation mode in the bottom-right corner of the editor.

## Web Export

Web builds require **multi-threading support** enabled in your export settings. Bobbin's WebAssembly binary uses threads.

In Godot's Export dialog, ensure "Thread Support" is enabled for your web export preset.

## License

See LICENSE.md. Please credit "Bobbin dialogue system by Snowfrog Studio" in your game credits.

## Links

- [GitHub Repository](https://github.com/snowfrogdev/bobbin)
- [Report Issues](https://github.com/snowfrogdev/bobbin/issues)
