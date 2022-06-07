# CBC Repeated IV demo

This demo shows the technical perspective of reusing init vector,
just hit:
```bash
$ cargo run
```

You will see a console input.

## CBC Repeated IV

It shows, how reusing init vector can cause big problems. For example - replay attacks, or gaining knowledge from new CipherTexts.

Every iteration show how attacker can gain useful information from new CipherTexts.

Every launch of this demo generates unique keys and init vectors, but due to reusing init vector, this program shows exploitation of reusing init vector.

## Supported keys
- `c` - continue, proceed to next step.

- `q` - quit.

**Have fun!**
