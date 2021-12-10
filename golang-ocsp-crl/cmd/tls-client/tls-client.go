package main

import (
	"bytes"
	"crypto"
	"crypto/tls"
	"crypto/x509"
	"flag"
	"fmt"
	"golang.org/x/crypto/ocsp"
	"io/ioutil"
	"net/http"
	"net/url"
	"time"
)

var (
	flagUseOCSP bool
	flagCRLFile string
)

func main() {
	flagCACert := flag.String("ca_cert", "ca/ca.crt.pem", "Root CA certificate")
	flag.BoolVar(&flagUseOCSP, "use_ocsp", false, "Use OCSP to validate server certificate")
	flag.StringVar(&flagCRLFile, "crl_file", "", "Use CRL to validate server certificate (pass the file name of the CRL)")
	flag.Parse()

	pemCert, err := ioutil.ReadFile(*flagCACert)
	if err != nil {
		fmt.Printf("Cannot read root CA certificate: %v\n", err)
		return
	}
	caCerts := x509.NewCertPool()
	ok := caCerts.AppendCertsFromPEM(pemCert)
	if !ok {
		fmt.Printf("Cannot add CA certificate to list of certificates\n")
		return
	}
	config := &tls.Config{
		RootCAs: caCerts,
		VerifyPeerCertificate: verifyPeerCertificate,
	}

	conn, err := tls.Dial("tcp", "localhost:12345", config)
	if err != nil {
		fmt.Printf("Cannot connect to server: %v\n", err)
		return
	}

	fmt.Printf("Successfully connected to server\n")

	buf := make([]byte, 100)
	n, err := conn.Read(buf)
	if err != nil {
		fmt.Println(string(buf[:n]))
	}

	conn.Close()
}

func verifyPeerCertificate(rawCerts [][]byte, verifiedChains [][]*x509.Certificate) error {
	cert := verifiedChains[0][0]
	issuerCert := verifiedChains[0][1]

	fmt.Printf("Server certificate: %s\n", cert.Subject.CommonName)
	fmt.Printf("Server certificate issuer: %s\n", issuerCert.Subject.CommonName)

	if flagUseOCSP {
		fmt.Printf("Validating server certificate with OCSP (serial: %s)\n", cert.SerialNumber.String())

		ocspResponse, err := QueryOCSP(cert.Subject.CommonName, cert, issuerCert, cert.OCSPServer[0])
		if err != nil {
			return err
		}
		switch ocspResponse.Status {
		case ocsp.Good:
			fmt.Printf("[+] Certificate status is Good\n")
		case ocsp.Revoked:
			fmt.Printf("[-] Certificate status is Revoked\n")
			return fmt.Errorf("The certificate was revoked!")
		case ocsp.Unknown:
			fmt.Printf("[-] Certificate status is Unknown\n")
			return fmt.Errorf("The certificate is unknown to OCSP server!")
		}
	}

	if flagCRLFile != "" {
		fmt.Printf("Validating server certificate with CRL (serial: %s)\n", cert.SerialNumber.String())

		err := QueryCRL(flagCRLFile, cert, issuerCert)
		if err != nil {
			return err;
		}
	}

	fmt.Printf("Server certificate was allowed\n")
	return nil
}

func QueryOCSP(commonName string, cert, issuerCert *x509.Certificate, ocspServerURL string) (*ocsp.Response, error) {
	fmt.Printf("[*] Crafting an OCSP request\n")
	opts := &ocsp.RequestOptions{Hash: crypto.SHA256}
	buffer, err := ocsp.CreateRequest(cert, issuerCert, opts)
	if err != nil {
		return nil, err
	}
	fmt.Printf("[*] Preparing HTTP request to OCSP server\n")
	httpRequest, err := http.NewRequest(http.MethodPost, ocspServerURL, bytes.NewBuffer(buffer))
	if err != nil {
		return nil, err
	}
	ocspURL, err := url.Parse(ocspServerURL)
	if err != nil {
		return nil, err
	}
	httpRequest.Header.Add("Content-Type", "application/ocsp-request")
	httpRequest.Header.Add("Accept", "application/ocsp-response")
	httpRequest.Header.Add("host", ocspURL.Host)
	httpClient := &http.Client{}
	fmt.Printf("[*] Launching HTTP request to OCSP server\n")
	httpResponse, err := httpClient.Do(httpRequest)
	if err != nil {
		return nil, err
	}
	defer httpResponse.Body.Close()
	output, err := ioutil.ReadAll(httpResponse.Body)
	if err != nil {
		return nil, err
	}
	fmt.Printf("[*] Parsing OCSP server response\n")
	ocspResponse, err := ocsp.ParseResponseForCert(output, cert, issuerCert)
	return ocspResponse, err
}

func QueryCRL(crlFilename string, cert *x509.Certificate, issuerCert *x509.Certificate) error {
	fmt.Printf("[*] Reading %s\n", crlFilename)
	rawCRL, err := ioutil.ReadFile(crlFilename)
	if err != nil {
		return err
	}

	fmt.Printf("[*] Parsing %s\n", crlFilename)
	crl, err := x509.ParseCRL(rawCRL)
	if err != nil {
		return err
	}

	fmt.Printf("[*] Checking CRL signature\n")
	err = issuerCert.CheckCRLSignature(crl)
	if err != nil {
		return err
	}

	fmt.Printf("[*] Checking CRL validity\n")
	if crl.TBSCertList.NextUpdate.Before(time.Now()) {
		return fmt.Errorf("CRL is outdated")
	}

	fmt.Printf("[*] Searching for our certificate\n")
	for _, revokedCertificate := range crl.TBSCertList.RevokedCertificates {
		fmt.Printf("[*] Revoked certificate serial: %s\n", revokedCertificate.SerialNumber.String())
		if revokedCertificate.SerialNumber.Cmp(cert.SerialNumber) == 0 {
			fmt.Printf("[-] Found validated certificate in list of revoked ones\n")
			return fmt.Errorf("The certificate was revoked!")
		}
	}

	fmt.Printf("[+] Did not find validated certificate among revoked ones\n")

	return nil
}
