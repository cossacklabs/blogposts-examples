package main

import (
	"crypto/tls"
	"flag"
	"fmt"
	"net"
)

func main() {
	flagCert := flag.String("cert", "cert1/cert1.crt.pem", "TLS server certificate")
	flagKey := flag.String("key", "cert1/cert1.key.pem", "TLS server key")
	flag.Parse()

	cert, err := tls.LoadX509KeyPair(*flagCert, *flagKey)
	if err != nil {
		fmt.Printf("Cannot read TLS certificate/key: %v\n", err)
		return
	}

	config := &tls.Config{
		Certificates: []tls.Certificate{cert},
	}

	listener, err := tls.Listen("tcp", "127.0.0.1:12345", config)
	if err != nil {
		fmt.Printf("Cannot create TLS listener: %v\n", err)
		return
	}
	defer listener.Close()

	for {
		fmt.Printf("Accepting connection...\n")
		conn, err := listener.Accept()
		if err != nil {
			fmt.Printf("Error accepting connection: %v\n", err)
			continue
		}
		fmt.Printf("New connection from %v\n", conn.RemoteAddr())
		go handleConnection(conn)
	}
}

func handleConnection(conn net.Conn) {
	defer conn.Close()

	_, _ = conn.Write([]byte("Hello!"))
}
