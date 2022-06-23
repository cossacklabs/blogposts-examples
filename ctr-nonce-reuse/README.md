# CTR Nonce Reuse demo

This demo is a little game that utilises technical issues of reusing the nonce in CTR mode,
just hit:
```bash
$ cargo run --release
```
You will see a window with 3 tabs:
- main tab with game. It has 4 eavesdropped packets. Those are ciphertexts. One of them is the ciphertext of the FLAG value. Also, there is a hint with the plaintext of one of 3 eavesdropped packets.
- xor tool tab. It helps in XORing two different hex values;
- hex encoder/decoder. It helps to convert text into ASCII and vice versa.

## CTR Nonce Reuse demo

Every launch of this demo generates a unique key and init vector. Also, the button `RESTART` changes the game’s values (like the key, nonce, flag, and eavesdropped packets).
To submit the flag: press the `SUBMIT` button or hit an `ENTER` while focusing on the flag textbox.

**Have fun!**
 
