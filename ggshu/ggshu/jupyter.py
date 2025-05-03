"""Shu constructor for opening an iframe inside a jupyter lab.

This allows showing the map and then interacting with it by loading maps
and metabolic data directly from python.
"""

import functools
import json
import mimetypes
import pathlib
import re
import socket
import sys
import tempfile
import threading
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from typing import Union
from urllib.parse import urljoin

import requests
from IPython.display import IFrame, Javascript, display

# MIME for WASM (older Python versions lack this)
mimetypes.add_type("application/wasm", ".wasm")

_LINK_RE = re.compile(
    r"""
        ["']                               # opening quote
        (?P<url>
            (?:\./|\../|/)*                # optional ./  ../  /  sequences
            [^"'<>\\]+?                    # anything except quotes / angle brackets
            \.
            (?:wasm|js|json|png|jpe?g|svg|gif|css|ttf|woff2?)  # extensions
        )
        ["']                               # closing quote
    """,
    re.I | re.X,
)
_ASSETS_MANIFEST = [
    # top‑level assets
    "arrow_grad.png",
    "arrow_grad_Layer 1.png",
    "gauss.png",
    "hist_legend.png",
    "hist_legend_right.png",
    "hover.png",
    "met_grad.png",
    "met_grad_Layer 1.png",
    "met_handle.png",
    "mockup_legend.svg",
    "rect_legend.png",
    "actual_ecoli.json",
    "ecoli_core_map.json",
    "hand_ecoli_map.json",
    # fonts
    "fonts/Assistant-LICENSE",
    "fonts/Assistant-Light.ttf",
    "fonts/Assistant-Regular.ttf",
    "fonts/Assistant-Regular.tttx",
    "fonts/FiraMono-LICENSE",
    "fonts/FiraMono-Medium.ttf",
    "fonts/FiraSans-Bold.ttf",
    "fonts/FiraSans-Bold.tttx",
]


class Shu:
    """View the Shu WebAssembly app inside a Jupyter notebook.

    It provides an 'offline' mode where the WASM is downloaded
    and then can be interacted with through python to load maps
    and data.

    There is also a non-offline that only works for interacting
    through the UI.

    Example
    -------
    ```python
    import json
    from ggshu import Shu

    view = Shu(height=740)
    view.show()
    with open("ecoli_core_map.json") as f_map:
        ecoli_map = json.load(f_map)
    view.load_map(ecoli_map)
    with open("omics.metabolism.json") as f_data:
        ecoli_data = json.load(f_data)
    view.load_data(ecoli_data)
    ```
    """

    def __init__(
        self,
        source_url: str = "https://biosustain.github.io/shu/",
        workdir: Union[str, pathlib.Path, None] = None,
        height: int = 740,
    ):
        self.source_url = source_url.rstrip("/") + "/"
        self.height = height
        self._base = (
            pathlib.Path(workdir).expanduser().resolve()
            if workdir
            else pathlib.Path(tempfile.mkdtemp(prefix="shu_"))
        )
        self._local_index = self._base / "index.html"
        self._server = None

    def close(self):
        """Stop the local web‑server."""
        if self._server:
            self._server.shutdown()
            self._server.server_close()
            self._server = None
            print("ShuViewer server stopped.", file=sys.stderr)

    def show(self, offline: bool = True) -> None:
        """Display the Shu app."""
        if offline:
            if not self._local_index.exists():
                self._download_site()
            self._start_server()
            src = self._http_url
        else:
            src = self.source_url

        display(self._iframe(src))

    def _start_server(self) -> None:
        """Launch a silent ThreadingHTTPServer that serves `self._base`.

        Ensures WASM is sent with the correct MIME.
        """
        if self._server:
            return

        # pick an unused port
        with socket.socket() as s:
            s.bind(("127.0.0.1", 0))
            port = s.getsockname()[1]

        class Quiet(SimpleHTTPRequestHandler):
            extensions_map = {
                **SimpleHTTPRequestHandler.extensions_map,
                ".wasm": "application/wasm",
                ".js": "application/javascript",
            }

            def log_message(self, *_):  # suppress console spam in notebooks
                pass

        handler = functools.partial(Quiet, directory=str(self._base))
        self._server = ThreadingHTTPServer(("127.0.0.1", port), handler)
        threading.Thread(target=self._server.serve_forever, daemon=True).start()
        self._http_url = f"http://127.0.0.1:{port}/index.html"

    def load_map(self, data: dict) -> None:
        """Load the map in the running Shu app (offline mode).

        Works after `show(offline=True)`.
        """
        self._load_data(data, "shu_load_map")

    def load_data(self, data: dict) -> None:
        """Load metabolic data in the running Shu app (offline mode).

        Works after `show(offline=True)`.
        """
        self._load_data(data, "shu_load_data")

    def download(self, to: Union[str, pathlib.Path, None] = None) -> pathlib.Path:
        """Force a fresh snapshot to *to* (or tmp dir)."""
        if to:
            self._base = pathlib.Path(to).expanduser().resolve()
            self._base.mkdir(parents=True, exist_ok=True)
            self._local_index = self._base / "index.html"
        self._download_site(force=True)
        return self._base

    def _load_data(self, data: dict, event: str):
        """Load data/maps in the running Shu app (offline mode).

        Works after `show(offline=True)`.
        """
        if self._server is None:
            raise RuntimeError(
                "You must start the `Shu` viewer in offline=False if you want "
                "to interact with it programatically. "
                "Call show(offline=True) first."
            )
        assert event in ["shu_load_data", "shu_load_map"], "Internal logic error"

        json_text = json.dumps(data, ensure_ascii=False)
        js_json_literal = json.dumps(json_text)

        js_code = f"""
        (async () => {{
            /* find the iframe showing index.html */
            const frame = [...document.querySelectorAll('iframe')]
                          .reverse()
                          .find(f => f.src.includes("index.html"));
            if (!frame) {{
                console.warn("[Shu] iframe not found");
                return;
            }}

            /* wait until the iframe has really loaded */
            if (frame.contentWindow === null) {{
                await new Promise(ok => frame.addEventListener("load", ok, {{once:true}}));
            }}

            /* send the JSON text (string) via postMessage */
            frame.contentWindow.postMessage({{
                type:    "{event}",
                payload: {js_json_literal}
            }}, "*");
            console.log("[Shu] map posted");
        }})();"""
        display(Javascript(js_code))

    def _iframe(self, src: str) -> IFrame:
        return IFrame(
            src,
            width="100%",
            height=self.height,
            sandbox="allow-scripts allow-same-origin",
        )

    def _download_site(self, force: bool = False) -> None:
        """Mirror every file under self.source_url into self._base, *then*
        guarantee that everything in `assets/` is present.
        """
        if self._local_index.exists() and not force:
            return

        base = self.source_url  # e.g. https://biosustain.github.io/shu/
        print(f"Downloading Shu from {base}…", file=sys.stderr)

        todo, seen = {base}, set()
        with requests.Session() as ses:
            while todo:
                url = todo.pop()
                if url in seen:
                    continue
                seen.add(url)

                r = ses.get(url, timeout=30)
                if r.status_code >= 400:
                    print("skipping", url, r.status_code, file=sys.stderr)
                    continue

                rel_url = url[len(base) :]  # '' for landing dir
                target = self._base / rel_url
                if url.endswith("/"):
                    target.mkdir(parents=True, exist_ok=True)
                    target = target / "index.html"
                else:
                    target.parent.mkdir(parents=True, exist_ok=True)

                target.write_bytes(r.content)

                # scrape HTML / JS / CSS for more links
                if target.suffix.lower() in {".html", ".js", ".css"}:
                    text = r.text
                    for m in _LINK_RE.finditer(text):
                        link = m.group("url")
                        if link.startswith(("http://", "https://", "data:", "mailto:")):
                            continue

                        # decide how to resolve
                        if link.startswith(("/", "assets/", "pkg/")):
                            abs_link = urljoin(self.source_url, link.lstrip("/"))
                        else:
                            abs_link = urljoin(url, link)

                        todo.add(abs_link)

            assets_root = self._base / "assets"
            assets_root.mkdir(exist_ok=True)

            # TODO: harcoded, should be properly scraped
            for rel_path in _ASSETS_MANIFEST:
                url = urljoin(self.source_url, f"assets/{rel_path}")
                dest = assets_root / rel_path
                if dest.exists():
                    continue  # already grabbed by crawler

                # print("→ assets:", url, file=sys.stderr)
                r = ses.get(url, timeout=30)
                if r.status_code == 200:
                    dest.parent.mkdir(parents=True, exist_ok=True)
                    dest.write_bytes(r.content)
                else:
                    print("404", file=sys.stderr)

        # patch absolute → relative in root index.html
        idx = self._local_index
        html = idx.read_text(encoding="utf8")
        html = html.replace('src="/', 'src="').replace('href="/', 'href="')

        # allow programmatic access (instead of clicking the buttons)
        inject = """
        <!-- inserted by ShuViewer -->
        <script>
        window.addEventListener("message", ev => {
            if (ev.data && ev.data.type === "shu_load_map") {
                const data = ev.data.payload;              // already JSON–serialised
                const blob = new Blob([data], {type:"application/json"});
                const file = new File([blob], "map.json", {type:"application/json"});
                const dt   = new DataTransfer();
                dt.items.add(file);
                const input = document.getElementById("fileb");
                if (input) {
                    input.files = dt.files;
                    input.dispatchEvent(new Event("change", {bubbles:true}));
                } else {
                    console.warn("[Shu] #fileb not present yet");
                }
            }
            if (ev.data?.type === "shu_load_data") {
                const blob = new Blob([ev.data.payload], {type:"application/json"});
                const file = new File([blob], "data.json", {type:"application/json"});
                const dt   = new DataTransfer();
                dt.items.add(file);
                const input = document.getElementById("fileData");
                if (input) {
                    input.files = dt.files;
                    input.dispatchEvent(new Event("change", {bubbles:true}));
                }
            }
        });
        </script>
        """

        if "shu_load_map" not in html:
            html = html.replace("</body>", inject + "\n</body>")

        idx.write_text(html, encoding="utf8")
        print("Snapshot ready at", self._base, file=sys.stderr)

    def __repr__(self) -> str:
        return f"<Shu base='{self._base}'>"
