#!/usr/bin/env python3
"""Read-only Linux CPU sampling. 100% means one fully occupied CPU core."""

import argparse
import json
import os
from pathlib import Path
import subprocess
import time


def stat(path):
    fields = path.read_text().rsplit(")", 1)[1].split()
    return {"ticks": int(fields[11]) + int(fields[12]), "start": int(fields[19])}


def snapshot(pid):
    root = Path("/proc") / str(pid)
    process = stat(root / "stat")
    process["threads"] = {}
    for task in (root / "task").iterdir():
        try:
            thread = stat(task / "stat")
            thread["name"] = (task / "comm").read_text().strip()
            process["threads"][task.name] = thread
        except (FileNotFoundError, ProcessLookupError):
            pass
    return process


def editor_pids():
    result = []
    for path in Path("/proc").iterdir():
        if not path.name.isdigit():
            continue
        try:
            if (path / "comm").read_text().strip() == "basscript-app":
                result.append(int(path.name))
        except (PermissionError, FileNotFoundError, ProcessLookupError):
            pass
    return sorted(result)


def active_window():
    result = subprocess.run(
        ["xprop", "-root", "_NET_ACTIVE_WINDOW"], text=True, capture_output=True
    )
    return result.stdout.strip() if result.returncode == 0 else "unavailable"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pid", type=int, action="append")
    parser.add_argument("--interval", type=float, default=10)
    parser.add_argument("--samples", type=int, default=3)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if not 0 < args.interval <= 30 or not 1 <= args.samples <= 12:
        parser.error("interval must be 0..30 seconds and samples 1..12")
    pids = args.pid or editor_pids()
    if not pids:
        raise SystemExit("No BasScript process is running")
    report = {"cpu_units": "100% = one CPU core", "processes": {}, "samples": []}
    for pid in pids:
        report["processes"][str(pid)] = {"cwd": str((Path('/proc') / str(pid) / 'cwd').resolve())}
    print(json.dumps(report["processes"]), flush=True)
    ticks_per_second = os.sysconf("SC_CLK_TCK")
    previous = {pid: snapshot(pid) for pid in pids}
    started = time.monotonic()
    for _ in range(args.samples):
        time.sleep(args.interval)
        now = time.monotonic()
        elapsed = now - started
        sample = {"seconds": round(elapsed, 3), "active_window": active_window(), "processes": {}}
        for pid in pids:
            try:
                current = snapshot(pid)
            except (FileNotFoundError, ProcessLookupError):
                continue
            before = previous[pid]
            if before["start"] != current["start"]:
                continue
            factor = 100 / (ticks_per_second * elapsed)
            threads = []
            for tid, thread in current["threads"].items():
                old = before["threads"].get(tid)
                if old and old["start"] == thread["start"]:
                    threads.append({"tid": tid, "name": thread["name"], "cpu_percent": round((thread["ticks"] - old["ticks"]) * factor, 2)})
            sample["processes"][str(pid)] = {
                "cpu_percent": round((current["ticks"] - before["ticks"]) * factor, 2),
                "top_threads": sorted(threads, key=lambda item: item["cpu_percent"], reverse=True)[:8],
            }
            previous[pid] = current
        report["samples"].append(sample)
        print(json.dumps(sample), flush=True)
        started = now
    if args.output:
        args.output.write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    main()
