#!/usr/bin/env python3
"""Generate categorized GitHub release notes from Conventional Commits."""

import argparse
import os
import re
import subprocess
import tempfile


def run_cmd(args: list[str]) -> str:
    """Run a command without a shell and return trimmed stdout."""
    return subprocess.check_output(args, text=True).strip()


def get_repo_slug() -> str:
    """Resolve owner/repo slug from origin remote, defaulting to cook-docs."""
    try:
        url = run_cmd(["git", "remote", "get-url", "origin"])
        match = re.search(r"github\.com[:/]([^/]+/[^/.]+)(?:\.git)?", url)
        if match:
            return match.group(1)
    except Exception:
        pass
    return "nicholaswilde/cook-docs"


def format_special_thanks(thanks: str | None, reason: str | None = None) -> str | None:
    """Format an optional Special Thanks section."""
    if not thanks:
        return None
    thanks = thanks.strip()
    if thanks.startswith("###") or thanks.startswith("Huge thanks") or "\n" in thanks:
        return thanks
    name = thanks.strip("*")
    if reason:
        reason = reason.strip()
        if not reason.endswith("!"):
            reason += "!"
        return f"Huge thanks to **{name}** for {reason}"
    return f"Huge thanks to **{name}**!"


def resolve_git_range(raw_range: str | None) -> tuple[str, str]:
    """Return (git_range, target_tag) from CLI range argument or latest tags."""
    all_tags = [t for t in run_cmd(["git", "tag", "--sort=-v:refname"]).splitlines() if t]

    if raw_range:
        if " to " in raw_range:
            left, right = raw_range.split(" to ", 1)
            git_range = f"{left.strip()}..{right.strip()}"
        else:
            git_range = raw_range.replace("...", "..")

        parts = git_range.split("..")
        if len(parts) == 2:
            return git_range, parts[1]

        target = parts[0]
        if target in all_tags:
            idx = all_tags.index(target)
            if idx + 1 < len(all_tags):
                return f"{all_tags[idx + 1]}..{target}", target
            return target, target
        return git_range, (all_tags[0] if all_tags else "HEAD")

    if len(all_tags) < 2:
        latest = all_tags[0] if all_tags else "HEAD"
        return "HEAD", latest

    return f"{all_tags[1]}..{all_tags[0]}", all_tags[0]


def build_release_notes(
    git_range: str,
    repo_slug: str,
    thanks: str | None = None,
    thanks_reason: str | None = None,
) -> str:
    """Categorize commits in git_range into Markdown release notes."""
    log_output = run_cmd(["git", "log", "--pretty=format:- %s", git_range])

    sections: dict[str, list[str]] = {
        "feat": [],
        "fix": [],
        "improve": [],
        "docs": [],
    }

    for line in log_output.splitlines():
        if "conductor" in line.lower() or "checkpoint" in line.lower():
            continue

        match = re.match(r"- (\w+)(\([^)]+\))?:\s*(.*)", line)
        if not match:
            continue

        type_str, scope, msg = match.groups()
        msg = msg.strip()
        msg = msg[0].upper() + msg[1:] if msg else msg

        if scope:
            entry = f"- `{scope[1:-1]}`: {msg}"
        else:
            entry = f"- {msg}"

        if type_str == "feat":
            sections["feat"].append(entry)
        elif type_str == "fix":
            sections["fix"].append(entry)
        elif type_str in {"refactor", "perf", "style"}:
            sections["improve"].append(entry)
        elif type_str == "docs":
            sections["docs"].append(entry)

    summary: list[str] = []
    headers = [
        ("feat", "### 🚀 **New Features**"),
        ("fix", "### 🐛 **Bug Fixes**"),
        ("improve", "### ✨ **Improvements**"),
        ("docs", "### 📝 **Documentation**"),
    ]
    for key, header in headers:
        if sections[key]:
            summary.append(f"{header}\n\n" + "\n".join(sections[key]) + "\n")

    thanks_entry = format_special_thanks(thanks, thanks_reason)
    if thanks_entry:
        summary.append(f"### 🙏 **Special Thanks**\n\n{thanks_entry}\n")

    if ".." in git_range:
        url_range = git_range.replace("..", "...")
        summary.append(
            f"**Full Changelog**: https://github.com/{repo_slug}/compare/{url_range}\n"
        )
    else:
        summary.append(
            f"**Full Changelog**: https://github.com/{repo_slug}/commits/{git_range}\n"
        )

    return "\n".join(summary)


def main() -> None:
    """CLI entrypoint for release summary generation."""
    parser = argparse.ArgumentParser(description="Generate release summary")
    parser.add_argument(
        "range",
        nargs="*",
        default=None,
        help="Git commit or tag range (e.g. v0.9.0..v0.9.1 or 'v0.9.0 to v0.9.1')",
    )
    parser.add_argument(
        "--thanks",
        "--special-thanks",
        dest="thanks",
        default=None,
        help="Person to thank or full thanks message",
    )
    parser.add_argument(
        "--for",
        "--reason",
        dest="thanks_reason",
        default=None,
        help="Reason or contribution to thank the person for",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print summary without updating GitHub release",
    )
    parser.add_argument(
        "--draft",
        action="store_true",
        help="Explicitly mark the release as a draft when editing",
    )
    parser.add_argument(
        "--prerelease",
        action="store_true",
        help="Explicitly mark the release as a pre-release when editing",
    )
    args = parser.parse_args()

    repo_slug = get_repo_slug()
    raw_range = " ".join(args.range).strip() if args.range else None
    git_range, latest = resolve_git_range(raw_range)

    print(f"Using range: {git_range}")
    notes = build_release_notes(git_range, repo_slug, args.thanks, args.thanks_reason)

    if args.dry_run:
        print("Dry run summary output:\n")
        print(notes)
        return

    tmp_path = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".md", encoding="utf-8", delete=False
        ) as tmp_file:
            tmp_file.write(notes)
            tmp_path = tmp_file.name

        print(f"Updating GitHub release for {latest}...")
        edit_cmd = ["gh", "release", "edit", latest, "-F", tmp_path]
        if args.draft:
            edit_cmd.append("--draft")
        if args.prerelease:
            edit_cmd.append("--prerelease")
        try:
            url = run_cmd(edit_cmd)
            print(f"Release updated: {url}")
        except Exception as err:
            print(f"Failed to edit release ({err}), attempting to create draft pre-release...")
            try:
                title = latest.lstrip("v")
                url = run_cmd(
                    [
                        "gh",
                        "release",
                        "create",
                        latest,
                        "--draft",
                        "--prerelease",
                        "--title",
                        title,
                        "-F",
                        tmp_path,
                    ]
                )
                print(f"Draft pre-release created: {url}")
            except Exception as err_create:
                print(f"Failed to create release: {err_create}")
    finally:
        if tmp_path and os.path.exists(tmp_path):
            os.remove(tmp_path)


if __name__ == "__main__":
    main()
