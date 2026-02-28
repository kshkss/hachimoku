package main

import (
	"fmt"
	"log"
	"net/http"

	"github.com/kshkss/hachimoku/internal/handler"
	"github.com/kshkss/hachimoku/internal/server"
)

func main() {
	fmt.Println("Starting hachimoku server...")

	// Initialize the server with handlers
	s := server.NewServer()
	s.RegisterHandler("/", handler.HomeHandler) // Example route

	log.Fatal(s.Start(":8080"))
}
