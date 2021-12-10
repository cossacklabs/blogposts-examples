#!/usr/bin/env bash
set -eo pipefail

rm -rf ca cert1 cert2-revoked ocsp-responder

# Common Name will be filled in later
SUBJ="/C=GB/ST=London/L=London/O=Global Security/OU=IT"
EXPIRE=18250 # 50 years
SKIP_EXISTING=${SKIP_EXISTING:-0} # set to 1 to avoid re-generation of existing certificates
TMPDIR="$(mktemp -d)"

mkdir ca
# Generate a new private key for CA and self-signed certificate.
# Use CA extensions from the configuration file and a different CN.
[ "$SKIP_EXISTING" = 0 ] || [ ! -f "ca/ca.crt" ] && \
openssl req -x509 \
    -newkey rsa:2048 -nodes \
    -keyout ca/ca.key.pem \
    -out ca/ca.crt.pem \
    -sha256 \
    -subj "${subj}/CN=Test CA certificate" \
    -config openssl.cnf -extensions "v3_ca" \
    -days ${EXPIRE}

# certificate database will be regenerated
rm -f index.txt

gen_cert() {
    signer="$1"
    name="$2"

    case "$name" in
        *ocsp*)
            extensions=v3_OCSP
            CN="Test leaf certificate ($name)"
            ;;
        *)
            extensions=v3_req
            CN="Test leaf certificate ($name)"
            ;;
    esac

    config=openssl.cnf
    signer_crt="$signer/$signer.crt.pem"
    signer_key="$signer/$signer.key.pem"

    [ -e "$signer/index.txt" ] || touch "$signer/index.txt"
    [ -e "$signer/serial" ] || echo 01 > "$signer/serial"

    subj="${SUBJ}/CN=$CN"
    mkdir -p "${name}"
    # create private key
    echo "==> Generating private key for ${name}" && \
    openssl genrsa -out "${name}/${name}.key.pem" 2048

    # Generate certificate signing request for a service.
    # Certificate's CN must be different from the CA's CN.
    echo "==> Generating certificate request for ${name}" && \
    openssl req -new \
        -key "${name}/${name}.key.pem" \
        -out "${name}/${name}.csr.pem" \
        -subj "$subj"

    # Sign certificate with CA private key, adding appropriate extensions.
    # The extensions issue certificate for "localhost" which validates locally
    # and avoids the need to patch /etc/hosts on the testing machines.
    echo "==> Signing certificate for ${name}" && \
    openssl ca \
        -config "$config" \
        -in "${name}/${name}.csr.pem" \
        -out "${name}/${name}.crt.pem" \
        -extensions "$extensions" \
        -batch \
        -outdir "$TMPDIR" \
        -rand_serial \
        -notext \
        -days "$EXPIRE"

    # remove .csr because we don't need it anymore
    rm -f "${name}/${name}.csr.pem"
    # set correct rights for private key
    chmod 0400 "${name}/${name}.key.pem"

    # certificates ending with "-revoked" should be revoked
    if [[ "$name" = *-revoked ]]; then
        echo "==> Revoking $name"
        openssl ca \
            -config "$config" \
            -revoke "${name}/${name}.crt.pem"
    fi
}

gen_crl() {
    signer="$1"

    config=openssl.cnf

    # generate CRL (Certificate Revocation List) signed by corresponding CA
    openssl ca \
        -gencrl \
        -config "$config" \
        -crldays "${EXPIRE}" \
        -out "$signer/crl.pem"
}

declare -a names=("cert1" "cert2-revoked" "ocsp-responder")
for name in "${names[@]}"; do
    gen_cert ca $name
done
gen_crl ca

rm ca/*.old
