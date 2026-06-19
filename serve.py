"""
StreamKit dev server — finds a free port automatically.
"""
import http.server
import socket
import os

def find_free_port(start=3001):
    for port in range(start, start + 50):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            try:
                s.bind(("127.0.0.1", port))
                return port
            except OSError:
                continue
    raise RuntimeError("No free port found in range")

os.chdir(os.path.dirname(os.path.abspath(__file__)))

port = find_free_port()
print(f"StreamKit running at http://localhost:{port}")
print(f"Dashboard:           http://localhost:{port}/src/main.html")

httpd = http.server.HTTPServer(("127.0.0.1", port), http.server.SimpleHTTPRequestHandler)
httpd.serve_forever()
