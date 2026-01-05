#!/usr/bin/env python3
"""
DPS Comparison Test Runner

This script runs DPS calculations for all configured operators from test_config.json
and can generate expected_dps.json for efficient Rust testing.

Usage:
    python run_comparison.py                    # Run and display results
    python run_comparison.py --generate-expected # Generate expected_dps.json
    python run_comparison.py --output results.json # Save full results
"""

import sys
import os
import json
import argparse

# Add paths before importing python_dps_harness
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
BACKEND_DIR = os.path.dirname(os.path.dirname(SCRIPT_DIR))
sys.path.insert(0, os.path.join(BACKEND_DIR, 'scripts'))

from python_dps_harness import calculate_dps


def load_test_cases():
    """Load test cases from test_config.json."""
    config_path = os.path.join(SCRIPT_DIR, 'test_config.json')

    if not os.path.exists(config_path):
        print(f"Error: test_config.json not found at {config_path}")
        print("Run 'cargo run --bin generate-dps-tests' to generate it.")
        sys.exit(1)

    with open(config_path, 'r') as f:
        return json.load(f)


def make_test_key(operator: str, skill: int, module: int, defense: float, res: float) -> str:
    """Create a unique key for a test case."""
    return f"{operator}_s{skill}_m{module}_{int(defense)}_{int(res)}"


def run_all_tests(output_file=None, generate_expected=False):
    """Run all test cases and collect results."""
    test_cases = load_test_cases()
    results = []
    errors = []
    expected_dps = {}

    print(f"Running {len(test_cases)} test cases from test_config.json...")

    for i, tc in enumerate(test_cases):
        result = calculate_dps(
            operator_name=tc['operator'],
            defense=tc['defense'],
            res=tc['res'],
            skill=tc['skill'],
            module=tc['module'],
        )

        if 'error' in result:
            errors.append({**tc, 'error': result['error']})
            print(f"  [{i+1}/{len(test_cases)}] {tc['operator']} S{tc['skill']+1} - ERROR: {result['error']}")
        else:
            results.append(result)
            dps = result['dps']
            key = make_test_key(tc['operator'], tc['skill'], tc['module'], tc['defense'], tc['res'])
            expected_dps[key] = dps
            print(f"  [{i+1}/{len(test_cases)}] {tc['operator']} S{tc['skill']+1} def={tc['defense']:.0f} res={tc['res']:.0f} -> DPS: {dps:.2f}")

    print()
    print(f"Completed: {len(results)} successful, {len(errors)} errors")

    if generate_expected:
        expected_path = os.path.join(SCRIPT_DIR, 'expected_dps.json')
        with open(expected_path, 'w') as f:
            json.dump(expected_dps, f, indent=2, sort_keys=True)
        print(f"Expected DPS values saved to {expected_path}")

    if output_file:
        with open(output_file, 'w') as f:
            json.dump({
                'results': results,
                'errors': errors,
            }, f, indent=2)
        print(f"Full results saved to {output_file}")

    return results, errors


def main():
    parser = argparse.ArgumentParser(description='Run DPS comparison tests')
    parser.add_argument('--output', '-o', help='Output JSON file for full results')
    parser.add_argument('--generate-expected', action='store_true',
                        help='Generate expected_dps.json for Rust tests')
    args = parser.parse_args()

    results, errors = run_all_tests(args.output, args.generate_expected)

    sys.exit(0 if len(errors) == 0 else 1)


if __name__ == '__main__':
    main()
