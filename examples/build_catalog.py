"""Build the maintained component catalogue from reusable JSON fragments."""
import json
from pathlib import Path
import sys
ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
from components import instantiate


def build():
    directory = Path(__file__).parent
    path = directory / 'component-catalog/model.json'
    model = json.loads(path.read_text())
    parts = [instantiate(json.loads((directory / 'fragments' / (name + '.json')).read_text()), prefix, offset)
             for name, prefix, offset in [('file-receipt', 'receipt', (64, 112)),
                                          ('raw-mart', 'warehouse', (64, 352)),
                                          ('consumers', 'use', (944, 112))]]
    # Outer presentation frames are owned by the catalogue; internals by fragments.
    model['regions'] = [r for r in model['regions'] if r['id'] in ('receipt_group', 'consumers_group')]
    for key in ('nodes', 'connections'):
        model[key] = [item for part in parts for item in part[key]]
    model['regions'].extend(r for part in parts for r in part['regions'])
    path.write_text(json.dumps(model, ensure_ascii=False, indent=2) + '\n')


if __name__ == '__main__':
    build()
