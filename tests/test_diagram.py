# coding: utf-8
"""Behavioral checks for shared SVG layout/graph invariants (standard library)."""
import copy
import hashlib
from pathlib import Path
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'scripts'))
from diagram import ROOT, NS, render, resolve, validate, wrap_text
from validate_diagrams import audit_svg


def fixture():
    return {'schema_version':2,'id':'test','title':'Test','canvas':[800,480],'outer_margin':48,'flow_direction':'left-to-right',
            'regions':[{'id':'left','label':'Input','kind':'stage','geometry':[48,64,272,320]},
                       {'id':'right','label':'Output','kind':'stage','geometry':[384,64,368,320]}],
            'nodes':[{'id':'a','name':'Source','role':'Input data','geometry':[80,160,208,112],'region_id':'left','stage_id':'left','representation':'logical'},
                     {'id':'b','name':'Target','role':'Output data','geometry':[408,160,240,112],'region_id':'right','stage_id':'right','representation':'logical'}],
            'connections':[{'id':'ab','source':'a','target':'b','kind':'data','points':[[288,216],[408,216]]}]}


class DiagramTests(unittest.TestCase):
    def test_fits_members_without_mutating_input(self):
        m=fixture();m['regions'].append({'id':'product','label':'Product','kind':'product','parent_id':'left','fit':{'members':['a'],'padding':24}})
        result=resolve(m)
        self.assertEqual(result['regions'][-1]['geometry'],[56,88,256,208])
        self.assertNotIn('geometry',m['regions'][-1])
        validate(result)

    def test_nested_fit_and_cycle_rejection(self):
        m=fixture();m['regions'] += [{'id':'inside','label':'Inside','kind':'functional','fit':{'members':['a'],'padding':8}},
                                    {'id':'outside','label':'Outside','kind':'functional','fit':{'members':['inside'],'padding':8}}]
        result=resolve(m)
        self.assertLess(result['regions'][-1]['geometry'][0],result['regions'][-2]['geometry'][0])
        m['regions'][-2]['fit']['members']=['outside']
        with self.assertRaisesRegex(ValueError,'cycle'):resolve(m)

    def test_unknown_members_and_ownership_cycle(self):
        m=fixture();m['regions'][0]['fit']={'members':['missing']}
        with self.assertRaisesRegex(ValueError,'unknown fit member'):resolve(m)
        m=fixture();m['regions'][0]['parent_id']='right';m['regions'][1]['parent_id']='left'
        with self.assertRaisesRegex(ValueError,'cycle'):resolve(m)

    def test_wrong_parent_and_port_are_rejected(self):
        m=fixture();m['nodes'][0]['region_id']='right'
        with self.assertRaisesRegex(ValueError,'outside parent'):validate(m)
        m=fixture();m['connections'][0]['points'][0]=[200,216]
        with self.assertRaisesRegex(ValueError,'port'):validate(m)

    def test_bent_route_advances_but_folded_route_fails(self):
        m=fixture();m['connections'][0]['points']=[[288,216],[344,216],[344,240],[408,240]]
        validate(m)
        m['connections'][0]['points']=[[288,216],[360,216],[340,240],[408,240]]
        with self.assertRaisesRegex(ValueError,'doubles back'):validate(m)

    def test_real_return_flow_requires_purpose(self):
        m=fixture();m['connections']=[{'id':'return','source':'b','target':'a','kind':'data','points':[[408,216],[288,216]],'return_path':True}]
        with self.assertRaisesRegex(ValueError,'requires purpose'):validate(m)
        m['connections'][0]['purpose']='reverse_etl';validate(m)

    def test_svg_matches_graph_and_keeps_lines_behind_labels(self):
        m=fixture()
        with tempfile.TemporaryDirectory() as d:
            path=Path(d)/'diagram.svg';root=render(m,path);audit_svg(resolve(m),path)
            ids=[e.get('id') for e in root]
            self.assertLess(ids.index('regions'),ids.index('connections'))
            self.assertLess(ids.index('connections'),ids.index('nodes'))
            self.assertLess(ids.index('nodes'),ids.index('labels'))
            group=next(e for e in root if e.get('id')=='connections');group.clear();ET.ElementTree(root).write(path)
            with self.assertRaisesRegex(ValueError,'connections differ'):audit_svg(resolve(m),path)

    def test_shared_region_does_not_replace_stage_membership(self):
        m=fixture();m['nodes'][0]['stage_id']='right'
        with self.assertRaisesRegex(ValueError,'outside stage'):validate(m)

    def test_official_icon_is_embedded_without_changing_asset(self):
        m=fixture();asset='assets/icons/material-symbols/person.svg';p=ROOT/asset;before=hashlib.sha256(p.read_bytes()).digest()
        m['nodes'][0].update(icons=[{'asset':asset}],representation='horizontal')
        with tempfile.TemporaryDirectory() as d:
            root=render(m,Path(d)/'diagram.svg')
            image=next(e for e in root.iter() if e.tag=='{'+NS+'}image')
            self.assertEqual(image.get('data-asset'),asset)
        self.assertEqual(hashlib.sha256(p.read_bytes()).digest(),before)

    def test_connection_metadata_is_displayed_without_inventing_values(self):
        m=fixture();m['connections'][0].update(method='API',frequency='日次')
        with self.assertRaisesRegex(ValueError,'label_position'):validate(m)
        m['connections'][0]['label_position']=[348,132]
        with tempfile.TemporaryDirectory() as d:
            root=render(m,Path(d)/'diagram.svg')
            text=''.join(e.text or '' for e in root.iter() if e.tag=='{'+NS+'}text')
            self.assertIn('API / 日次',text)
        m['connections'][0]['junctions']=[[700,400]]
        with self.assertRaisesRegex(ValueError,'junction outside'):validate(m)

    def test_word_wrap_preserves_product_words_and_explicit_lines(self):
        self.assertEqual(wrap_text('Great Expectations',176,22),['Great','Expectations'])
        self.assertEqual(wrap_text('Cloud Storage',176,24),['Cloud Storage'])
        self.assertEqual(wrap_text('Raw\nMart',176,24),['Raw','Mart'])


if __name__=='__main__':unittest.main()
