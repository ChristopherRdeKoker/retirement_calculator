import http.server, os, sys

DIST = os.path.abspath(sys.argv[1])
PORT = int(sys.argv[2])
PREFIX = "/apps/retirement/"
CSP = open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "csp.txt")).read().strip()
TYPES = {".html": "text/html; charset=utf-8", ".js": "text/javascript", ".wasm": "application/wasm",
         ".css": "text/css", ".svg": "image/svg+xml"}

class H(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        path = self.path.split("?")[0]
        if path == "/apps/retirement":
            self.send_response(301); self.send_header("Location", PREFIX); self.end_headers(); return
        if not path.startswith(PREFIX):
            self.send_error(404); return
        rel = path[len(PREFIX):] or "index.html"
        full = os.path.abspath(os.path.join(DIST, rel))
        if not full.startswith(DIST + os.sep) or not os.path.isfile(full):
            self.send_error(404); return
        body = open(full, "rb").read()
        self.send_response(200)
        self.send_header("Content-Type", TYPES.get(os.path.splitext(full)[1], "application/octet-stream"))
        self.send_header("Content-Security-Policy", CSP)
        self.send_header("X-Content-Type-Options", "nosniff")
        self.send_header("Cache-Control", "no-store")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)
    def log_message(self, *a):
        pass

http.server.ThreadingHTTPServer(("127.0.0.1", PORT), H).serve_forever()
