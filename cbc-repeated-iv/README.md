# CBC Repeated IV demo

This demo shows the technical perspective of reusing the init vector,
just hit:
```bash
$ cargo run
```

You will see a console input.

## CBC Repeated IV

It shows how reusing the init vector can cause big problems. For example - replay attacks or gaining knowledge from new CipherTexts.

Every iteration shows how an attacker can gain useful information from new CipherTexts.

Every launch of this demo generates a unique key and init vector. But since the IV is reused every iteration, we can still gain some information about the content of the messages.
## Supported keys
- `c` - continue, proceed to next step.

- `q` - quit.

**Have fun!**



