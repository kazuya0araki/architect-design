"""Instantiate local diagram fragments; IDs and routes are isolated per instance."""
import copy
import re


def instantiate(fragment, instance_id, offset=(0, 0), labels=None):
    """Return model lists plus exported ports; input is never mutated.

    labels maps local node/region IDs to display labels. Ports identify nodes or
    regions; callers explicitly draw connections to their geometry boundaries.
    Composition never invents cross-fragment data flows or selects products.
    """
    if not re.fullmatch(r'[a-z][a-z0-9_]*', instance_id):
        raise ValueError('instance_id must be a lowercase identifier')
    result = copy.deepcopy(fragment)
    items = result.get('regions', []) + result.get('nodes', [])
    objects = items + result.get('connections', [])
    local_ids = [item['id'] for item in objects]
    if len(local_ids) != len(set(local_ids)):
        raise ValueError('duplicate fragment ID')
    ids = {key: instance_id + '_' + key for key in local_ids}
    dx, dy = offset
    def point(p):
        return [p[0] + dx, p[1] + dy]
    def reference(key):
        if key not in ids:
            raise ValueError('unknown fragment reference: ' + key)
        return ids[key]
    region_ids = {item['id'] for item in result.get('regions', [])}
    element_ids = {item['id'] for item in items}
    if any(key not in element_ids for key in result.get('ports', {}).values()):
        raise ValueError('unknown fragment reference in ports')
    labels = labels or {}
    if set(labels) - {item['id'] for item in items}:
        raise ValueError('unknown label override')
    for item in objects:
        local = item['id']
        item['id'] = ids[local]
        for key in ('parent_id', 'region_id', 'stage_id', 'source', 'target'):
            if item.get(key):
                item[key] = reference(item[key])
        if 'geometry' in item:
            item['geometry'] = point(item['geometry']) + item['geometry'][2:]
        if 'fit' in item:
            item['fit']['members'] = [reference(x) for x in item['fit']['members']]
        for key in ('points', 'junctions'):
            if key in item:
                item[key] = [point(p) for p in item[key]]
        if 'label_position' in item:
            item['label_position'] = point(item['label_position'])
        if local in labels:
            item['label' if local in region_ids else 'display_name'] = labels[local]
    result['ports'] = {name: reference(key) for name, key in result.get('ports', {}).items()}
    return result
