# coding: utf-8
"""Shared, offline SVG primitives. Explicit positions; content-fitted named regions."""
from __future__ import annotations
import argparse
import base64
import copy
import json
import math
import re
from pathlib import Path
import unicodedata
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
NS = 'http://www.w3.org/2000/svg'
ET.register_namespace('', NS)
TOKENS = json.loads((ROOT / 'assets/design-tokens.json').read_text())


def text_width(value, size):
    """Conservative layout estimate; final font metrics are checked in the browser."""
    return sum(1 if unicodedata.east_asian_width(c) in ('W', 'F') else .55 for c in value) * size


def wrap_text(value, width, size):
    """Keep Latin words intact; explicit newlines remain authoritative."""
    lines = []
    for paragraph in str(value).split('\n'):
        words = re.findall(r'[A-Za-z0-9_./-]+|\s+|[^A-Za-z0-9_./\s-]', paragraph)
        current = ''
        for word in words:
            if word.isspace() and not current:
                continue
            if text_width(word,size)>width:
                if current.strip():lines.append(current.rstrip());current=''
                chunks = max(2,math.ceil(text_width(word,size)/width))
                length = math.ceil(len(word)/chunks)
                parts = [word[i:i+length] for i in range(0,len(word),length)]
                lines.extend(parts[:-1]);current=parts[-1]
            elif current and text_width(current+word,size)>width:
                lines.append(current.rstrip());current=word.lstrip()
            else:
                current += word
        lines.append(current.rstrip())
    return lines


def contains(outer, inner, tolerance=.1):
    x, y, w, h = outer
    a, b, c, d = inner
    return x-tolerance <= a and y-tolerance <= b and a+c <= x+w+tolerance and b+d <= y+h+tolerance


def connection_label(connection):
    if 'label' in connection:
        return connection['label']
    return ' / '.join(str(connection[key]) for key in ('data', 'method', 'frequency') if connection.get(key))


def resolve(model):
    """Resolve fitted regions inside-out. Stages and ownership remain independent."""
    model = copy.deepcopy(model)
    regions = {r['id']: r for r in model.get('regions', [])}
    nodes = {n['id']: n for n in model.get('nodes', [])}
    active, done = set(), set()

    def region_geometry(id):
        if id in active:
            raise ValueError(f'region cycle: {id}')
        if id in done:
            return regions[id]['geometry']
        if id not in regions:
            raise ValueError(f'unknown region: {id}')
        r = regions[id]
        active.add(id)
        if r.get('parent_id'):
            # Check the ownership tree independently of content fitting.
            parent = r['parent_id']
            seen = {id}
            while parent:
                if parent in seen:
                    raise ValueError(f'region cycle: {id}')
                seen.add(parent)
                if parent not in regions:
                    raise ValueError(f'unknown region: {parent}')
                parent = regions[parent].get('parent_id')
        if 'fit' in r:
            members = r['fit'].get('members', [])
            if not members:
                raise ValueError(f'empty fitted region: {id}')
            geometries = []
            for member in members:
                if member in nodes:
                    geometries.append(nodes[member]['geometry'])
                elif member in regions:
                    geometries.append(region_geometry(member))
                else:
                    raise ValueError(f'unknown fit member: {member}')
            padding = r['fit'].get('padding', 24)
            header = r.get('header_height', 48)
            left = min(g[0] for g in geometries)-padding
            top = min(g[1] for g in geometries)-padding-header
            right = max(g[0]+g[2] for g in geometries)+padding
            bottom = max(g[1]+g[3] for g in geometries)+padding
            r['geometry'] = [left, top, right-left, bottom-top]
        if 'geometry' not in r:
            raise ValueError(f'missing region geometry: {id}')
        active.remove(id)
        done.add(id)
        return r['geometry']

    for id in regions:
        region_geometry(id)
    return model


def validate(model):
    errors = []
    if model.get('schema_version') != 2:
        errors.append('expected SVG model schema_version 2')
    regions, nodes = model.get('regions', []), model.get('nodes', [])
    ids = [x['id'] for x in regions+nodes]
    if len(ids) != len(set(ids)):
        errors.append('duplicate region/node ID')
    by_id = {x['id']: x for x in regions+nodes}
    region_by_id = {r['id']: r for r in regions}
    canvas = [0, 0, *model['canvas']]
    margin = model.get('outer_margin', 48)
    inner_canvas = [margin, margin, canvas[2]-2*margin, canvas[3]-2*margin]
    for item in regions+nodes:
        id, geometry = item['id'], item['geometry']
        if len(geometry) != 4 or not all(isinstance(v, (int,float)) and math.isfinite(v) for v in geometry) or min(geometry[2:]) <= 0:
            errors.append(f'invalid geometry: {id}')
            continue
        if not contains(inner_canvas, geometry):
            errors.append(f'outside outer margin: {id}')
        parent = item.get('parent_id') or item.get('region_id')
        if parent:
            if parent not in region_by_id:
                errors.append(f'unknown parent: {id} -> {parent}')
            elif not contains(region_by_id[parent]['geometry'], geometry):
                errors.append(f'outside parent: {id} -> {parent}')
        stage = item.get('stage_id')
        if stage:
            if stage not in region_by_id or region_by_id[stage]['kind'] != 'stage':
                errors.append(f'invalid stage: {id} -> {stage}')
            elif not contains(region_by_id[stage]['geometry'], geometry):
                errors.append(f'outside stage: {id} -> {stage}')
        if item in regions and item['kind'] not in TOKENS['region_styles']:
            errors.append(f'unknown region kind: {id}')
        if item in nodes and item.get('representation','logical') not in ('icon','horizontal','logical','data'):
            errors.append(f'unknown representation: {id}')
    connections = model.get('connections', [])
    connection_ids = [c['id'] for c in connections]
    if len(connection_ids) != len(set(connection_ids)) or set(connection_ids) & set(ids):
        errors.append('duplicate connection ID')
    for c in connections:
        if c['source'] not in by_id or c['target'] not in by_id:
            errors.append(f'unknown edge endpoint: {c["id"]}')
            continue
        if c['kind'] not in ('data','control'):
            errors.append(f'invalid connection kind: {c["id"]}')
        points = c.get('points', [])
        if len(points) < 2:
            errors.append(f'missing path: {c["id"]}')
            continue
        if any(not (0 <= x <= canvas[2] and 0 <= y <= canvas[3]) for x,y in points):
            errors.append(f'path outside canvas: {c["id"]}')
        for item_id, point in [(c['source'],points[0]),(c['target'],points[-1])]:
            x,y,w,h = by_id[item_id]['geometry']
            px,py = point
            on_edge = ((abs(px-x)<.1 or abs(px-x-w)<.1) and y-.1<=py<=y+h+.1) or ((abs(py-y)<.1 or abs(py-y-h)<.1) and x-.1<=px<=x+w+.1)
            if not on_edge:
                errors.append(f'path port outside endpoint: {c["id"]} -> {item_id}')
        if c['kind']=='data' and model.get('flow_direction')=='left-to-right' and not c.get('return_path'):
            source, target = by_id[c['source']]['geometry'], by_id[c['target']]['geometry']
            if source[0]+source[2] > target[0]+.1 or any(a[0] > b[0] for a,b in zip(points,points[1:])):
                errors.append(f'data flow doubles back: {c["id"]}')
        if c.get('return_path') and not c.get('purpose'):
            errors.append(f'return path requires purpose: {c["id"]}')
        if connection_label(c) and not c.get('label_position'):
            errors.append(f'label requires label_position: {c["id"]}')
        for px,py in c.get('junctions',[]):
            if not any(abs((b[0]-a[0])*(py-a[1])-(b[1]-a[1])*(px-a[0]))<.1 and min(a[0],b[0])-.1<=px<=max(a[0],b[0])+.1 and min(a[1],b[1])-.1<=py<=max(a[1],b[1])+.1 for a,b in zip(points,points[1:])):
                errors.append(f'junction outside path: {c["id"]}')
    if errors:
        raise ValueError('\n'.join(errors))


def render(model, destination):
    model = resolve(model)
    validate(model)
    colors, font, spacing = TOKENS['color'], TOKENS['font'], TOKENS['spacing']
    w,h = model['canvas']
    root = ET.Element(f'{{{NS}}}svg', {'width':str(w),'height':str(h),'viewBox':f'0 0 {w} {h}','role':'img','aria-labelledby':'title description','data-schema-version':'2'})
    def element(parent, tag, attrs=None, value=None):
        e = ET.SubElement(parent, f'{{{NS}}}{tag}', {k:str(v) for k,v in (attrs or {}).items()})
        e.text = value
        return e
    element(root,'title',{'id':'title'},model['title'])
    element(root,'desc',{'id':'description'},model.get('purpose','')+' '+model.get('brand_notice',''))
    element(root,'metadata',{'id':'model_metadata'},json.dumps({'id':model['id'],'status':model.get('status'),'source':model.get('source'),'omitted':model.get('omitted',[])},ensure_ascii=False))
    defs = element(root,'defs')
    marker = element(defs,'marker',{'id':'arrow','viewBox':'0 0 10 10','refX':9,'refY':5,'markerWidth':6,'markerHeight':6,'orient':'auto'})
    element(marker,'path',{'d':'M0 0L10 5L0 10Z','fill':colors['connector']})
    element(root,'style',value=f'text{{font-family:{font["family"]};fill:{colors["text"]}}}')
    element(root,'rect',{'width':w,'height':h,'fill':colors['canvas']})
    frames = element(root,'g',{'id':'regions'})
    edges = element(root,'g',{'id':'connections'})
    content = element(root,'g',{'id':'nodes'})
    labels = element(root,'g',{'id':'labels'})
    text_boxes = []

    def text(parent, id, x, baseline, value, size, width, centered=False, secondary=False, bold=False):
        lines = wrap_text(value, width, size)
        for i,line in enumerate(lines):
            y = baseline+i*(size+spacing['line_gap'])
            measured = text_width(line,size)
            left = x-measured/2 if centered else x
            box = [left,y-size,measured,size+2]
            text_boxes.append((id+'_'+str(i),box))
            element(parent,'text',{'id':id+'_'+str(i),'x':x,'y':y,'font-size':size,'font-weight':600 if bold else 400,'text-anchor':'middle' if centered else 'start','style':f'fill:{colors["secondary"] if secondary else colors["text"]}','data-text-box':json.dumps(box)},line)
        return baseline+len(lines)*(size+spacing['line_gap'])

    def icon(parent, id, asset, x, y, size):
        data = base64.b64encode((ROOT/asset).read_bytes()).decode()
        element(parent,'image',{'id':id,'x':x,'y':y,'width':size,'height':size,'preserveAspectRatio':'xMidYMid meet','href':'data:image/svg+xml;base64,'+data,'data-asset':asset})

    regions = {r['id']:r for r in model.get('regions',[])}
    def depth(r):
        return 1+depth(regions[r['parent_id']]) if r.get('parent_id') else 0
    for r in sorted(regions.values(),key=depth):
        style = TOKENS['region_styles'][r['kind']]
        x,y,rw,rh = r['geometry']
        header = r.get('header_height',spacing['header'])
        attrs = {'id':r['id'],'data-region-kind':r['kind'],'data-parent':r.get('parent_id',''),'data-geometry':json.dumps(r['geometry']),'x':x,'y':y,'width':rw,'height':rh,'fill':colors.get(style['fill'],style['fill']),'stroke':colors.get(style['stroke'],style['stroke']),'stroke-width':1,'rx':4}
        if style.get('dash'):attrs['stroke-dasharray']=style['dash']
        element(frames,'rect',attrs)
        if style['header']=='bar':
            element(frames,'rect',{'x':x+.5,'y':y+.5,'width':rw-1,'height':header-1,'fill':colors['heading']})
            element(frames,'path',{'d':f'M{x} {y+header}H{x+rw}','stroke':colors['border'],'fill':'none'})
        tx = x+16
        if r.get('asset'):
            icon(content,r['id']+'_icon',r['asset'],x+12,y+12,36)
            tx = x+60
        text(labels,r['id']+'_label',tx,y+32,r['label'],r.get('label_size',font['heading'] if r['kind']=='stage' else font['region']),rw-(tx-x)-12,bold=r['kind']=='stage')
    for c in model.get('connections',[]):
        g = element(edges,'g',{'id':c['id'],'data-source':c['source'],'data-target':c['target'],'data-kind':c['kind'],'data-purpose':c.get('purpose','')})
        attrs = {'d':'M'+' L'.join(f'{x} {y}' for x,y in c['points']),'fill':'none','stroke':colors['connector'],'stroke-width':1.5,'marker-end':'url(#arrow)'}
        if c['kind']=='control':attrs['stroke-dasharray']='6 5'
        element(g,'path',attrs)
        if c.get('junctions'):
            for x,y in c['junctions']:element(g,'circle',{'cx':x,'cy':y,'r':2.5,'fill':colors['connector']})
        label = connection_label(c)
        if label:
            x,y = c['label_position']
            label_width = c.get('label_width',192)
            lines = wrap_text(label,label_width,font['role'])
            measured = max(text_width(s,font['role']) for s in lines)
            element(labels,'rect',{'x':x-measured/2-4,'y':y-font['role']-2,'width':measured+8,'height':len(lines)*(font['role']+6),'fill':colors['canvas']})
            text(labels,c['id']+'_label',x,y,label,font['role'],label_width,True,True)
    for n in model['nodes']:
        x,y,nw,nh = n['geometry']
        group = element(content,'g',{'id':n['id'],'data-kind':'node','data-region':n.get('region_id',''),'data-stage':n.get('stage_id',''),'data-geometry':json.dumps(n['geometry']),'data-basis':n.get('basis',model.get('status','unspecified'))})
        element(group,'title',value=n['name']+' / '+n.get('role',''))
        name = n.get('display_name',n['name'])
        role = n.get('display_role',n.get('role',''))
        layout = n.get('representation','icon' if n.get('icons') else 'logical')
        icons = n.get('icons',[])
        size = n.get('icon_size',spacing['icon'])
        name_size,role_size = n.get('name_size',font['name']),n.get('role_size',font['role'])
        if layout=='icon':
            row_width = len(icons)*size+max(0,len(icons)-1)*12
            for i,a in enumerate(icons):icon(group,n['id']+'_icon_'+str(i),a['asset'],x+(nw-row_width)/2+i*(size+12),y,size)
            baseline = y+size+name_size+12
            baseline = text(labels,n['id']+'_name',x+nw/2,baseline,name,name_size,nw+16,True)
            if role:text(labels,n['id']+'_role',x+nw/2,baseline+2,role,role_size,nw+16,True,True)
        elif layout=='horizontal':
            for i,a in enumerate(icons):icon(group,n['id']+'_icon_'+str(i),a['asset'],x+i*(size+12),y,size)
            left = x+len(icons)*(size+12)
            baseline = text(labels,n['id']+'_name',left,y+name_size,name,name_size,nw-(left-x))
            if role:text(labels,n['id']+'_role',left,baseline+2,role,role_size,nw-(left-x),secondary=True)
        else:
            if layout=='data':
                d = f'M{x} {y+12}C{x} {y-4} {x+nw} {y-4} {x+nw} {y+12}V{y+nh-12}C{x+nw} {y+nh+4} {x} {y+nh+4} {x} {y+nh-12}Z M{x} {y+12}C{x} {y+28} {x+nw} {y+28} {x+nw} {y+12}'
                element(group,'path',{'d':d,'fill':colors['surface'],'stroke':colors['inner_border'],'stroke-width':1})
            else:
                element(group,'rect',{'x':x,'y':y,'width':nw,'height':nh,'fill':colors['surface'],'stroke':colors['inner_border'],'stroke-width':1,'rx':3})
            pad = 8
            names,roles = wrap_text(name,nw-2*pad,name_size),wrap_text(role,nw-2*pad,role_size) if role else []
            height = len(names)*(name_size+6)+len(roles)*(role_size+6)
            baseline = y+(nh-height)/2+name_size
            baseline = text(labels,n['id']+'_name',x+nw/2,baseline,name,name_size,nw-2*pad,True)
            if role:text(labels,n['id']+'_role',x+nw/2,baseline+2,role,role_size,nw-2*pad,True,True)
    # Label extents are included in machine checks and remain inspectable in SVG.
    for id,box in text_boxes:
        if not contains([0,0,w,h],box):raise ValueError(f'text outside canvas: {id}')
    ET.indent(root,space='  ')
    destination = Path(destination)
    destination.parent.mkdir(parents=True,exist_ok=True)
    ET.ElementTree(root).write(destination,encoding='utf-8',xml_declaration=True)
    return root


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('model',type=Path)
    parser.add_argument('--output',type=Path)
    args = parser.parse_args()
    model = json.loads(args.model.read_text())
    render(model,args.output or args.model.with_name('diagram.svg'))


if __name__=='__main__':
    main()
