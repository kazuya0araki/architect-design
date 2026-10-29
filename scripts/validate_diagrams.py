# coding: utf-8
"""Validate maintained v2 models and their SVGs; legacy diagrams are left untouched."""
from pathlib import Path
import argparse
import json
import xml.etree.ElementTree as ET
from diagram import ROOT, NS, contains, resolve, validate


def model_files():
    for directory in (ROOT/'examples', ROOT/'architectures'):
        for path in sorted(directory.rglob('*.json')):
            if path.name=='model.json' or path.name.endswith('.model.json'):
                model = json.loads(path.read_text())
                if model.get('schema_version')==2:
                    yield path,model


def svg_path(path,model):
    if path.name.endswith('.model.json'):
        return path.with_name(path.name.replace('.model.json','.svg'))
    return path.with_name('overview.svg' if path.parent.name=='manaslu-v2' else 'diagram.svg')


def intersect(a,b,tolerance=1):
    return min(a[0]+a[2],b[0]+b[2])-max(a[0],b[0])>tolerance and min(a[1]+a[3],b[1]+b[3])-max(a[1],b[1])>tolerance


def audit_svg(model,path):
    tree = ET.parse(path)
    elements = list(tree.iter())
    ids = [e.get('id') for e in elements if e.get('id')]
    errors = []
    if len(ids)!=len(set(ids)):errors.append('duplicate SVG ID')
    nodes = {n['id']:n for n in model['nodes']}
    regions = {r['id']:r for r in model.get('regions',[])}
    actual_nodes = {e.get('id') for e in elements if e.get('data-kind')=='node'}
    if actual_nodes!=set(nodes):errors.append('SVG nodes differ from model')
    actual_edges = {(e.get('id'),e.get('data-source'),e.get('data-target'),e.get('data-kind')) for e in elements if e.get('data-source')}
    expected_edges = {(c['id'],c['source'],c['target'],c['kind']) for c in model.get('connections',[])}
    if actual_edges!=expected_edges:errors.append('SVG connections differ from model')
    actual_regions = {e.get('id') for e in elements if e.get('data-region-kind')}
    if actual_regions!=set(regions):errors.append('SVG regions differ from model')
    labels = []
    margin = model.get('outer_margin',48)
    w,h = model['canvas']
    for e in elements:
        if not e.get('data-text-box'):continue
        id,box = e.get('id'),json.loads(e.get('data-text-box'))
        labels.append((id,box))
        if not contains([margin-8,margin-8,w-2*margin+16,h-2*margin+16],box):errors.append('text outside outer margin: '+id)
        for node_id,n in nodes.items():
            if id.startswith(node_id+'_name_') or id.startswith(node_id+'_role_'):
                x,y,nw,nh=n['geometry']
                if not contains([x-8,y-2,nw+16,nh+4],box):errors.append('text outside node: '+id)
        for region_id,r in regions.items():
            if id.startswith(region_id+'_label_') and not contains(r['geometry'],box):errors.append('text outside region: '+id)
    for i,(id,box) in enumerate(labels):
        for other,rect in labels[i+1:]:
            if intersect(box,rect):errors.append(f'overlapping text: {id} / {other}')
    for c in model.get('connections',[]):
        for a,b in zip(c['points'],c['points'][1:]):
            if a[0]==b[0]:line=[a[0]-.5,min(a[1],b[1]),1,abs(a[1]-b[1])]
            elif a[1]==b[1]:line=[min(a[0],b[0]),a[1]-.5,abs(a[0]-b[0]),1]
            else:continue
            for id,box in labels:
                if not id.startswith(c['id']+'_label_') and intersect(line,box,tolerance=.2):errors.append(f'edge crosses text: {c["id"]} / {id}')
    if errors:raise ValueError('\n'.join(errors))


def main():
    count=0
    for path,raw in model_files():
        model=resolve(raw)
        try:
            validate(model)
            audit_svg(model,svg_path(path,model))
        except (ValueError,FileNotFoundError) as exc:
            raise SystemExit(f'{path.relative_to(ROOT)}:\n{exc}')
        count+=1
    print(f'PASS: {count} v2 diagrams; regions, ports, rightward flow, SVG parity, estimated text bounds and intersections')


if __name__=='__main__':main()
