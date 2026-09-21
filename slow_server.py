import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

class SlowHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        time.sleep(2)
        self.send_response(200)
        self.end_headers()
        self.wfile.write(b"slow response\n")

ThreadingHTTPServer(("127.0.0.1", 9001), SlowHandler).serve_forever()