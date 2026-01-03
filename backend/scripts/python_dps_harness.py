#!/usr/bin/env python3
"""
Python DPS Harness for testing Rust implementations against the original Python code.

This script loads the ArknightsDpsCompare damage_formulas module and calculates DPS
for given operator configurations, outputting results as JSON for comparison.

Usage:
    python python_dps_harness.py <operator_class> <defense> <res> [skill_index] [module_index] [potential]

    # Or in batch mode via stdin (JSON lines):
    echo '{"operator": "Blaze", "defense": 0, "res": 0, "skill": 2}' | python python_dps_harness.py --batch
"""

import sys
import os
import json
import argparse

# Add the ArknightsDpsCompare path
# Default to the submodule location relative to this script
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
BACKEND_DIR = os.path.dirname(SCRIPT_DIR)
DEFAULT_ARKDPS_PATH = os.path.join(BACKEND_DIR, 'external', 'ArknightsDpsCompare')
ARKDPS_PATH = os.environ.get('ARKDPS_PATH', DEFAULT_ARKDPS_PATH)
sys.path.insert(0, ARKDPS_PATH)

# Change to ArknightsDpsCompare directory so relative paths work
ORIGINAL_DIR = os.getcwd()
os.chdir(ARKDPS_PATH)

# Import after path setup
try:
    from damagecalc.damage_formulas import *
    from damagecalc.utils import PlotParameters
except ImportError as e:
    print(json.dumps({"error": f"Failed to import damage_formulas: {e}"}))
    sys.exit(1)
finally:
    # Change back to original directory
    os.chdir(ORIGINAL_DIR)


def get_operator_class(name: str):
    """Get operator class by name from the damage_formulas module."""
    # Handle special cases
    name_mapping = {
        "twelveF": "twelveF",
        "12F": "twelveF",
        "TwelveF": "twelveF",
    }

    class_name = name_mapping.get(name, name)

    # Try to get the class from globals
    if class_name in globals():
        return globals()[class_name]

    # Try case-insensitive search
    for key, value in globals().items():
        if key.lower() == class_name.lower() and isinstance(value, type) and issubclass(value, Operator):
            return value

    return None


def create_params(
    skill: int = -1,
    module: int = -1,
    module_lvl: int = 3,
    potential: int = -1,
    promotion: int = -1,
    level: int = -1,
    mastery: int = -1,
    trust: int = 100,
    targets: int = 1,
    conditionals: dict = None,
) -> PlotParameters:
    """Create PlotParameters for the operator."""
    pp = PlotParameters()
    pp.skill = skill
    pp.module = module
    pp.module_lvl = module_lvl
    pp.pot = potential
    pp.promotion = promotion
    pp.level = level
    pp.mastery = mastery
    pp.trust = trust
    pp.targets = targets
    pp.sp_boost = 0
    pp.base_buffs = [1, 0]  # [atk_multiplier, atk_flat]
    pp.buffs = [0, 0, 0, 0]  # [atk%, atk_flat, aspd, fragile]
    pp.shred = [1, 0, 1, 0]  # [def%, def_flat, res%, res_flat]
    pp.mul_add = [1, 0]

    # Set conditionals
    if conditionals:
        pp.conditionals = [
            conditionals.get('trait', True),
            conditionals.get('talent', True),
            conditionals.get('talent2', True),
            conditionals.get('skill', True),
            conditionals.get('module', True),
        ]
    else:
        pp.conditionals = [True, True, True, True, True]
    pp.allCond = True

    return pp


def calculate_dps(
    operator_name: str,
    defense: float,
    res: float,
    skill: int = -1,
    module: int = -1,
    module_lvl: int = 3,
    potential: int = -1,
    promotion: int = -1,
    level: int = -1,
    mastery: int = -1,
    trust: int = 100,
    targets: int = 1,
    conditionals: dict = None,
) -> dict:
    """Calculate DPS for an operator configuration."""

    operator_class = get_operator_class(operator_name)
    if operator_class is None:
        return {"error": f"Unknown operator: {operator_name}"}

    try:
        params = create_params(
            skill=skill,
            module=module,
            module_lvl=module_lvl,
            potential=potential,
            promotion=promotion,
            level=level,
            mastery=mastery,
            trust=trust,
            targets=targets,
            conditionals=conditionals,
        )

        # Create the operator instance
        op = operator_class(params)

        # Calculate DPS
        # Note: In the Python code, defense and res can be numpy arrays or scalars
        dps = op.skill_dps(defense, res)

        # Handle numpy array result (take first element if array)
        if hasattr(dps, '__iter__') and not isinstance(dps, (str, bytes)):
            dps = float(dps.flat[0]) if hasattr(dps, 'flat') else float(dps[0])
        else:
            dps = float(dps)

        return {
            "operator": operator_name,
            "name": op.get_name(),
            "defense": defense,
            "res": res,
            "skill": skill,
            "module": module,
            "dps": dps,
            "atk": float(op.atk),
            "attack_speed": float(op.attack_speed),
            "attack_interval": float(op.atk_interval),
        }

    except Exception as e:
        return {
            "error": str(e),
            "operator": operator_name,
            "defense": defense,
            "res": res,
        }


def run_batch_mode():
    """Read JSON lines from stdin and output DPS calculations."""
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue

        try:
            config = json.loads(line)
            result = calculate_dps(
                operator_name=config.get('operator', ''),
                defense=config.get('defense', 0),
                res=config.get('res', 0),
                skill=config.get('skill', -1),
                module=config.get('module', -1),
                module_lvl=config.get('module_lvl', 3),
                potential=config.get('potential', -1),
                promotion=config.get('promotion', -1),
                level=config.get('level', -1),
                mastery=config.get('mastery', -1),
                trust=config.get('trust', 100),
                targets=config.get('targets', 1),
                conditionals=config.get('conditionals'),
            )
            print(json.dumps(result))
        except json.JSONDecodeError as e:
            print(json.dumps({"error": f"Invalid JSON: {e}"}))


def run_single_calculation(args):
    """Run a single DPS calculation from command line arguments."""
    result = calculate_dps(
        operator_name=args.operator,
        defense=args.defense,
        res=args.res,
        skill=args.skill,
        module=args.module,
        module_lvl=args.module_lvl,
        potential=args.potential,
        promotion=args.promotion,
        level=args.level,
        mastery=args.mastery,
        trust=args.trust,
        targets=args.targets,
    )
    print(json.dumps(result, indent=2))


def list_operators():
    """List all available operator classes."""
    operators = []
    for name, obj in globals().items():
        if isinstance(obj, type) and issubclass(obj, Operator) and obj != Operator:
            operators.append(name)
    operators.sort()
    print(json.dumps({"operators": operators}))


def main():
    parser = argparse.ArgumentParser(description='Calculate DPS using ArknightsDpsCompare')
    parser.add_argument('--batch', action='store_true', help='Read JSON configurations from stdin')
    parser.add_argument('--list', action='store_true', help='List all available operators')
    parser.add_argument('operator', nargs='?', help='Operator class name')
    parser.add_argument('defense', nargs='?', type=float, default=0, help='Enemy defense')
    parser.add_argument('res', nargs='?', type=float, default=0, help='Enemy resistance')
    parser.add_argument('--skill', '-s', type=int, default=-1, help='Skill index (0, 1, 2)')
    parser.add_argument('--module', '-m', type=int, default=-1, help='Module index')
    parser.add_argument('--module-lvl', type=int, default=3, help='Module level')
    parser.add_argument('--potential', '-p', type=int, default=-1, help='Potential (1-6)')
    parser.add_argument('--promotion', type=int, default=-1, help='Elite level (0, 1, 2)')
    parser.add_argument('--level', type=int, default=-1, help='Operator level')
    parser.add_argument('--mastery', type=int, default=-1, help='Mastery level')
    parser.add_argument('--trust', type=int, default=100, help='Trust level')
    parser.add_argument('--targets', type=int, default=1, help='Number of targets')

    args = parser.parse_args()

    if args.list:
        list_operators()
    elif args.batch:
        run_batch_mode()
    elif args.operator:
        run_single_calculation(args)
    else:
        parser.print_help()


if __name__ == '__main__':
    main()
