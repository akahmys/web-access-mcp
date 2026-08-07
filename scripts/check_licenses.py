#!/usr/bin/env python3
import shutil
import subprocess
import sys

def main():
    print("=== License Audit Scanner (cargo-deny) ===")
    
    cargo_deny_bin = shutil.which("cargo-deny")
    cargo_bin = shutil.which("cargo")

    cmd = None
    if cargo_deny_bin:
        cmd = [cargo_deny_bin, "check", "licenses"]
    elif cargo_bin:
        # Check if cargo-deny subcommand is available
        res = subprocess.run([cargo_bin, "deny", "--version"], capture_output=True)
        if res.returncode == 0:
            cmd = [cargo_bin, "deny", "check", "licenses"]

    if not cmd:
        print("ERROR: 'cargo-deny' is not installed. Please install it using 'cargo install cargo-deny'.", file=sys.stderr)
        sys.exit(1)

    try:
        result = subprocess.run(cmd)
        sys.exit(result.returncode)
    except Exception as e:
        print(f"Error running cargo-deny: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()

