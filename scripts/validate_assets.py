"""Validate local icon inventory and example references; does not fetch updates.

Run from any directory: python3 scripts/validate_assets.py
"""
import base64
import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]


def validate():
    manifest = json.loads((ROOT / 'assets/icons/manifest.json').read_text())
    entries = {}
    errors = []
    for item in manifest['assets']:
        relative = item['path']
        if relative in entries:
            errors.append(f'duplicate manifest path: {relative}')
        entries[relative] = item
        file = ROOT / relative
        if not file.is_file():
            errors.append(f'missing asset: {relative}')
            continue
        if hashlib.sha256(file.read_bytes()).hexdigest() != item['sha256']:
            errors.append(f'SHA-256 mismatch: {relative}')
        try:
            if ET.parse(file).getroot().tag != '{http://www.w3.org/2000/svg}svg':
                errors.append(f'not an SVG: {relative}')
        except ET.ParseError as exc:
            errors.append(f'invalid SVG: {relative}: {exc}')
    actual = {p.relative_to(ROOT).as_posix() for p in (ROOT / 'assets/icons').rglob('*.svg')}
    errors.extend(f'unregistered asset: {p}' for p in sorted(actual - entries.keys()))
    google = [a for a in entries.values() if a['provider'] == 'Google Cloud']
    for item in google:
        collection = item.get('collection')
        if collection not in ('core', 'categories'):
            errors.append(f'unsupported Google Cloud collection: {item["path"]}')
        elif not item['path'].startswith(f'assets/icons/google-cloud/{collection}/'):
            errors.append(f'noncanonical Google Cloud path: {item["path"]}')
    collections = [c for c in manifest.get('collections', []) if c['provider'] == 'Google Cloud']
    if {c['collection'] for c in collections} != {'core', 'categories'}:
        errors.append('Google Cloud collection inventory must contain core and categories')
    for collection in collections:
        items = [a for a in google if a['collection'] == collection['collection']]
        if len(items) != collection['svg_count']:
            errors.append(f'collection count mismatch: {collection["collection"]}')
        if any(a['archive_sha256'] != collection['archive_sha256'] for a in items):
            errors.append(f'archive hash mismatch: {collection["collection"]}')

    def check_reference(relative):
        if relative not in entries or not (ROOT / relative).is_file():
            errors.append(f'invalid example asset reference: {relative}')
            return None
        return (ROOT / relative).read_bytes()

    def walk(value):
        if isinstance(value, dict):
            if 'asset' in value:
                check_reference(value['asset'])
            for child in value.values():
                walk(child)
        elif isinstance(value, list):
            for child in value:
                walk(child)

    for file in [p for base in ('examples','architectures') for p in (ROOT/base).rglob('*.json') if p.name=='model.json' or p.name.endswith('.model.json')]:
        walk(json.loads(file.read_text()))
    for file in [p for base in ('examples','architectures') for p in (ROOT/base).rglob('*.svg')]:
        for element in ET.parse(file).iter():
            if element.tag == '{http://www.w3.org/2000/svg}image':
                data = check_reference(element.get('data-asset', ''))
                try:
                    payload = base64.b64decode(element.get('href', '').split(',', 1)[1], validate=True)
                    if data is not None and payload != data:
                        errors.append(f'embedded SVG asset mismatch: {file.relative_to(ROOT)}')
                except (ValueError, IndexError) as exc:
                    errors.append(f'invalid SVG image embedding: {file.relative_to(ROOT)}: {exc}')
    registered_hashes = {item['sha256'] for item in entries.values()}
    for file in [p for base in ('examples','architectures') for p in (ROOT/base).rglob('*.drawio')]:
        for cell in ET.parse(file).iter('mxCell'):
            style = cell.get('style', '')
            prefix = 'image=data:image/svg+xml,'
            if prefix in style:
                try:
                    payload = base64.b64decode(style.split(prefix, 1)[1].split(';', 1)[0], validate=True)
                    if hashlib.sha256(payload).hexdigest() not in registered_hashes:
                        errors.append(f'unregistered draw.io image: {file.relative_to(ROOT)}')
                except ValueError as exc:
                    errors.append(f'invalid draw.io image embedding: {file.relative_to(ROOT)}: {exc}')
    if errors:
        raise SystemExit('\n'.join(errors))
    print(f'PASS: {len(entries)} icons; manifest/files/SHA-256/SVG/collections/example/architecture references and SVG/draw.io embeddings')


if __name__ == '__main__':
    validate()
