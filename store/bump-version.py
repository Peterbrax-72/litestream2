"""Bump the app version everywhere it lives. Usage: python store/bump-version.py 0.3.0"""
import json, re, sys
v = sys.argv[1] if len(sys.argv) > 1 else None
if not v or not re.fullmatch(r'\d+\.\d+\.\d+', v):
    sys.exit('usage: python store/bump-version.py X.Y.Z')
def sub(path, pattern, repl, count=1):
    s = open(path, encoding='utf-8').read()
    n, k = re.subn(pattern, repl, s, count=count, flags=re.M)
    if not k: sys.exit(f'no version found in {path}')
    open(path, 'w', encoding='utf-8', newline='').write(n)
sub('package.json', r'("version":\s*")[^"]+(")', rf'\g<1>{v}\2')
sub('package-lock.json', r'("version":\s*")[^"]+(")', rf'\g<1>{v}\2', count=2)
sub('src-tauri/tauri.conf.json', r'("version":\s*")[^"]+(")', rf'\g<1>{v}\2')
sub('src-tauri/Cargo.toml', r'^(version\s*=\s*")[^"]+(")', rf'\g<1>{v}\2')
sub('src-tauri/Cargo.lock', r'(name = "litestream"\nversion = ")[^"]+(")', rf'\g<1>{v}\2')
sub('ui/app.js', r"(const APP_VERSION=')[^']+(')", rf'\g<1>{v}\2')
print('Version set to', v)
