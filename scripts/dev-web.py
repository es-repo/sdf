#!/usr/bin/env python3

from __future__ import annotations

import argparse
import errno
import subprocess
import sys
import threading
import time
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit


WATCHED_DIRS = ("src", "assets")
WATCHED_FILES = ("Cargo.lock", "Cargo.toml", "favicon.svg", "index.html", "scripts/build-web.sh")

LIVE_RELOAD_SCRIPT = """
<script>
(() => {
    const events = new EventSource("/__reload");
    events.onmessage = () => window.location.reload();
})();
</script>
"""


class ReloadState:
    def __init__(self) -> None:
        self.generation = 0
        self.condition = threading.Condition()

    def notify_reload(self) -> None:
        with self.condition:
            self.generation += 1
            self.condition.notify_all()


class DevServer(ThreadingHTTPServer):
    allow_reuse_address = True

    def __init__(self, server_address: tuple[str, int], handler: type[SimpleHTTPRequestHandler], state: ReloadState) -> None:
        super().__init__(server_address, handler)
        self.state = state


class LiveReloadHandler(SimpleHTTPRequestHandler):
    def end_headers(self) -> None:
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Cross-Origin-Resource-Policy", "cross-origin")
        super().end_headers()

    def do_GET(self) -> None:
        path = urlsplit(self.path).path

        if path == "/__reload":
            self.handle_reload_events()
            return

        if path in ("/", "/index.html"):
            self.handle_index()
            return

        super().do_GET()

    def handle_index(self) -> None:
        index_path = Path(self.directory) / "index.html"

        if not index_path.exists():
            self.send_error(503, "index.html does not exist yet. Wait for the first successful build.")
            return

        html = index_path.read_text(encoding="utf-8")
        if "</body>" in html:
            html = html.replace("</body>", f"{LIVE_RELOAD_SCRIPT}</body>")
        else:
            html = f"{html}{LIVE_RELOAD_SCRIPT}"

        body = html.encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def handle_reload_events(self) -> None:
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        self.end_headers()

        state = self.server.state
        last_generation = state.generation

        try:
            while True:
                with state.condition:
                    state.condition.wait_for(lambda: state.generation != last_generation)
                    last_generation = state.generation

                self.wfile.write(f"data: {last_generation}\n\n".encode("utf-8"))
                self.wfile.flush()
        except (BrokenPipeError, ConnectionResetError):
            return


def main() -> int:
    parser = argparse.ArgumentParser(description="Build, serve, and live-reload the threaded Wasm web app.")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8000)
    parser.add_argument("--interval", type=float, default=0.5, help="File polling interval in seconds.")
    parser.add_argument("--debounce", type=float, default=0.3, help="Delay after a change before rebuilding.")
    args = parser.parse_args()

    root_dir = Path(__file__).resolve().parents[1]
    dist_dir = root_dir / "dist"
    build_script = root_dir / "scripts" / "build-web.sh"
    state = ReloadState()
    handler = partial(LiveReloadHandler, directory=str(dist_dir))
    server = create_server(args.host, args.port, handler, state)

    if server is None:
        return 1

    if not run_build(root_dir, build_script) and not dist_dir.exists():
        server.server_close()
        return 1

    watcher = threading.Thread(
        target=watch_and_build,
        args=(root_dir, build_script, state, args.interval, args.debounce),
        daemon=True,
    )
    watcher.start()

    print(f"Serving {dist_dir} at http://{args.host}:{args.port}")
    print("Watching for changes. Press Ctrl+C to stop.")

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nStopping dev server.")
    finally:
        server.server_close()

    return 0


def create_server(
    host: str,
    port: int,
    handler: type[SimpleHTTPRequestHandler],
    state: ReloadState,
) -> DevServer | None:
    try:
        return DevServer((host, port), handler, state)
    except OSError as err:
        if err.errno == errno.EADDRINUSE:
            print(
                f"Port {port} is already in use. Stop the existing server or run with another port, "
                f"for example: ./scripts/dev-web.sh --port {port + 1}",
                file=sys.stderr,
            )
            return None

        raise


def watch_and_build(
    root_dir: Path,
    build_script: Path,
    state: ReloadState,
    interval: float,
    debounce: float,
) -> None:
    previous_snapshot = file_snapshot(root_dir)

    while True:
        time.sleep(interval)
        current_snapshot = file_snapshot(root_dir)

        if current_snapshot == previous_snapshot:
            continue

        time.sleep(debounce)
        current_snapshot = file_snapshot(root_dir)

        if run_build(root_dir, build_script):
            state.notify_reload()

        previous_snapshot = file_snapshot(root_dir)


def run_build(root_dir: Path, build_script: Path) -> bool:
    print(f"\nRunning {build_script.relative_to(root_dir)}")
    started_at = time.monotonic()
    result = subprocess.run([str(build_script)], cwd=root_dir)
    elapsed = time.monotonic() - started_at

    if result.returncode == 0:
        print(f"Build finished in {elapsed:.1f}s")
        return True

    print(f"Build failed after {elapsed:.1f}s; keeping the previous page loaded.", file=sys.stderr)
    return False


def file_snapshot(root_dir: Path) -> dict[str, tuple[int, int]]:
    snapshot: dict[str, tuple[int, int]] = {}

    for relative_file in WATCHED_FILES:
        add_file_snapshot(root_dir, root_dir / relative_file, snapshot)

    for relative_dir in WATCHED_DIRS:
        directory = root_dir / relative_dir
        if not directory.exists():
            continue

        for path in directory.rglob("*"):
            add_file_snapshot(root_dir, path, snapshot)

    return snapshot


def add_file_snapshot(root_dir: Path, path: Path, snapshot: dict[str, tuple[int, int]]) -> None:
    if not path.is_file():
        return

    try:
        stat = path.stat()
    except FileNotFoundError:
        return

    snapshot[str(path.relative_to(root_dir))] = (stat.st_mtime_ns, stat.st_size)


if __name__ == "__main__":
    raise SystemExit(main())
