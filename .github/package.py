#!/usr/bin/env python3
"""Zip a desktop build for a GitHub Release. Usage: package.py <platform-name>"""
import os, sys, zipfile

name = sys.argv[1]
out = f"pilgrim-{name}.zip"
readme = """PILGRIM
Somewhere beyond the event horizon. A small ship, a long way from home.

Keyboard: Z or Enter to start, Esc back to the title, F11 or Alt+Enter fullscreen.
(Flying, shooting and controllers arrive in v0.2.)

Windows: if SmartScreen warns you, click "More info" then "Run anyway".
macOS: the first time, right-click the app and choose Open (it isn't signed).
Linux: run ./pilgrim (you may need: chmod +x pilgrim).

https://github.com/Bananalorian/macroshmup
"""

def add_exec(z, path, arcname):
    info = zipfile.ZipInfo(arcname)
    info.external_attr = (0o755 & 0xFFFF) << 16
    info.compress_type = zipfile.ZIP_DEFLATED
    with open(path, "rb") as f:
        z.writestr(info, f.read())

with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
    z.writestr("READ ME.txt", readme)
    if name.startswith("windows"):
        z.write("target/release/pilgrim.exe", "PILGRIM.exe")
    elif name.startswith("macos"):
        app = "PILGRIM.app/Contents"
        plist = f"""<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>PILGRIM</string>
  <key>CFBundleDisplayName</key><string>PILGRIM</string>
  <key>CFBundleIdentifier</key><string>io.github.bananalorian.pilgrim</string>
  <key>CFBundleExecutable</key><string>pilgrim</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>{os.environ.get("GITHUB_REF_NAME", "dev").lstrip("v")}</string>
  <key>NSHighResolutionCapable</key><true/>
</dict></plist>
"""
        z.writestr(f"{app}/Info.plist", plist)
        add_exec(z, "target/release/pilgrim", f"{app}/MacOS/pilgrim")
    else:
        add_exec(z, "target/release/pilgrim", "pilgrim")
print("wrote", out)
