# coding: utf-8
"""Render maintained example SVGs with the shared primitives. No network access."""
from pathlib import Path
import json
import sys
ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'scripts'))
from diagram import render

if __name__=='__main__':
    for path in sorted(Path(__file__).parent.glob('*/model.json')):
        model = json.loads(path.read_text())
        if model.get('schema_version')==2:
            render(model,path.with_name('diagram.svg'))
            print(path.parent.name)
