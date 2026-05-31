#!/usr/bin/env python3
"""Replace 'kcraft' (case-insensitive) with 'kcraft' with case preservation."""

import os
import re
import subprocess
import sys

def target_for_case(match_text):
    if match_text.islower():
        return "kcraft"
    if match_text.isupper():
        return "KCRAFT"
    return "KCraft"

def replace_in_content(content):
    def replacer(m):
        return target_for_case(m.group(0))
    return re.sub(r'(?i)kcraft', replacer, content)

def main():
    # Step 1: Get all tracked files
    result = subprocess.run(
        ["git", "ls-files"],
        capture_output=True, text=True
    )
    files = [f for f in result.stdout.strip().split("\n") if f]
    
    # Step 2: Replace content
    changed_files = []
    for filepath in files:
        if not os.path.isfile(filepath):
            continue
        try:
            with open(filepath, 'rb') as f:
                raw = f.read()
            content = raw.decode('utf-8', errors='replace')
            new_content = replace_in_content(content)
            if new_content != content:
                with open(filepath, 'w', encoding='utf-8') as f:
                    f.write(new_content)
                changed_files.append(filepath)
        except Exception as e:
            print(f"Warning: could not process {filepath}: {e}", file=sys.stderr)
    
    print(f"Content replaced in {len(changed_files)} files.")
    
    # Step 3: Rename files and dirs
    # Collect paths containing 'kcraft'
    all_paths = {}
    for f in files:
        parts = f.split("/")
        for i in range(1, len(parts) + 1):
            parent = "/".join(parts[:i])
            if re.search(r'(?i)kcraft', parent) and parent not in all_paths:
                all_paths[parent] = True
    
    sorted_paths = sorted(all_paths.keys(), key=lambda p: p.count("/"), reverse=True)
    
    renamed = 0
    for old_path in sorted_paths:
        if not os.path.exists(old_path):
            continue
        new_path = re.sub(r'(?i)kcraft', lambda m: target_for_case(m.group(0)), old_path)
        if old_path == new_path:
            continue
        new_dir = os.path.dirname(new_path)
        if new_dir and not os.path.exists(new_dir):
            os.makedirs(new_dir, exist_ok=True)
        try:
            subprocess.run(["git", "mv", old_path, new_path], check=True, capture_output=True)
            renamed += 1
        except subprocess.CalledProcessError as e:
            print(f"Error renaming {old_path} -> {new_path}: {e}", file=sys.stderr)
    
    print(f"Renamed {renamed} paths.")
    
    # Step 4: Final add
    subprocess.run(["git", "add", "-A"], check=True)
    
    status = subprocess.run(["git", "status", "--porcelain"], capture_output=True, text=True)
    lines = [l for l in status.stdout.strip().split("\n") if l]
    print(f"Changes staged: {len(lines)}")
    for l in lines[:20]:
        print(l)

if __name__ == "__main__":
    main()
