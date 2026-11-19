"""Check the links in the repository's Markdown documents and doc comments.

A relative link must name a file that exists, and its anchor, if any, must
name a heading of that file as GitHub renders it. A link to the API
reference on GitHub Pages must name a file and an anchor that exist in the
rustdoc output given with --doc (ADR 0052). The doc comments of the Rust
sources link to the documents by their GitHub URL on main, and such a link
must name a file and a heading of the working tree. Other absolute links
are not checked.

Usage: python3 -I .github/scripts/check-links.py [--doc target/doc]
"""

import os
import re
import sys

PAGES_API = "https://photom.github.io/hallmark/api/"
REPO_BLOB = "https://github.com/photom/hallmark/blob/main/"
DOC_COMMENT = re.compile(r"\s*//[/!] ?(.*)")
LINK = re.compile(r"(?<!!)\[[^\]]*\]\(([^)\s]+)\)")
HEADING = re.compile(r"#{1,6} (.*)")
SCHEME = re.compile(r"[A-Za-z][A-Za-z0-9+.-]*:")
HTML_ID = re.compile(r'\sid="([^"]+)"')
SKIPPED_DIRS = {".git", "target", "node_modules"}


def slug(heading):
    text = heading.replace("`", "")
    text = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", text)
    text = text.strip().lower()
    text = re.sub(r"[^\w\- ]", "", text)
    return text.replace(" ", "-")


def markdown_anchors(path, cache):
    if path in cache:
        return cache[path]
    anchors = set()
    counts = {}
    in_code = False
    with open(path, encoding="utf-8") as file:
        for line in file:
            if line.startswith("```"):
                in_code = not in_code
                continue
            if in_code:
                continue
            match = HEADING.match(line)
            if match is None:
                continue
            base = slug(match.group(1))
            seen = counts.get(base, 0)
            counts[base] = seen + 1
            if seen == 0:
                anchors.add(base)
            else:
                anchors.add(base + "-" + str(seen))
    cache[path] = anchors
    return anchors


def html_ids(path, cache):
    if path in cache:
        return cache[path]
    with open(path, encoding="utf-8") as file:
        ids = set(HTML_ID.findall(file.read()))
    cache[path] = ids
    return ids


def check_api_link(target, doc_dir, html_cache):
    rest = target[len(PAGES_API):]
    path_part, _, anchor = rest.partition("#")
    if doc_dir is None:
        return "an API link, but no --doc directory was given"
    if path_part == "" or path_part.endswith("/"):
        path_part = path_part + "index.html"
    local = os.path.join(doc_dir, path_part)
    if not os.path.isfile(local):
        return "no such page in the rustdoc output: " + path_part
    if anchor != "" and anchor not in html_ids(local, html_cache):
        return "no such anchor in " + path_part + ": " + anchor
    return None


def check_relative_link(target, source, md_cache):
    path_part, _, anchor = target.partition("#")
    if path_part == "":
        resolved = source
    else:
        resolved = os.path.normpath(os.path.join(os.path.dirname(source), path_part))
    if not os.path.exists(resolved):
        return "no such file: " + path_part
    if anchor != "" and resolved.endswith(".md"):
        if anchor not in markdown_anchors(resolved, md_cache):
            return "no such heading: #" + anchor
    return None


def check_repo_link(target, md_cache):
    rest = target[len(REPO_BLOB):]
    path_part, _, anchor = rest.partition("#")
    resolved = os.path.normpath(path_part)
    if path_part == "" or not os.path.exists(resolved):
        return "no such file in the repository: " + path_part
    if anchor != "" and resolved.endswith(".md"):
        if anchor not in markdown_anchors(resolved, md_cache):
            return "no such heading: #" + anchor
    return None


def files_ending(root, suffix):
    found = []
    for directory, subdirs, files in os.walk(root):
        kept = []
        for name in subdirs:
            if name not in SKIPPED_DIRS:
                kept.append(name)
        subdirs[:] = kept
        for name in files:
            if name.endswith(suffix):
                found.append(os.path.join(directory, name))
    found.sort()
    return found


def doc_comment_lines(path):
    """Gives each doc comment line of a Rust file, outside code blocks, with
    its line number."""
    found = []
    in_code = False
    with open(path, encoding="utf-8") as file:
        lines = file.readlines()
    for number, line in enumerate(lines, start=1):
        match = DOC_COMMENT.match(line)
        if match is None:
            continue
        text = match.group(1)
        if text.startswith("```"):
            in_code = not in_code
            continue
        if not in_code:
            found.append((number, text))
    return found


def main(arguments):
    doc_dir = None
    if len(arguments) == 2 and arguments[0] == "--doc":
        doc_dir = arguments[1]
    elif len(arguments) != 0:
        print(__doc__, file=sys.stderr)
        return 2

    md_cache = {}
    html_cache = {}
    problems = 0
    checked = 0
    for source in files_ending(".", ".md"):
        in_code = False
        with open(source, encoding="utf-8") as file:
            lines = file.readlines()
        for number, line in enumerate(lines, start=1):
            if line.startswith("```"):
                in_code = not in_code
                continue
            if in_code:
                continue
            for target in LINK.findall(line):
                if target.startswith(PAGES_API):
                    problem = check_api_link(target, doc_dir, html_cache)
                elif target.startswith(REPO_BLOB):
                    problem = check_repo_link(target, md_cache)
                elif SCHEME.match(target):
                    continue
                else:
                    problem = check_relative_link(target, source, md_cache)
                checked += 1
                if problem is not None:
                    problems += 1
                    print(source + ":" + str(number) + ": " + target + ": " + problem)
    for source in files_ending(".", ".rs"):
        for number, text in doc_comment_lines(source):
            for target in LINK.findall(text):
                if not target.startswith(REPO_BLOB):
                    continue
                problem = check_repo_link(target, md_cache)
                checked += 1
                if problem is not None:
                    problems += 1
                    print(source + ":" + str(number) + ": " + target + ": " + problem)
    print("links checked: " + str(checked) + ", broken: " + str(problems))
    if problems == 0:
        return 0
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
