#!/usr/bin/env python3
"""Run after `tauri android init`: adds permissions and LiteStream's native bridge.

Usage: python3 store/android/patch-android.py
"""
import pathlib, re, sys

root = pathlib.Path("src-tauri/gen/android/app/src/main")
manifest = root / "AndroidManifest.xml"
if not manifest.exists():
    sys.exit("AndroidManifest.xml not found - did `tauri android init` run?")

text = manifest.read_text(encoding="utf-8")
wanted = [
    '<uses-permission android:name="android.permission.INTERNET" />',
    '<uses-permission android:name="android.permission.CAMERA" />',
    '<uses-permission android:name="android.permission.RECORD_AUDIO" />',
    '<uses-permission android:name="android.permission.MODIFY_AUDIO_SETTINGS" />',
    '<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" android:maxSdkVersion="28" />',
    '<uses-feature android:name="android.hardware.camera" android:required="false" />',
    '<uses-feature android:name="android.hardware.microphone" android:required="false" />',
]
add = "\n    ".join(w for w in wanted if w.split('"')[1] not in text)
if add:
    text = text.replace("<application", add + "\n\n    <application", 1)
# Remote control talks to the PC at http://192.168.x.x, which Android blocks by default.
if "usesCleartextTraffic" not in text:
    text = text.replace("<application", '<application android:usesCleartextTraffic="true"', 1)
manifest.write_text(text, encoding="utf-8")

kt = next(root.rglob("MainActivity.kt"), None)
if kt is None:
    sys.exit("MainActivity.kt not found")
package = re.search(r"^package\s+([\w.]+)", kt.read_text(encoding="utf-8"), re.M).group(1)
template = pathlib.Path("store/android/MainActivity.kt").read_text(encoding="utf-8")
kt.write_text(template.replace("__PACKAGE__", package), encoding="utf-8")
print("Patched", manifest, "and", kt, "(package", package + ")")
