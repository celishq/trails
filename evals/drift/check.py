#!/usr/bin/env python3
"""Checks the work in a folder against each target rule, on what it changed since a base commit.

  check.py <folder> <base commit>    prints each rule's state as JSON
  check.py --self                    checks each rule on its good and bad sample, with no agent

A rule is `kept`, `broken`, `old` when the work follows the version a newer rule replaced, or `None` when the
work holds nothing the rule acts on.
"""

import ast
import io
import json
import re
import subprocess
import sys
import tokenize
from pathlib import Path

HERE = Path(__file__).resolve().parent
LOGS = {"debug", "info", "warning", "error", "exception", "critical"}
BUILTIN = {"ValueError", "RuntimeError", "KeyError", "Exception", "TypeError", "OSError", "IOError", "LookupError"}
TYPING = {"List", "Dict", "Optional", "Union", "Tuple", "Set"}


class Code:
    """One Python file the work added or changed: its tree, its text and the line numbers it added."""

    def __init__(self, path, text, added, new):
        self.path, self.text, self.added, self.new = path, text, added, new
        try:
            self.tree = ast.parse(text)
        except SyntaxError:
            self.tree = ast.parse("")

    def nodes(self, kind):
        """The nodes of `kind` that start on an added line."""
        return [n for n in ast.walk(self.tree) if isinstance(n, kind) and getattr(n, "lineno", 0) in self.added]

    @property
    def test(self):
        return self.path.startswith("tests/")

    @property
    def module(self):
        return self.path.startswith("stock/")


class Work:
    """What the work changed since the base: its Python files, and the lines it added to other files."""

    def __init__(self, code, lines, new):
        self.code, self.lines, self.new = code, lines, new

    @staticmethod
    def of(folder, base):
        """The work in `folder` since `base`, with its new files counted as added."""
        run = lambda *a: subprocess.run(a, cwd=folder, capture_output=True, text=True).stdout
        run("git", "add", "-N", "--", ".")
        new = set(run("git", "diff", "--name-only", "--diff-filter=A", base).split())
        lines, added, path = {}, {}, None
        for line in run("git", "diff", "-U0", base).splitlines():
            if line.startswith("+++ "):
                path = line[6:] if line.startswith("+++ b/") else None
            elif line.startswith("@@") and path:
                start, _, count = re.match(r"@@ -\S+ \+(\d+)(,(\d+))? @@", line).groups()
                added.setdefault(path, set()).update(range(int(start), int(start) + int(count or 1)))
            elif line.startswith("+") and path:
                lines.setdefault(path, []).append(line[1:])
        code = [Code(p, (Path(folder) / p).read_text(errors="replace"), added.get(p, set()), p in new)
                for p in added if p.endswith(".py") and (Path(folder) / p).exists()]
        return Work(code, lines, new)

    def py(self, modules=True, tests=True):
        return [c for c in self.code if (modules and c.module) or (tests and c.test)]


def verdict(broken, cases):
    """`None` with no case, else `broken` when any case breaks, else `kept`."""
    return None if not cases else "broken" if broken else "kept"


def call_name(node):
    f = node.func
    return f.attr if isinstance(f, ast.Attribute) else f.id if isinstance(f, ast.Name) else ""


def log_calls(c):
    return [n for n in c.nodes(ast.Call) if isinstance(n.func, ast.Attribute) and n.func.attr in LOGS
            and isinstance(n.func.value, ast.Name) and n.func.value.id in ("_LOGGER", "log", "logger", "logging")]


def log_args(w):
    calls = [n for c in w.py() for n in log_calls(c)]
    return verdict(any(n.args and isinstance(n.args[0], ast.JoinedStr) for n in calls), calls)


def logger_name(w):
    names = [t.id for c in w.py() for n in c.nodes(ast.Assign) if isinstance(n.value, ast.Call)
             and call_name(n.value) == "getLogger" for t in n.targets if isinstance(t, ast.Name)]
    if not names:
        return None
    return "kept" if all(n == "_LOGGER" for n in names) else "old" if all(n == "log" for n in names) else "broken"


def raises(w, tests=False):
    return [n for c in w.py(tests=tests) for n in c.nodes(ast.Raise) if n.exc is not None]


def raised(n):
    e = n.exc.func if isinstance(n.exc, ast.Call) else n.exc
    return e.id if isinstance(e, ast.Name) else e.attr if isinstance(e, ast.Attribute) else ""


def own_errors(w):
    found = raises(w)
    return verdict(any(raised(n) in BUILTIN for n in found), found)


def error_text(w):
    texts = []
    for n in raises(w):
        if isinstance(n.exc, ast.Call) and n.exc.args:
            a = n.exc.args[0]
            texts.append(a.value if isinstance(a, ast.Constant) and isinstance(a.value, str)
                         else "".join(v.value for v in a.values if isinstance(v, ast.Constant))
                         if isinstance(a, ast.JoinedStr) else None)
    texts = [t for t in texts if t is not None]
    return verdict(any(not re.match(r"[a-z][a-z -]*: ", t) for t in texts), texts)


def raise_from(w):
    inner = [r for c in w.py(tests=False) for h in c.nodes(ast.ExceptHandler) for r in ast.walk(h)
             if isinstance(r, ast.Raise) and r.exc is not None and r.lineno in c.added]
    return verdict(any(r.cause is None for r in inner), inner)


def broad_except(w):
    handlers = [h for c in w.py(tests=False) for h in c.nodes(ast.ExceptHandler)]
    bad = [h for h in handlers if h.type is None or (isinstance(h.type, ast.Name) and h.type.id in ("Exception", "BaseException"))]
    return verdict(bad, handlers)


def new_modules(w):
    return [c for c in w.code if c.new and c.module and c.text.strip()]


def all_names(w):
    mods = [c for c in new_modules(w) if not c.path.endswith("__init__.py")]
    has = lambda c: any(isinstance(t, ast.Name) and t.id == "__all__" for n in c.tree.body
                        if isinstance(n, ast.Assign) for t in n.targets)
    return verdict(any(not has(c) for c in mods), mods)


def constants(w):
    found = [(c, t.id) for c in w.py(tests=False) for n in c.nodes(ast.Assign) if n in c.tree.body
             for t in n.targets if isinstance(t, ast.Name) and re.fullmatch(r"[A-Z][A-Z0-9_]+", t.id)
             and t.id not in ("_LOGGER",)]
    return verdict(any(c.path != "stock/const.py" for c, _ in found), found)


def options(w):
    return [n for c in w.py(tests=False) for n in c.nodes(ast.Call) if call_name(n) == "add_argument"
            and n.args and isinstance(n.args[0], ast.Constant) and str(n.args[0].value).startswith("-")]


def flag_kebab(w):
    found = options(w)
    def bad(n):
        names = [a.value for a in n.args if isinstance(a, ast.Constant) and isinstance(a.value, str)]
        longs = [s for s in names if s.startswith("--")]
        return not longs or any("_" in s for s in longs)
    return verdict(any(bad(n) for n in found), found)


def help_text(w):
    texts = [k.value.value for c in w.py(tests=False) for n in c.nodes(ast.Call)
             if call_name(n) in ("add_argument", "add_parser") for k in n.keywords
             if k.arg == "help" and isinstance(k.value, ast.Constant) and isinstance(k.value.value, str)]
    return verdict(any(t.endswith(".") or t[:1].isupper() for t in texts), texts)


def command_names(w):
    defs = [n.name for c in w.code if c.path.startswith("stock/commands/") and not c.path.endswith("__init__.py")
            for n in c.nodes(ast.FunctionDef) if n in c.tree.body]
    return verdict(any(not (d.startswith("cmd_") or d.startswith("_")) for d in defs), defs)


def no_print(w):
    calls = [n for c in w.py(tests=False) if c.path != "stock/cli.py" for n in c.nodes(ast.Call) if call_name(n) == "print"]
    out = [n for c in w.py(tests=False) for n in c.nodes(ast.Call) if call_name(n) in ("print", "out")]
    return verdict(calls, out)


def pathlib_only(w):
    uses = [n for c in w.py() for n in c.nodes(ast.Attribute) if n.attr == "path"
            and isinstance(n.value, ast.Name) and n.value.id == "os"]
    paths = uses + [n for c in w.py() for n in c.nodes(ast.Name) if n.id == "Path"]
    return verdict(uses, paths)


def encoding(w):
    calls = []
    for c in w.py():
        for n in c.nodes(ast.Call):
            name = call_name(n)
            if name in ("open", "read_text", "write_text"):
                mode = [a.value for a in n.args[1:2] if isinstance(a, ast.Constant)] + \
                       [k.value.value for k in n.keywords if k.arg == "mode" and isinstance(k.value, ast.Constant)]
                if not any("b" in str(m) for m in mode):
                    calls.append(n)
    return verdict(any(not any(k.arg == "encoding" for k in n.keywords) for n in calls), calls)


def test_files(w):
    files = [p for p in w.new if p.startswith("tests/") and p.endswith(".py") and not p.endswith("__init__.py")]
    if not files:
        return None
    names = [Path(p).name for p in files]
    return "kept" if all(n.endswith("_test.py") for n in names) else \
        "old" if all(n.startswith("test_") for n in names) else "broken"


def test_classes(w):
    classes = [n.name for c in w.py(modules=False) for n in c.nodes(ast.ClassDef)]
    return verdict(any(not n.endswith("Tests") for n in classes), classes)


def frozen(w):
    decos = [d for c in w.py(tests=False) for n in c.nodes(ast.ClassDef) for d in n.decorator_list
             if (call_name(d) if isinstance(d, ast.Call) else getattr(d, "id", getattr(d, "attr", ""))) == "dataclass"]
    ok = lambda d: isinstance(d, ast.Call) and {(k.arg, getattr(k.value, "value", None)) for k in d.keywords} >= \
        {("frozen", True), ("slots", True)}
    return verdict(any(not ok(d) for d in decos), decos)


def utc(w):
    calls = [n for c in w.py() for n in c.nodes(ast.Call) if call_name(n) in ("now", "utcnow", "today")
             and isinstance(n.func, ast.Attribute) and "datetime" in ast.unparse(n.func.value)]
    return verdict(any(call_name(n) != "now" or not (n.args or any(k.arg == "tz" for k in n.keywords)) for n in calls), calls)


def json_format(w):
    calls = [n for c in w.py(tests=False) for n in c.nodes(ast.Call) if call_name(n) in ("dump", "dumps")
             and isinstance(n.func, ast.Attribute) and getattr(n.func.value, "id", "") == "json"]
    ok = lambda n: {(k.arg, getattr(k.value, "value", None)) for k in n.keywords} >= {("indent", 2), ("sort_keys", True)}
    return verdict(any(not ok(n) for n in calls), calls)


def future(w):
    mods = new_modules(w) + [c for c in w.code if c.new and c.test and c.text.strip()]
    has = lambda c: any(isinstance(n, ast.ImportFrom) and n.module == "__future__" for n in c.tree.body)
    return verdict(any(not has(c) for c in mods), mods)


def absolute(w):
    imports = [n for c in w.py() for n in c.nodes(ast.ImportFrom)]
    return verdict(any(n.level for n in imports), imports)


def settings(w):
    uses = [n for c in w.py() if c.path != "stock/config.py" for n in c.nodes(ast.Attribute)
            if n.attr in ("environ", "getenv") and getattr(n.value, "id", "") == "os"]
    reads = uses + [n for c in w.py() for n in c.nodes(ast.Call) if call_name(n) == "setting"]
    return verdict(uses, reads)


def processes(w):
    calls = [n for c in w.py() for n in c.nodes(ast.Call) if isinstance(n.func, ast.Attribute)
             and getattr(n.func.value, "id", "") == "subprocess"]
    ok = lambda n: n.func.attr == "run" and n.args and isinstance(n.args[0], (ast.List, ast.Name)) and \
        any(k.arg == "check" and getattr(k.value, "value", None) is True for k in n.keywords) and \
        not any(k.arg == "shell" and getattr(k.value, "value", None) is True for k in n.keywords)
    return verdict(any(not ok(n) for n in calls), calls)


def keyword_bools(w):
    found = []
    for c in w.py(tests=False):
        for f in c.nodes(ast.FunctionDef):
            positional = f.args.posonlyargs + f.args.args
            defaults = [None] * (len(positional) - len(f.args.defaults)) + f.args.defaults
            for a, d in zip(positional, defaults):
                if (a.annotation is not None and ast.unparse(a.annotation) == "bool") or \
                        (isinstance(d, ast.Constant) and isinstance(d.value, bool)):
                    found.append(("positional", a.arg))
            for a in f.args.kwonlyargs:
                if a.annotation is not None and ast.unparse(a.annotation) == "bool":
                    found.append(("keyword", a.arg))
    return verdict(any(k == "positional" for k, _ in found), found)


def no_assert(w):
    asserts = [n for c in w.py(tests=False) for n in c.nodes(ast.Assert)]
    return "broken" if asserts else ("kept" if w.py(tests=False) else None)


def line_length(w):
    lengths = [len(l) for c in w.code for l in w.lines.get(c.path, [])]
    if not lengths:
        return None
    longest = max(lengths)
    return "kept" if longest <= 79 else "old" if longest <= 100 else "broken"


def quotes(w):
    strings = []
    for c in w.py():
        try:
            tokens = list(tokenize.generate_tokens(io.StringIO(c.text).readline))
        except (tokenize.TokenError, IndentationError):
            continue
        for t in tokens:
            if t.type == tokenize.STRING and t.start[0] in c.added:
                body = re.sub(r"^[rbfuRBFU]*", "", t.string)
                if not body.startswith(('"""', "'''")):
                    strings.append(body)
    return verdict(any(s.startswith('"') and "'" not in s for s in strings), strings)


def changelog(w):
    lines = [l.strip() for l in w.lines.get("CHANGELOG.md", []) if l.strip().startswith("- ")]
    if any(re.match(r"- (Added|Changed|Fixed): ", l) for l in lines):
        return "kept"
    return "old" if any(re.match(r"- \[(added|changed|fixed)\] ", l) for l in lines) else "broken"


def module_docstring(w):
    mods = new_modules(w)
    one = lambda c: (d := ast.get_docstring(c.tree)) is not None and "\n" not in d.strip()
    return verdict(any(not one(c) for c in mods), mods)


def store_names(w):
    classes = [n for c in w.code if c.path.startswith("stock/storage/") and c.path != "stock/storage/base.py"
               for n in c.nodes(ast.ClassDef)]
    ok = lambda n: n.name.endswith("Store") and any(getattr(b, "id", getattr(b, "attr", "")) == "Store" for b in n.bases)
    return verdict(any(not ok(n) for n in classes), classes)


def type_hints(w):
    funcs = [f for c in w.py(tests=False) for f in c.nodes(ast.FunctionDef)]
    def hinted(f):
        args = [a for a in f.args.posonlyargs + f.args.args + f.args.kwonlyargs if a.arg not in ("self", "cls")]
        return f.returns is not None and all(a.annotation is not None for a in args)
    return verdict(any(not hinted(f) for f in funcs), funcs)


def readme_commands(w):
    commands = [n for c in w.code if c.path.startswith("stock/commands/") for n in c.nodes(ast.FunctionDef)
                if n.name.startswith("cmd_") and n in c.tree.body]
    if not commands:
        return None
    rows = [l for l in w.lines.get("README.md", []) if l.startswith("| `stock ")]
    return "kept" if rows else "broken"


def exit_codes(w):
    returns = [n for c in w.code if c.path.startswith("stock/commands/") for f in c.nodes(ast.FunctionDef)
               if f.name.startswith("cmd_") for n in ast.walk(f) if isinstance(n, ast.Return) and n.value is not None]
    bare = [n for n in returns if isinstance(n.value, ast.Constant) and isinstance(n.value.value, int)]
    return verdict(bare, returns)


def builtin_generics(w):
    imports = [n for c in w.py() for n in c.nodes(ast.ImportFrom) if n.module == "typing"]
    hints = [n for c in w.py() for n in c.nodes(ast.FunctionDef) if n.returns is not None or n.args.args]
    bad = [n for n in imports if any(a.name in TYPING for a in n.names)]
    return verdict(bad, hints + imports)


CHECKS = {f.__name__: f for f in [
    log_args, logger_name, own_errors, error_text, raise_from, broad_except, all_names, constants, flag_kebab,
    help_text, command_names, no_print, pathlib_only, encoding, test_files, test_classes, frozen, utc, json_format,
    future, absolute, settings, processes, keyword_bools, no_assert, line_length, quotes, changelog,
    module_docstring, store_names, type_hints, readme_commands, exit_codes, builtin_generics]}


def check(folder, base, targets):
    """Each target's state for the work in `folder` since `base`, and whether the tests pass."""
    work = Work.of(folder, base)
    states = {t["id"]: CHECKS[t["check"]](work) for t in targets}
    tests = subprocess.run([sys.executable, "-m", "unittest", "discover", "-s", "tests", "-p", "*test*.py"],
                           cwd=folder, capture_output=True, text=True, timeout=300)
    return {"states": states, "tests": tests.returncode == 0}


def selftest(targets):
    """Runs each target's check on its good and bad sample in a scratch repo; prints the failures."""
    import shutil
    import tempfile
    bad = 0
    for t in targets:
        for sample, want in (("good", "kept"), ("bad", t.get("bad_state", "broken"))):
            with tempfile.TemporaryDirectory() as d:
                shutil.copytree(HERE / "seed", d, dirs_exist_ok=True)
                git = lambda *a: subprocess.run(["git", *a], cwd=d, capture_output=True, check=True)
                git("init", "-q")
                git("add", "-A")
                git("-c", "user.name=b", "-c", "user.email=b@l", "commit", "-q", "-m", "base")
                base = subprocess.run(["git", "rev-parse", "HEAD"], cwd=d, capture_output=True, text=True).stdout.strip()
                for path, text in t[sample].items():
                    p = Path(d) / path
                    p.parent.mkdir(parents=True, exist_ok=True)
                    p.write_text(p.read_text() + text if p.exists() and path.endswith(".md") else text)
                got = CHECKS[t["check"]](Work.of(d, base))
                if got != want:
                    bad += 1
                    print(f"{t['id']} {sample}: got {got}, want {want}")
    print(f"{len(targets) * 2 - bad} of {len(targets) * 2} samples right")
    return bad


def main():
    targets = json.loads((HERE / "data" / "targets.json").read_text())
    if sys.argv[1:] == ["--self"]:
        sys.exit(1 if selftest(targets) else 0)
    print(json.dumps(check(sys.argv[1], sys.argv[2], targets), indent=2))


if __name__ == "__main__":
    main()
