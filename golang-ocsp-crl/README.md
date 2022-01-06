## Preparation

We will use `openssl` to generate certificates, create CRL, run OCSP server and so on.
Clone the repo and run `./generate.sh`.

As a result you will have:
* CA certificate/key in `ca` directory
* Valid certificate/key in `cert1` directory
* Revoked certificate/key in `cert2-revoked`
* CRL containing revoked certificate serial, `ca/crl.pem`
* Dedicated certificate/key for OCSP responder (can be used instead of CA ones) in `ocsp-responder`

## Launching OCSP responder

With `openssl` it's as easy as
```
openssl ocsp \
     -index ca/index.txt \
     -CA ca/ca.crt.pem \
     -rsigner ocsp-responder/ocsp-responder.crt.pem \
     -rkey ocsp-responder/ocsp-responder.key.pem \
     -port 8888 \
     -ignore_err
```

you can even query it using another `openssl` command from a different console:
```
openssl ocsp -issuer ca/ca.crt.pem -cert cert1/cert1.crt.pem -url http://127.0.0.1:8888
```
just replace `cert1` with `cert2-revoked` and you will see a different response.

## Running example applications

### Run the server

Run with a valid certificate (should expect a successful connection on client):
```
go run ./cmd/tls-server -cert cert1/cert1.crt.pem -key cert1/cert1.key.pem
```

Run with a revoked certificate (should expect TLS handshake error on client side, but only of OCSP or CRL validation was enabled):
```
go run ./cmd/tls-server \
    -cert cert2-revoked/cert2-revoked.crt.pem \
    -key cert2-revoked/cert2-revoked.key.pem
```
### Run the client

`go run ./cmd/tls-client -ca_cert ca/ca.crt.pem`

Add `-use_ocsp` or `-crl_file ca/crl.pem` to enable corresponding checks. OCSP server should be running for the `-use_ocsp`, see the command above.

The result of running this command will depend on these additional flags and the certificate used by the server.
