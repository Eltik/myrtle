#!/usr/bin/env python3
"""Minimal DPS calculator wrapper. Reads JSON test case from stdin, prints DPS to stdout."""
import sys, json, os
ark_dir = os.path.join(os.path.dirname(__file__), '..', 'external', 'ArknightsDpsCompare')
os.chdir(ark_dir)
sys.path.insert(0, '.')

from damagecalc.damage_formulas import *
from damagecalc.utils import PlotParameters

def calc(tc):
    name = tc['operator']
    skill = tc.get('skill', -1)
    module = tc.get('module', -1)
    defense = tc.get('defense', 0)
    res = tc.get('res', 0)
    fragile = tc.get('fragile', 0)
    def_shred_mult = tc.get('def_shred_mult', 1)
    def_shred_flat = tc.get('def_shred_flat', 0)
    res_shred_mult = tc.get('res_shred_mult', 1)
    res_shred_flat = tc.get('res_shred_flat', 0)

    pp = PlotParameters(skill=skill, module=module)
    try:
        op_class = globals()[name]
        op = op_class(pp)
    except Exception:
        return None

    shredded_def = max(0, (defense - def_shred_flat)) * def_shred_mult
    shredded_res = max(0, (res - res_shred_flat)) * res_shred_mult

    try:
        dps = float(op.skill_dps(shredded_def, shredded_res))
        dps *= (1 + fragile)
        return dps
    except Exception:
        return None

# Batch mode: read JSON array from stdin
cases = json.load(sys.stdin)
results = {}
for tc in cases:
    key = tc['key']
    dps = calc(tc)
    if dps is not None:
        results[key] = dps
json.dump(results, sys.stdout)
