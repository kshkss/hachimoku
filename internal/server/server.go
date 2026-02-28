package server

import (
	"log"
	"net/http"
)

// Server holds the HTTP server and its dependencies.
type Server struct {
	router *http.ServeMux
}

// NewServer creates and returns a new Server instance.
func NewServer() *Server {
	return &Server{
		router: http.NewServeMux(),
	}
}

// RegisterHandler registers an HTTP handler function for a given path.
func (s *Server) RegisterHandler(path string, handler http.HandlerFunc) {
	s.router.HandleFunc(path, handler)
}

// Start starts the HTTP server on the specified address.
func (s *Server) Start(addr string) error {
	log.Printf("Server starting on %s
", addr)
	return http.ListenAndServe(addr, s.router)
}
