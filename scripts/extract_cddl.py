#!/usr/bin/env python3
"""Extract CDDL definitions from the WebDriver BiDi spec source (index.bs).

Python stdlib port of w3c/webdriver-bidi scripts/cddl/generate.js, so no
Node/parse5 install is needed.

Usage: extract_cddl.py <path/to/index.bs> <output-dir>

Writes local.cddl, remote.cddl and all.cddl into <output-dir>.
"""

import os
import sys
from html.parser import HTMLParser


class CddlExtractor(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self._module = None
        self._buf = []
        self.entries = {"local": [], "remote": [], "all": []}

    def handle_starttag(self, tag, attrs):
        if tag != "pre":
            return
        attrs = dict(attrs)
        if "cddl" in (attrs.get("class") or ""):
            self._module = attrs.get("data-cddl-module") or ""
            self._buf = []

    def handle_endtag(self, tag):
        if tag != "pre" or self._module is None:
            return
        content = "".join(self._buf)
        is_local = "local-cddl" in self._module
        is_remote = "remote-cddl" in self._module
        if is_local:
            self.entries["local"].append(content)
        if is_remote:
            self.entries["remote"].append(content)
        if is_local or is_remote:
            self.entries["all"].append(content)
        self._module = None

    def handle_data(self, data):
        if self._module is not None:
            self._buf.append(data)


def dedent(entry):
    lines = entry.split("\n")
    indents = [len(l) - len(l.lstrip()) for l in lines if l.strip()]
    n = min(indents) if indents else 0
    return "\n".join(l[n:] for l in lines)


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    spec, out_dir = sys.argv[1], sys.argv[2]
    parser = CddlExtractor()
    with open(spec, encoding="utf-8") as f:
        parser.feed(f.read())
    for name, entries in parser.entries.items():
        if not entries:
            sys.exit(f"no {name} CDDL blocks found in {spec}")
        path = os.path.join(out_dir, f"{name}.cddl")
        with open(path, "w", encoding="utf-8") as f:
            f.write("\n".join(dedent(e) for e in entries).strip("\n") + "\n")
        print(f"wrote {path} ({len(entries)} blocks)")


if __name__ == "__main__":
    main()
