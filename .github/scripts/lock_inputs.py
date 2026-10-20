"""Whether a change to Cargo.lock reaches the code generator or the server.

The lockfile is shared by the whole workspace, so a change to it says nothing
on its own about which jobs it can break. This prints `codegen=true|false`
and `apple=true|false` for the change between two commits:

- codegen: a package xtask depends on, directly or not, was added, removed or
  changed. The generated client is a function of xtask and the spec alone.
- apple: the same for tfl-mcp, the binary the macOS job builds. The lock lists
  every target's dependencies, so the macOS-only crates (the platform TLS
  verifier's Security.framework bindings, for one) are in that closure too.

    python3 lock_inputs.py BASE HEAD
"""

import subprocess
import sys
import tomllib


def show(rev, path):
    out = subprocess.run(["git", "show", f"{rev}:{path}"], capture_output=True, text=True)
    return out.stdout if out.returncode == 0 else ""


def packages(text):
    return tomllib.loads(text).get("package", []) if text else []


def reaches(old, new, root):
    def key(p):
        return (p["name"], p["version"])

    before = {key(p): p for p in packages(old)}
    after = {key(p): p for p in packages(new)}
    changed = {k[0] for k in before.keys() ^ after.keys()}
    changed |= {k[0] for k in before.keys() & after.keys() if before[k] != after[k]}
    if not changed:
        return False
    graph = {}
    for p in [*before.values(), *after.values()]:
        graph.setdefault(p["name"], set()).update(d.split()[0] for d in p.get("dependencies", []))
    seen, stack = set(), [root]
    while stack:
        name = stack.pop()
        if name not in seen:
            seen.add(name)
            stack.extend(graph.get(name, ()))
    return bool(changed & seen)


def main():
    base, head = sys.argv[1:3]
    old, new = show(base, "Cargo.lock"), show(head, "Cargo.lock")
    print(f"codegen={str(reaches(old, new, 'xtask')).lower()}")
    print(f"apple={str(reaches(old, new, 'tfl-mcp')).lower()}")


main()
