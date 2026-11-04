"""Reuse must preserve graph meaning and isolate each instance."""
import copy
import json
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
from components import instantiate
from diagram import resolve, validate


class ComponentTests(unittest.TestCase):
    def test_repeated_instances_have_distinct_ids_and_valid_paths(self):
        fragment = json.loads((ROOT / 'examples/fragments/file-receipt.json').read_text())
        before = copy.deepcopy(fragment)
        first = instantiate(fragment, 'first', (64, 64), {'file': '売上CSV'})
        second = instantiate(fragment, 'second', (64, 320))
        model = dict(schema_version=2, canvas=[960, 640], regions=[],
                     nodes=first['nodes'] + second['nodes'],
                     connections=first['connections'] + second['connections'],
                     flow_direction='left-to-right')
        validate(resolve(model))
        self.assertEqual(fragment, before)
        self.assertEqual(first['ports']['output'], 'first_storage')
        self.assertEqual(first['nodes'][0]['name'], '受領ファイル')
        self.assertEqual(first['nodes'][0]['display_name'], '売上CSV')
        self.assertNotIn('display_name', second['nodes'][0])
        self.assertEqual(first['connections'][0]['points'][0], [256, 176])

    def test_nested_fit_and_connection_annotations_move_together(self):
        fragment = dict(regions=[dict(id='r', kind='product', label='Store', fit={'members':['a'], 'padding':24})],
                        nodes=[dict(id='a', name='A', role='', geometry=[0,80,192,96], region_id='r')],
                        connections=[dict(id='c', source='a', target='r', kind='control',
                                          points=[[0,100],[0,200]], label_position=[0,50], junctions=[[0,150]])],
                        ports={'store':'r'})
        result = instantiate(fragment, 'one', (64, 64))
        self.assertEqual(result['regions'][0]['fit']['members'], ['one_a'])
        self.assertEqual(result['nodes'][0]['region_id'], 'one_r')
        self.assertEqual(result['connections'][0]['target'], 'one_r')
        self.assertEqual(result['connections'][0]['label_position'], [64,114])
        self.assertEqual(result['connections'][0]['junctions'], [[64,214]])
        self.assertIn('geometry', resolve(result)['regions'][0])

    def test_invalid_local_references_and_overrides_are_rejected(self):
        fragment = dict(regions=[], nodes=[dict(id='a', name='A', role='', geometry=[0,0,192,96])],
                        connections=[], ports={'output':'missing'})
        with self.assertRaisesRegex(ValueError, 'unknown fragment reference'):
            instantiate(fragment, 'one')
        fragment['ports'] = {}
        with self.assertRaisesRegex(ValueError, 'unknown label override'):
            instantiate(fragment, 'one', labels={'missing':'X'})
        with self.assertRaisesRegex(ValueError, 'instance_id'):
            instantiate(fragment, '../bad')
        fragment['nodes'].append(copy.deepcopy(fragment['nodes'][0]))
        with self.assertRaisesRegex(ValueError, 'duplicate fragment ID'):
            instantiate(fragment, 'one')


if __name__ == '__main__':
    unittest.main()
