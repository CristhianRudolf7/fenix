#!/usr/bin/env python3
"""Copy a stopped standalone runtime into a newly provisioned tenant; keep originals."""
import argparse
import fcntl
import json
import os
from pathlib import Path
import shutil
import sqlite3


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("old_data", type=Path)
    parser.add_argument("user_root", type=Path)
    parser.add_argument("workspace", type=Path)
    args = parser.parse_args()
    os.umask(0o077)
    root = args.user_root.resolve(strict=True)
    workspace = args.workspace.resolve(strict=True)
    if not workspace.is_relative_to(root):
        raise SystemExit("The new workspace must be inside the user directory")
    state = root / "state"
    target = state / "fenix.sqlite3"
    if target.exists():
        raise SystemExit("Refusing to replace an existing tenant database")
    with (args.old_data / "instance.lock").open("r+") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        with sqlite3.connect(args.old_data / "fenix.sqlite3") as source, sqlite3.connect(target) as dest:
            source.backup(dest)
            row = dest.execute("SELECT data FROM records WHERE kind='settings' AND id='active'").fetchone()
            if row:
                settings = json.loads(row[0])
                old_workspace = Path(settings["workspace"])
                settings["workspace"] = str(workspace)
                dest.execute("UPDATE records SET data=? WHERE kind='settings' AND id='active'", (json.dumps(settings),))
                for project_id, old in dest.execute("SELECT id, workspace FROM projects WHERE workspace IS NOT NULL").fetchall():
                    original = Path(old)
                    replacement = workspace / original.relative_to(old_workspace) if original.is_relative_to(old_workspace) else None
                    dest.execute("UPDATE projects SET workspace=? WHERE id=?", (str(replacement) if replacement and replacement.is_dir() else None, project_id))
            dest.execute("INSERT OR REPLACE INTO metadata(key,value) VALUES('tenant_user_id',?)", (root.name,))
            dest.commit()
        for name in ["instructions", "processes", "browser", "web", "artifacts", "checkpoints"]:
            source = args.old_data / name
            if source.exists():
                shutil.copytree(source, state / name, dirs_exist_ok=True, symlinks=True)
        for name in ["browser.json", "web-search.json"]:
            source = args.old_data / name
            if source.exists():
                shutil.copy2(source, state / name)
        browser_path = state / "browser.json"
        browser = json.loads(browser_path.read_text()) if browser_path.exists() else {}
        browser["executable"] = "/usr/local/bin/fenix-chromium"
        browser_path.write_text(json.dumps(browser, indent=2))
    print(f"Migrated runtime into {state}; originals preserved")


if __name__ == "__main__":
    main()
