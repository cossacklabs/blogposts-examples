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

Every launch of this demo generates a unique key and init vector. Also, the button `RESTART` changes the game’s values (like the key, nonce, flag, and eavesdropped packets).
To submit the flag: press the `SUBMIT` button or hit an `ENTER` while focusing on the flag textbox.

# How to play

On the main page, there are 4 eavesdropped ciphertexts. All of them can be different in each iteration. One of them is the flag, which you should find and submit.
Also, there is a known ciphertext, which may help find the key to decrypt the flag ciphertext message and retrieve it.
In order to help people with this task - there are 2 different tabs:
- hex encoder/decoder
- xor tool tab
  When you retrieve the Flag value - pass it into the Flag input field and hit `enter`. Or press the `submit` button.

### Hint
To find correlating ciphertext to known plaintext - look at the length of the messages :)
Flag ciphertext will ALWAYS be equal to the minimum message length in order to make the challenge easier.

### Step-by-step
1. Convert plaintext to hex;
2. Find correlating ciphertext to known plaintext (you can tell one by length);
3. Xor plaintext with correlating ciphertext. This will give you a XOR CTR stream key (!!NOT THE AES ONE!!);
4. Find the smallest ciphertext and XOR it with the given key;
5. Convert the result from hex to text. Voilà, you have the flag;
6. Input the flag into the flag field. Submit it;

In order to replay - hit the `RESTART` button.
Also, you can decrypt not only flag messages but also others. Begin with the `4th` step, but choose a different ciphertext.

**Have fun!**
