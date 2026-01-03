#!/usr/bin/env python3
"""
DPS Comparison Test Runner

This script runs DPS calculations for all configured operators from test_config.json
and outputs reference values that can be used in Rust tests.

Usage:
    python run_comparison.py [--output results.json]
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


def run_all_tests(output_file=None):
    """Run all test cases and collect results."""
    test_cases = load_test_cases()
    results = []
    errors = []

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
            errors.append(result)
            print(f"  [{i+1}/{len(test_cases)}] {tc['operator']} S{tc['skill']+1} - ERROR: {result['error']}")
        else:
            results.append(result)
            print(f"  [{i+1}/{len(test_cases)}] {tc['operator']} S{tc['skill']+1} def={tc['defense']:.0f} res={tc['res']:.0f} -> DPS: {result['dps']:.2f}")

    print()
    print(f"Completed: {len(results)} successful, {len(errors)} errors")

    if output_file:
        with open(output_file, 'w') as f:
            json.dump({
                'results': results,
                'errors': errors,
            }, f, indent=2)
        print(f"Results saved to {output_file}")

    return results, errors


def generate_rust_expected_values(results):
    """Generate Rust code with expected values from Python results."""
    print("\n// Expected DPS values from Python reference implementation")
    print("// Copy these into your Rust tests\n")

    for r in results:
        skill_name = f"s{r.get('skill', 0) + 1}" if r.get('skill', -1) >= 0 else "base"
        print(f"// {r['operator']} {skill_name} def={r['defense']:.0f} res={r['res']:.0f}")
        print(f"// Expected DPS: {r['dps']:.2f}")
        print()


def main():
    parser = argparse.ArgumentParser(description='Run DPS comparison tests')
    parser.add_argument('--output', '-o', help='Output JSON file for results')
    parser.add_argument('--rust-values', action='store_true', help='Generate Rust expected values')
    args = parser.parse_args()

    results, errors = run_all_tests(args.output)

    if args.rust_values:
        generate_rust_expected_values(results)

    sys.exit(0 if len(errors) == 0 else 1)


if __name__ == '__main__':
    main()
