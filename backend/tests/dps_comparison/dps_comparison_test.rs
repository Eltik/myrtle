//! DPS Comparison Tests
//!
//! These tests compare Rust DPS calculations against the Python reference implementation.
//!
//! Run with: cargo test --test dps_comparison
//!
//! Note: The Python harness must be set up and ARKDPS_PATH environment variable must be set.

use std::process::Command;
use std::io::Write;

/// Tolerance for DPS comparison (percentage difference allowed)
const TOLERANCE_PERCENT: f64 = 0.01; // 1%

/// Absolute tolerance for very small values
const TOLERANCE_ABSOLUTE: f64 = 1.0;

#[derive(Debug, serde::Deserialize)]
struct PythonDpsResult {
    operator: Option<String>,
    dps: Option<f64>,
    error: Option<String>,
    defense: Option<f64>,
    res: Option<f64>,
}

fn get_python_dps(operator: &str, defense: f64, res: f64, skill: i32, module: i32) -> Result<f64, String> {
    let harness_path = std::env::var("DPS_HARNESS_PATH")
        .unwrap_or_else(|_| "scripts/python_dps_harness.py".to_string());

    let arkdps_path = std::env::var("ARKDPS_PATH")
        .unwrap_or_else(|_| "/Users/eltik/Documents/Coding/ArknightsDpsCompare".to_string());

    let config = serde_json::json!({
        "operator": operator,
        "defense": defense,
        "res": res,
        "skill": skill,
        "module": module,
    });

    let output = Command::new("python3")
        .arg(&harness_path)
        .arg("--batch")
        .env("ARKDPS_PATH", arkdps_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(stdin) = child.stdin.as_mut() {
                writeln!(stdin, "{}", config.to_string())?;
            }
            child.wait_with_output()
        })
        .map_err(|e| format!("Failed to run Python harness: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python harness failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: PythonDpsResult = serde_json::from_str(stdout.trim())
        .map_err(|e| format!("Failed to parse Python output: {} (output: {})", e, stdout))?;

    if let Some(error) = result.error {
        return Err(error);
    }

    result.dps.ok_or_else(|| "No DPS value in response".to_string())
}

fn compare_dps(rust_dps: f64, python_dps: f64, test_name: &str) -> Result<(), String> {
    let diff = (rust_dps - python_dps).abs();
    let percent_diff = if python_dps.abs() > 0.01 {
        diff / python_dps.abs()
    } else {
        diff
    };

    if diff <= TOLERANCE_ABSOLUTE || percent_diff <= TOLERANCE_PERCENT {
        Ok(())
    } else {
        Err(format!(
            "{}: DPS mismatch - Rust: {:.2}, Python: {:.2}, Diff: {:.2} ({:.2}%)",
            test_name, rust_dps, python_dps, diff, percent_diff * 100.0
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod twelve_f {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("twelveF", 0.0, 0.0, 0, -1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "twelveF s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("twelveF", 300.0, 0.0, 0, -1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "twelveF s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("twelveF", 0.0, 20.0, 0, -1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "twelveF s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("twelveF", 500.0, 30.0, 0, -1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "twelveF s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("twelveF", 1000.0, 50.0, 0, -1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "twelveF s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod aak {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aak", 0.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aak s3 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aak", 300.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aak s3 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aak", 0.0, 20.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aak s3 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aak", 500.0, 30.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aak s3 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aak", 1000.0, 50.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aak s3 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aak", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aak s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aak", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aak s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aak", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aak s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aak", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aak s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aak", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aak s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod absinthe {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Absinthe", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Absinthe s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Absinthe", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Absinthe s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Absinthe", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Absinthe s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Absinthe", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Absinthe s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Absinthe", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Absinthe s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Absinthe", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Absinthe s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Absinthe", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Absinthe s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Absinthe", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Absinthe s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Absinthe", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Absinthe s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Absinthe", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Absinthe s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod aciddrop {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aciddrop", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aciddrop s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aciddrop", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aciddrop s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aciddrop", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aciddrop s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aciddrop", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aciddrop s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aciddrop", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aciddrop s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aciddrop", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aciddrop s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aciddrop", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aciddrop s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aciddrop", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aciddrop s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aciddrop", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aciddrop s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aciddrop", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aciddrop s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod adnachiel {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Adnachiel", 0.0, 0.0, 0, -1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Adnachiel s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Adnachiel", 300.0, 0.0, 0, -1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Adnachiel s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Adnachiel", 0.0, 20.0, 0, -1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Adnachiel s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Adnachiel", 500.0, 30.0, 0, -1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Adnachiel s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Adnachiel", 1000.0, 50.0, 0, -1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Adnachiel s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod andreana {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Andreana", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Andreana s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Andreana", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Andreana s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Andreana", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Andreana s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Andreana", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Andreana s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Andreana", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Andreana s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Andreana", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Andreana s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Andreana", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Andreana s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Andreana", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Andreana s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Andreana", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Andreana s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Andreana", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Andreana s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod angelina {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Angelina", 0.0, 0.0, 1, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Angelina s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Angelina", 300.0, 0.0, 1, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Angelina s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Angelina", 0.0, 20.0, 1, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Angelina s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Angelina", 500.0, 30.0, 1, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Angelina s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Angelina", 1000.0, 50.0, 1, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Angelina s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Angelina", 0.0, 0.0, 2, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Angelina s3 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Angelina", 300.0, 0.0, 2, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Angelina s3 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Angelina", 0.0, 20.0, 2, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Angelina s3 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Angelina", 500.0, 30.0, 2, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Angelina s3 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Angelina", 1000.0, 50.0, 2, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Angelina s3 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Angelina", 0.0, 0.0, 0, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Angelina s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Angelina", 300.0, 0.0, 0, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Angelina s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Angelina", 0.0, 20.0, 0, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Angelina s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Angelina", 500.0, 30.0, 0, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Angelina s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Angelina", 1000.0, 50.0, 0, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Angelina s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod aosta {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aosta", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aosta s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aosta", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aosta s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aosta", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aosta s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aosta", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aosta s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aosta", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aosta s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aosta", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aosta s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aosta", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aosta s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aosta", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aosta s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aosta", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aosta s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Aosta", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Aosta s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod april {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("April", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "April s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("April", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "April s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("April", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "April s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("April", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "April s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("April", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "April s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("April", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "April s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("April", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "April s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("April", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "April s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("April", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "April s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("April", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "April s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod archetto {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Archetto", 0.0, 0.0, 0, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Archetto s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Archetto", 300.0, 0.0, 0, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Archetto s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Archetto", 0.0, 20.0, 0, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Archetto s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Archetto", 500.0, 30.0, 0, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Archetto s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Archetto", 1000.0, 50.0, 0, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Archetto s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Archetto", 0.0, 0.0, 2, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Archetto s3 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Archetto", 300.0, 0.0, 2, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Archetto s3 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Archetto", 0.0, 20.0, 2, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Archetto s3 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Archetto", 500.0, 30.0, 2, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Archetto s3 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Archetto", 1000.0, 50.0, 2, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Archetto s3 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Archetto", 0.0, 0.0, 1, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Archetto s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Archetto", 300.0, 0.0, 1, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Archetto s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Archetto", 0.0, 20.0, 1, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Archetto s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Archetto", 500.0, 30.0, 1, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Archetto s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Archetto", 1000.0, 50.0, 1, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Archetto s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod arene {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Arene", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Arene s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Arene", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Arene s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Arene", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Arene s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Arene", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Arene s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Arene", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Arene s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Arene", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Arene s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Arene", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Arene s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Arene", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Arene s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Arene", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Arene s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Arene", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Arene s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod asbestos {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Asbestos", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Asbestos s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Asbestos", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Asbestos s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Asbestos", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Asbestos s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Asbestos", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Asbestos s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Asbestos", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Asbestos s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Asbestos", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Asbestos s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Asbestos", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Asbestos s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Asbestos", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Asbestos s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Asbestos", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Asbestos s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Asbestos", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Asbestos s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod ascalon {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ascalon", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ascalon s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ascalon", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ascalon s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ascalon", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ascalon s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ascalon", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ascalon s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ascalon", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ascalon s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ascalon", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ascalon s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ascalon", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ascalon s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ascalon", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ascalon s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ascalon", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ascalon s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ascalon", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ascalon s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ascalon", 0.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ascalon s3 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ascalon", 300.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ascalon s3 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ascalon", 0.0, 20.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ascalon s3 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ascalon", 500.0, 30.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ascalon s3 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ascalon", 1000.0, 50.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ascalon s3 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod ash {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ash", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ash s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ash", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ash s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ash", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ash s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ash", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ash s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ash", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ash s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ash", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ash s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ash", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ash s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ash", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ash s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ash", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ash s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ash", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ash s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ash", 0.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ash s3 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ash", 300.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ash s3 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ash", 0.0, 20.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ash s3 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ash", 500.0, 30.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ash s3 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ash", 1000.0, 50.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ash s3 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod ashlock {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ashlock", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ashlock s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ashlock", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ashlock s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ashlock", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ashlock s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ashlock", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ashlock s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ashlock", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ashlock s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ashlock", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ashlock s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ashlock", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ashlock s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ashlock", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ashlock s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ashlock", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ashlock s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ashlock", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ashlock s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod bagpipe {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Bagpipe", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Bagpipe s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Bagpipe", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Bagpipe s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Bagpipe", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Bagpipe s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Bagpipe", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Bagpipe s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Bagpipe", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Bagpipe s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Bagpipe", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Bagpipe s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Bagpipe", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Bagpipe s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Bagpipe", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Bagpipe s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Bagpipe", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Bagpipe s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Bagpipe", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Bagpipe s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Bagpipe", 0.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Bagpipe s3 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Bagpipe", 300.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Bagpipe s3 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Bagpipe", 0.0, 20.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Bagpipe s3 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Bagpipe", 500.0, 30.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Bagpipe s3 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Bagpipe", 1000.0, 50.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Bagpipe s3 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod blaze {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Blaze", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Blaze s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Blaze", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Blaze s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Blaze", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Blaze s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Blaze", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Blaze s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Blaze", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Blaze s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Blaze", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Blaze s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Blaze", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Blaze s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Blaze", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Blaze s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Blaze", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Blaze s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Blaze", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Blaze s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod blue_poison {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("BluePoison", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "BluePoison s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("BluePoison", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "BluePoison s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("BluePoison", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "BluePoison s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("BluePoison", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "BluePoison s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("BluePoison", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "BluePoison s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("BluePoison", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "BluePoison s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("BluePoison", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "BluePoison s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("BluePoison", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "BluePoison s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("BluePoison", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "BluePoison s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("BluePoison", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "BluePoison s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod ceobe {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ceobe", 0.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ceobe s3 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ceobe", 300.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ceobe s3 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ceobe", 0.0, 20.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ceobe s3 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ceobe", 500.0, 30.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ceobe s3 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ceobe", 1000.0, 50.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ceobe s3 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ceobe", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ceobe s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ceobe", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ceobe s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ceobe", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ceobe s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ceobe", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ceobe s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ceobe", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ceobe s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ceobe", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ceobe s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ceobe", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ceobe s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ceobe", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ceobe s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ceobe", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ceobe s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ceobe", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ceobe s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod chen {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Chen", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Chen s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Chen", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Chen s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Chen", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Chen s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Chen", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Chen s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Chen", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Chen s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Chen", 0.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Chen s3 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Chen", 300.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Chen s3 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Chen", 0.0, 20.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Chen s3 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Chen", 500.0, 30.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Chen s3 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Chen", 1000.0, 50.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Chen s3 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Chen", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Chen s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Chen", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Chen s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Chen", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Chen s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Chen", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Chen s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Chen", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Chen s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod exusiai {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Exusiai", 0.0, 0.0, 0, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Exusiai s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Exusiai", 300.0, 0.0, 0, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Exusiai s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Exusiai", 0.0, 20.0, 0, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Exusiai s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Exusiai", 500.0, 30.0, 0, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Exusiai s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Exusiai", 1000.0, 50.0, 0, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Exusiai s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Exusiai", 0.0, 0.0, 2, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Exusiai s3 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Exusiai", 300.0, 0.0, 2, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Exusiai s3 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Exusiai", 0.0, 20.0, 2, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Exusiai s3 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Exusiai", 500.0, 30.0, 2, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Exusiai s3 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Exusiai", 1000.0, 50.0, 2, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Exusiai s3 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Exusiai", 0.0, 0.0, 1, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Exusiai s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Exusiai", 300.0, 0.0, 1, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Exusiai s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Exusiai", 0.0, 20.0, 1, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Exusiai s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Exusiai", 500.0, 30.0, 1, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Exusiai s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Exusiai", 1000.0, 50.0, 1, 2)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Exusiai s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod eyjafjalla {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Eyjafjalla", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Eyjafjalla s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Eyjafjalla", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Eyjafjalla s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Eyjafjalla", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Eyjafjalla s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Eyjafjalla", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Eyjafjalla s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Eyjafjalla", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Eyjafjalla s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Eyjafjalla", 0.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Eyjafjalla s3 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Eyjafjalla", 300.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Eyjafjalla s3 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Eyjafjalla", 0.0, 20.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Eyjafjalla s3 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Eyjafjalla", 500.0, 30.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Eyjafjalla s3 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Eyjafjalla", 1000.0, 50.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Eyjafjalla s3 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Eyjafjalla", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Eyjafjalla s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Eyjafjalla", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Eyjafjalla s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Eyjafjalla", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Eyjafjalla s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Eyjafjalla", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Eyjafjalla s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Eyjafjalla", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Eyjafjalla s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod ifrit {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ifrit", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ifrit s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ifrit", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ifrit s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ifrit", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ifrit s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ifrit", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ifrit s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ifrit", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ifrit s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ifrit", 0.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ifrit s3 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ifrit", 300.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ifrit s3 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ifrit", 0.0, 20.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ifrit s3 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ifrit", 500.0, 30.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ifrit s3 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ifrit", 1000.0, 50.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ifrit s3 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ifrit", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ifrit s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ifrit", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ifrit s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ifrit", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ifrit s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ifrit", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ifrit s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Ifrit", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Ifrit s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod mlynar {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mlynar", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mlynar s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mlynar", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mlynar s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mlynar", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mlynar s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mlynar", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mlynar s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mlynar", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mlynar s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mlynar", 0.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mlynar s3 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mlynar", 300.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mlynar s3 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mlynar", 0.0, 20.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mlynar s3 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mlynar", 500.0, 30.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mlynar s3 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mlynar", 1000.0, 50.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mlynar s3 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mlynar", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mlynar s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mlynar", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mlynar s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mlynar", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mlynar s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mlynar", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mlynar s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mlynar", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mlynar s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod mountain {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mountain", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mountain s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mountain", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mountain s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mountain", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mountain s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mountain", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mountain s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mountain", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mountain s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mountain", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mountain s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mountain", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mountain s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mountain", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mountain s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mountain", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mountain s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mountain", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mountain s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mountain", 0.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mountain s3 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mountain", 300.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mountain s3 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mountain", 0.0, 20.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mountain s3 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mountain", 500.0, 30.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mountain s3 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mountain", 1000.0, 50.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mountain s3 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod mudrock {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mudrock", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mudrock s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mudrock", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mudrock s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mudrock", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mudrock s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mudrock", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mudrock s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mudrock", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mudrock s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mudrock", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mudrock s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mudrock", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mudrock s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mudrock", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mudrock s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mudrock", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mudrock s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mudrock", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mudrock s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mudrock", 0.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mudrock s3 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mudrock", 300.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mudrock s3 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mudrock", 0.0, 20.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mudrock s3 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mudrock", 500.0, 30.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mudrock s3 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Mudrock", 1000.0, 50.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Mudrock s3 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod rosa {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Rosa", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Rosa s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Rosa", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Rosa s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Rosa", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Rosa s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Rosa", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Rosa s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Rosa", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Rosa s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Rosa", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Rosa s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Rosa", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Rosa s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Rosa", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Rosa s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Rosa", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Rosa s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Rosa", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Rosa s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Rosa", 0.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Rosa s3 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Rosa", 300.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Rosa s3 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Rosa", 0.0, 20.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Rosa s3 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Rosa", 500.0, 30.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Rosa s3 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Rosa", 1000.0, 50.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Rosa s3 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod schwarz {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Schwarz", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Schwarz s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Schwarz", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Schwarz s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Schwarz", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Schwarz s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Schwarz", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Schwarz s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Schwarz", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Schwarz s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Schwarz", 0.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Schwarz s3 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Schwarz", 300.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Schwarz s3 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Schwarz", 0.0, 20.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Schwarz s3 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Schwarz", 500.0, 30.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Schwarz s3 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Schwarz", 1000.0, 50.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Schwarz s3 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Schwarz", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Schwarz s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Schwarz", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Schwarz s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Schwarz", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Schwarz s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Schwarz", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Schwarz s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Schwarz", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Schwarz s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod silver_ash {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("SilverAsh", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "SilverAsh s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("SilverAsh", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "SilverAsh s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("SilverAsh", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "SilverAsh s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("SilverAsh", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "SilverAsh s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("SilverAsh", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "SilverAsh s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("SilverAsh", 0.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "SilverAsh s3 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("SilverAsh", 300.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "SilverAsh s3 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("SilverAsh", 0.0, 20.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "SilverAsh s3 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("SilverAsh", 500.0, 30.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "SilverAsh s3 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("SilverAsh", 1000.0, 50.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "SilverAsh s3 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("SilverAsh", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "SilverAsh s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("SilverAsh", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "SilverAsh s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("SilverAsh", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "SilverAsh s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("SilverAsh", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "SilverAsh s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("SilverAsh", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "SilverAsh s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod surtr {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Surtr", 0.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Surtr s3 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Surtr", 300.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Surtr s3 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Surtr", 0.0, 20.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Surtr s3 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Surtr", 500.0, 30.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Surtr s3 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Surtr", 1000.0, 50.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Surtr s3 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Surtr", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Surtr s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Surtr", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Surtr s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Surtr", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Surtr s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Surtr", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Surtr s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Surtr", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Surtr s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Surtr", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Surtr s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Surtr", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Surtr s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Surtr", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Surtr s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Surtr", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Surtr s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Surtr", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Surtr s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod thorns {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Thorns", 0.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Thorns s3 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Thorns", 300.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Thorns s3 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Thorns", 0.0, 20.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Thorns s3 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Thorns", 500.0, 30.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Thorns s3 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Thorns", 1000.0, 50.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Thorns s3 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Thorns", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Thorns s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Thorns", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Thorns s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Thorns", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Thorns s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Thorns", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Thorns s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Thorns", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Thorns s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Thorns", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Thorns s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Thorns", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Thorns s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Thorns", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Thorns s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Thorns", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Thorns s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("Thorns", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "Thorns s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

    mod w {
        use super::*;

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("W", 0.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "W s1 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("W", 300.0, 0.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "W s1 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("W", 0.0, 20.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "W s1 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("W", 500.0, 30.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "W s1 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s1_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("W", 1000.0, 50.0, 0, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "W s1 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("W", 0.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "W s2 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("W", 300.0, 0.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "W s2 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("W", 0.0, 20.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "W s2 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("W", 500.0, 30.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "W s2 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s2_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("W", 1000.0, 50.0, 1, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "W s2 def=1000 res=50")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("W", 0.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "W s3 def=0 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def300_res0() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("W", 300.0, 0.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "W s3 def=300 res=0")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def0_res20() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("W", 0.0, 20.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "W s3 def=0 res=20")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def500_res30() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("W", 500.0, 30.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "W s3 def=500 res=30")
                .expect("DPS comparison failed");
        }

        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn test_s3_def1000_res50() {
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("W", 1000.0, 50.0, 2, 1)
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "W s3 def=1000 res=50")
                .expect("DPS comparison failed");
        }

    }

}
