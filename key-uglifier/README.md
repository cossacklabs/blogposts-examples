# Key Uglification demo

This demo shows the technical perspective of wrong key treatment,
just hit:
```bash
$ cargo run
```

You will see a console input.

## Key Uglifier

The whole demo is built while using original function names and their functionality.

It shows, how wrong key derivation from an input can cause big problems. For example - missing bits, trimming and even using default keys.

All shenanigans with key can be seen as coloured output. Also there are `Warnings` that can explain, what have gone wrong.

## Supported keys
- `[ANYSTR]` - it will derive key from this input string.

- `q` - quit.

**Have fun!**
