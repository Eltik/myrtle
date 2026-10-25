#!/usr/bin/env python3
"""Derive the polarity of every DPS / HPS conditional from the Python reference.

A conditional's `name` is the label the reference appends to the operator's
name in `__init__`, on a line that also reads the flag:

    if self.skill == 1 and not self.trait_dmg: self.name += "rangedAtk"

Here the label describes the flag OFF: the switch is INVERTED. This script
finds, for every conditional in `operator_formulas.json` and
`heal_formulas.json`, the `__init__` line(s) the generator took the label from
(same flag test and same label regex as `extract_conditionals` in
`src/bin/generate_dps.rs`) and records:

    true   the label sits under `not self.<flag>` on every such line
    false  the label sits under `self.<flag>` on every such line
    null   unresolved: no such line, or lines that disagree, or a line that
           reads the flag both ways; `HAND` below settles some by reading

It writes `src/dps/config/conditional_polarity.json` and never touches the
generated formula files, which a `generate-dps` run rewrites. Hand-layer
operators (`src/dps/custom/hand/formulas.json`) are merged over the generated
tables the way `engine::load_formulas` merges them; they have no Python class,
so their polarity comes from `HAND` below.

Run from `backend/`:  python3 scripts/dps_conditional_polarity.py [--check]
`--check` exits 1 when the checked-in file differs from a fresh derivation.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REF = ROOT / "external/ArknightsDpsCompare/damagecalc"
CONFIG = ROOT / "src/dps/config"
OUT = CONFIG / "conditional_polarity.json"
HAND_FORMULAS = ROOT / "src/dps/custom/hand/formulas.json"

# Section name -> (formula table, Python source). An operator can sit in both
# tables (Skadi the Corrupting Heart), so each table has its own section.
SOURCES = [
    ("dps", "operator_formulas.json", REF / "damage_formulas.py"),
    ("hps", "heal_formulas.json", REF / "healing_formulas.py"),
]

FLAGS = {
    "trait": "trait_dmg",
    "talent": "talent_dmg",
    "talent2": "talent2_dmg",
    "skill": "skill_dmg",
    "module": "module_dmg",
}

# The generator's label regex, verbatim.
LABEL = re.compile(r"""self\.name\s*\+=\s*[f"]?\s*"?\s*(\w+)""")

# Settled by reading the Python, for conditionals the line scan cannot decide.
# Key: "<operator id>|<type>|<name>" -> inverted (DPS table entries). Each entry carries why.
HAND: dict[str, bool] = {
    # Bellone (`custom/hand/bellone.rs`): talent_damage true takes talent 1's max
    # damage bonus (target below 20% HP); module_damage true adds the FGT-Y trait ASPD
    # while she is above 50% HP. Both labels describe the flag ON.
    "char_4037_demetr|talent|vsLowHp": False,
    "char_4037_demetr|module|above50Hp": False,
    # Zima the Raging Tide (`custom/hand/zima_alter.rs`): skill_damage true is S2's
    # second activation onward; module_damage true the HAM-X x1.15 (Odda's `3inRange`).
    # Both labels describe the flag ON.
    "char_1051_headb2|skill|2ndActivation": False,
    "char_1051_headb2|module|3inRange": False,
    # Ch'en the Dawnstreak (`custom/hand/chen_dawnstreak.rs`): module_damage true adds
    # the AFT-X trait ASPD when not blocking. ON label.
    "char_1050_chen3|module|NotBlocking": False,
    # Wang (`custom/hand/wang.rs`): talent2_damage true puts every stone in a chain at
    # max talent 2 stacks (Dorothy's `maxTalent2`). ON label.
    "char_2027_wang|talent2|maxTalent2": False,
    # Viy (`custom/hand/viy.rs`): trait_damage true fires every attack as a stored
    # charge, talent_damage true releases them as Transposed Energy, talent2_damage true
    # adds the Arts of War DoT, skill_damage true takes 9 S2 stacks. All describe ON.
    "char_4226_veen|trait|storedCharges": False,
    "char_4226_veen|talent|transposedEnergy": False,
    "char_4226_veen|talent2|artsOfWarDot": False,
    "char_4226_veen|skill|maxStacks": False,
    # Mantra (`custom/hand/mantra.rs`): skill_damage true averages the Nervous
    # Impairment cycle (ON label); trait_damage true is the 1000 gauge, so `vsBoss`
    # labels the flag OFF (as Tragodia's `vsBoss`).
    "char_4204_mantra|skill|avgNerv": False,
    "char_4204_mantra|trait|vsBoss": True,
    # Mantra: talent_damage true adds talent 1's Elemental damage per Paralysis
    # interrupt from her own bursts' stacks (the target attacks). ON label.
    "char_4204_mantra|talent|targetAttacks": False,
    # SilverAsh the Reignfrost (`custom/hand/silverash_alter.rs`): skill_damage true
    # applies his own S3 Fragile. ON label.
    "char_1045_svash2|skill|withFragile": False,
    # Closure (`custom/hand/closure.rs`): trait_damage true is the x1.5 against enemies
    # her Reinforcements block. ON label.
    "char_4228_closur|trait|vsBlocked": False,
    # Angelina the Mellow Wish is in the hand layer (`custom/hand/
    # angelina_alter.rs`, no Python class): talent_damage true takes the mass <= 3 scale,
    # the Weightless target; talent2_damage true adds the Liftoff ATK buff.
    # Both labels describe the flag ON.
    "char_1015_aglna2|talent|weightlessTarget": False,
    "char_1015_aglna2|talent2|liftoffSelfBuff": False,
    # Astgenne the Lightchaser (`custom/hand/astgenne_lightchaser.rs`): talent_damage
    # true takes talent 1 at full Slow stacks, talent2_damage true the 7 s Fragile.
    # Both labels describe the flag ON.
    "char_1047_halo2|talent|maxSlowStacks": False,
    "char_1047_halo2|talent2|fragileAfter7s": False,
}
# Left unresolved on purpose: Diamante (`char_499_kaitou`) talent
# `avgNecrosis` labels BOTH states, " avgNecrosis(vsBoss)" under
# `not self.talent_dmg and not self.skill_dmg` and " avgNecrosis(nonBoss)" under
# `self.talent_dmg ^ self.skill_dmg`, so no single polarity is right.


def class_bodies(source: str) -> dict[str, str]:
    bodies: dict[str, str] = {}
    current = None
    lines: list[str] = []
    for line in source.splitlines():
        m = re.match(r"class (\w+)\(", line)
        if m:
            if current:
                bodies[current] = "\n".join(lines)
            current, lines = m.group(1), []
        elif current:
            lines.append(line)
    if current:
        bodies[current] = "\n".join(lines)
    return bodies


def init_body(class_body: str) -> list[str]:
    """`__init__` lines, stripped, the way the generator reads them."""
    out: list[str] = []
    found = False
    base = 0
    for line in class_body.splitlines():
        stripped = line.lstrip()
        if stripped.startswith("def __init__("):
            found = True
            base = len(line) - len(stripped)
            continue
        if found:
            if not stripped:
                continue
            if len(line) - len(stripped) <= base:
                break
            out.append(stripped)
    return out


def line_polarity(line: str, flag: str) -> str | None:
    """'inverted' / 'on' for one line, None when it reads the flag both ways."""
    reads = re.findall(r"(\bnot\s+)?self\." + flag + r"\b", line)
    kinds = {"inverted" if neg else "on" for neg in reads}
    return kinds.pop() if len(kinds) == 1 else None


def derive() -> tuple[dict, dict]:
    out: dict[str, dict[str, dict[str, bool | None]]] = {}
    counts = {"on": 0, "inverted": 0, "unresolved": 0, "hand": 0}
    for section, formulas_file, py in SOURCES:
        out[section] = {}
        formulas = json.loads((CONFIG / formulas_file).read_text())
        formulas.update(json.loads(HAND_FORMULAS.read_text())[section])
        bodies = class_bodies(py.read_text())
        for op_id in sorted(formulas):
            f = formulas[op_id]
            if not f["conditionals"]:
                continue
            lines = init_body(bodies.get(f["class_name"], ""))
            entry: dict[str, bool | None] = {}
            for c in f["conditionals"]:
                flag = FLAGS[c["type"]]
                found = set()
                for line in lines:
                    m = LABEL.search(line)
                    if flag in line and m and m.group(1) == c["name"]:
                        found.add(line_polarity(line, flag))
                key = f"{c['type']}|{c['name']}"
                hand_key = f"{op_id}|{key}"
                if section == "dps" and hand_key in HAND:
                    value = HAND[hand_key]
                    counts["hand"] += 1
                elif len(found) == 1 and None not in found:
                    value = found.pop() == "inverted"
                else:
                    value = None
                counts["unresolved" if value is None else ("inverted" if value else "on")] += 1
                entry[key] = value
            out[section][op_id] = entry
    return out, counts


def main() -> int:
    derived, counts = derive()
    text = json.dumps(derived, indent=1, sort_keys=True) + "\n"
    if "--check" in sys.argv:
        same = OUT.exists() and OUT.read_text() == text
        print("up to date" if same else f"{OUT} is stale")
        return 0 if same else 1
    OUT.write_text(text)
    print(json.dumps(counts))
    return 0


if __name__ == "__main__":
    sys.exit(main())
