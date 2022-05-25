# Crc integrity demo

This is a small demo that shows the issue with CRC integrity check that is used instead of HMAC.

It's a small command-line interpreter. You are an attacker that intercepts a packet between two devices.
The packet is encrypted with AES-CTR, but you know the content. Also, at the end of every packet, there is a CRC-16 digest.

Your task is to create a new packet with your custom command, but with a valid digest, and send it to the server to disable it.

You have a couple of commands to accomplish this:
```bash
  crc <hex>       # Calculate crc on a hex-string
  hex <string>    # Convert string into hex
  intercept       # Print intercepted packet
  send <hex>      # Send encrypted command to the server in a hex format
  xor <hex> <hex> # Xor two hex-strings together
```

To start, just run
```bash
$ cargo run
```

## Solution

<details>
    <summary>Spoiler warning</summary>

1. If you run the program, you would see:

   ```
   Congratulations! You've intercepted a packet with command "don't roll your own crypto"
   The packet is encrypted in CTR mode with 16-bit crc check appended after the packet:
   
   df042ab7bcce84bde97d08499cf0d13025b1880a6184d3c6d3006486
   
   Your next task: forge "detonate" command.
   Commands
   crc          Calculate crc on a hex-string
   help         Print this message or the help of the given subcommand(s)
   hex          Convert string into hex
   intercept    Print intercepted packet
   send         Send encrypted command to the server in a hex format
   xor          Xor two hex-strings together
   
   ~> 
   ```
   
   You can check that this string is indeed a valid message:
   ```
   ~> send df042ab7bcce84bde97d08499cf0d13025b1880a6184d3c6d3006486
   why not? :)
   ~> 
   ```

   Invalid message is not decrypted:
   ```
   ~> send df042ab7bcce84bde97d08499cf0d0
   error: decryption error
   ~> 
   ```
2. We know that the ciphertext is encrypted "don't roll your own crypto" string. We can craft desired "detonate" string:
    
   ```
   ~> hex "don't roll your own crypto"
   646f6e277420726f6c6c20796f7572206f776e2063727970746f
   ~> hex detonate
   6465746f6e617465
   ~> xor 6465746f6e617465 646f6e277420726f6c6c20796f7572206f776e2063727970746f
   
     6465746f6e617465 xor
     646f6e277420726f6c6c20796f7572206f776e2063727970746f = 
     000a1a481a41060a
   
   ~> xor 000a1a481a41060a df042ab7bcce84bde97d08499cf0d13025b1880a6184d3c6d3006486
   
     000a1a481a41060a xor
     df042ab7bcce84bde97d08499cf0d13025b1880a6184d3c6d3006486 = 
     df0e30ffa68f82b7

   ~> 
   ```

   Now, `df0e30ffa68f82b7` contains encrypted "detonate" string.

3. Forge crc
   ```
   ~> crc df0e30ffa68f82b7
   a2c5
   ```

4. Send combined `"detonate"` + `crc`:
   
   ```
   ~> send df0e30ffa68f82b7a2c5
   
              _.-^^---....,,--
          _--                  --_
          <                        >)
          |                         |
          \._                   _./
             ```--. . , ; .--'''
                   | |   |
                .-=||  | |=-.
                `-=#$%&%$#=-'
                   | ;  :|
   ____________.,-#%&$@%#&#~,.____________
   
   _____________ Memory dump _____________
   |Seriously, don't roll your own crypto|
   |, especially if you have no idea abou|
   |t it.                                |
   ---------------------------------------
   ```
  
</details>
