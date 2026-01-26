<p align="center">
  <img src="branding/bobbin_logo.png" alt="Bobbin Logo" width="100" height="100">
</p>

<h1 align="center">Bobbin</h1>

<p align="center">
  A clean, readable scripting language for branching dialogue and interactive stories.<br>
  Built for game developers. Ships with first-class Godot support.
</p>

<p align="center">
  <a href="https://github.com/snowfrogdev/bobbin/actions/workflows/build-and-test.yml"><img src="https://github.com/snowfrogdev/bobbin/actions/workflows/build-and-test.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/snowfrogdev/bobbin/releases"><img src="https://img.shields.io/github/v/release/snowfrogdev/bobbin?label=Release" alt="Release"></a>
  <a href="https://godotengine.org"><img src="https://img.shields.io/badge/Godot-4.5+-478CBF?logo=godotengine&logoColor=white" alt="Godot 4.5+"></a>
  <a href="https://deepwiki.com/snowfrogdev/bobbin"><img src="https://deepwiki.com/badge.svg" alt="Ask DeepWiki"></a>
</p>

> **WARNING:** Bobbin is in early development... You should expect:
> - Some features may be missing or incomplete
> - Documentation is sparse
> - There may be breaking changes between releases
> - The API is not yet stable

## Example

```bobbin
extern player_name
extern gold
save met_merchant = false

Welcome to the Brass Lantern, {player_name}!

- Browse wares
    set met_merchant = true
    The merchant spreads out their goods.
    - Buy healing potion (10 gold)
        if gold >= 10
            temp price = 10
            You hand over {price} gold coins.
            if met_merchant
                Here's a 2 gold discount for a returning customer!
                Actually, you only owe {price - 2} gold.
        else
            You don't have enough gold.
    - Just looking
        No problem, take your time.
    Come back anytime!

- Ask about rumors
    The merchant leans in close...
    Heard there's treasure in the old ruins.

Farewell, {player_name}! You have {gold} gold remaining.
```

## Features

- **Writer-friendly syntax** — No boilerplate, just dialogue and choices
- **Smart variable scoping** — `save` persists across sessions, `temp` lives for the scene, `extern` reads from your game
- **Nested branching** — Unlimited nesting depth with automatic gather points
- **Conditional logic** — `if`/`elseif`/`else` with full expression support
- **Expressions everywhere** — Arithmetic, comparison, and logical operators in assignments and interpolations
- **String interpolation** — Embed variables and expressions in dialogue with `{variable}` or `{a + b}`
- **Rich error messages** — Rust-quality diagnostics that point to exactly what went wrong
- **Fast & lightweight** — Rust-powered runtime, instant parsing
- **Cross-platform** — Linux, Windows, macOS, and WASM (web exports)

## Godot

### Installation

1. Download the latest release from [GitHub Releases](https://github.com/snowfrogdev/bobbin/releases)
2. Extract the `addons/` folder into your Godot project root
3. Enable the plugin in **Project → Project Settings → Plugins**

### Quick Start

```gdscript
# Create runtime with extern variables
var host_state = { "player_name": "Ada" }
var runtime = BobbinRuntime.from_string_with_host(script_content, host_state)

# Main dialogue loop
while runtime.has_more():
    runtime.advance()
    print(runtime.current_line())

    if runtime.is_waiting_for_choice():
        var choices = runtime.current_choices()
        for i in choices.size():
            print("  %d. %s" % [i + 1, choices[i]])
        runtime.select_choice(0)  # Pick first choice
```

## What's Next

- Character/speaker management
- Localization support
- Event triggering/callbacks
- Visit tracking for choices

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.
