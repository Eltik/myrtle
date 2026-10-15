use std::{
    io,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use super::manifest::ResourceManifest;
use super::text_asset::export_text_asset;
use crate::unity::bundle::BundleFile;
use crate::unity::object_reader::read_object;
use crate::unity::serialized_file::SerializedFile;
use walkdir::WalkDir;

pub fn export_gamedata(
    bundle_dir: &Path,
    idx_path: &Path,
    output_dir: &Path,
) -> Result<(usize, usize), io::Error> {
    let manifest = ResourceManifest::load(idx_path)?;
    let mut exported = 0;
    // Counted, not just printed. The caller has to be able to fail the run: a
    // partial gamedata export is followed by an orphan sweep that deletes every
    // table this run did not write, so "some tables failed" cannot be a log line.
    let mut failed = 0;
    // Tables skipped for verifying under no schema, `(file stem, previous output kept)`,
    // and whether the walk reached the excel tables at all. See `write_skipped`.
    let mut skipped: Vec<(String, bool)> = Vec::new();
    let mut visited_excel = false;

    for entry in WalkDir::new(bundle_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        // Only process .bin files
        if path.extension().is_none_or(|ext| ext != "bin") {
            continue;
        }

        // The manifest names a bundle by its path relative to the server's
        // asset root, e.g. `anon/0dfb59a7275154d610241fcaef7aef4d.bin`, which
        // is exactly what the walk yields once the input root is stripped.
        let bundle_key = path
            .strip_prefix(bundle_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned();

        let data = std::fs::read(path)?;
        let bundle = match BundleFile::parse(data) {
            Ok(b) => b,
            Err(_) => continue,
        };

        if bundle.files.is_empty() {
            continue;
        }

        let sf = match SerializedFile::parse(bundle.files[0].data.clone()) {
            Ok(sf) => sf,
            Err(_) => continue,
        };

        for obj in &sf.objects {
            if obj.class_id != 49 {
                continue;
            } // TextAsset only

            let val = match read_object(&sf, obj) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let name = match val["m_Name"].as_str() {
                Some(n) => n,
                None => continue,
            };

            // Look up the real path from the manifest, KEYED BY BUNDLE. An
            // asset name is not unique: every story ships twice under one
            // name, the script in one bundle and its one-line summary in
            // another, so a name-only lookup handed both objects the same
            // destination and the summary landed on top of the script. Fall
            // back to the name-only map only when the bundle is unknown to the
            // manifest, so a bundle the walk found under a different root
            // still exports what it did before.
            let real_paths: Vec<&str> = match manifest.get_output_paths(&bundle_key, name) {
                Some(paths) => paths.iter().map(std::string::String::as_str).collect(),
                None => match manifest.get_output_path(name) {
                    Some(p) => vec![p],
                    None => continue,
                },
            };

            for real_path in real_paths {
                // Only extract gamedata files
                if !real_path.starts_with("gamedata/") {
                    continue;
                }

                visited_excel |= real_path.starts_with("gamedata/excel/");

                // Build output directory from the real path
                let out_path = output_dir.join(real_path);
                let out_parent = out_path.parent().unwrap_or(output_dir);
                std::fs::create_dir_all(out_parent)?;

                // Export using text_asset (handles AES decrypt + JSON detection)
                let file_stem = Path::new(real_path)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or(name);

                // A FlatBuffer that verifies under no schema is SKIPPED, not
                // decoded — see `decode_flatbuffer` for why an unchecked decode of
                // a mismatched schema can run forever. The check has to happen
                // HERE rather than inside `export_text_asset`, because that
                // function's last resort is to dump the raw payload as `.txt` or
                // `.bytes`, which would replace the previous extraction's good
                // `.json` with rubbish. Writing nothing leaves that file in place,
                // which is the graceful degrade the VPS watcher wants: a stale
                // table beats a missing one, and both beat a garbage one.
                if let Some(fb) = crate::export::text_asset::flatbuffer_payload(&val)
                    && let Some(table) = crate::flatbuffers_decode::unverified_table(&fb, file_stem)
                {
                    // Keeping the file is not enough: `run.mjs` sweeps every file in a
                    // written subtree whose mtime predates the extract, so an untouched
                    // stale table is deleted as an orphan. That is how CN 2.7.81 lost
                    // activity_table, building_data and roguelike_topic_table on the VPS
                    // (2026-10-09) and the release planner with them.
                    let kept = out_parent.join(format!("{file_stem}.json"));
                    let touched = std::fs::OpenOptions::new()
                        .write(true)
                        .open(&kept)
                        .and_then(|f| f.set_modified(std::time::SystemTime::now()))
                        .is_ok();
                    eprintln!(
                        "schema: {table} verifies under no schema, skipped ({})",
                        if touched {
                            "previous output kept"
                        } else {
                            "no previous output"
                        }
                    );
                    if !skipped.iter().any(|(t, _)| t == file_stem) {
                        skipped.push((file_stem.to_owned(), touched));
                    }
                    continue;
                }

                match export_text_asset(&val, out_parent, Some(file_stem)) {
                    Ok(()) => exported += 1,
                    Err(e) => {
                        eprintln!("  error exporting {name}: {e}");
                        failed += 1;
                    }
                }
            }
        }
    }

    if visited_excel {
        write_skipped(&output_dir.join("gamedata"), bundle_dir, &skipped);
    }

    Ok((exported, failed))
}

/// `gamedata/.skipped.json`: the tables the last gamedata export skipped, so the
/// backend can report a stale table instead of serving it silently. On 2026-10-14
/// EN `activity_table` stopped verifying, the previous `.json` was kept, and the
/// release planner missed a new event for days with nothing in `/health` to say so.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkippedFile {
    pub tables: Vec<SkippedTable>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkippedTable {
    pub table: String,
    /// RFC 3339 UTC of the first extract that skipped this table, carried over for
    /// as long as every extract since has skipped it too.
    pub since: String,
    /// The resource version of the bundles that failed to verify, from
    /// `<input>/.downloaded_version` (written by the downloader after a complete
    /// download). Absent when the input was not fetched by our downloader.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub res_version: Option<String>,
    /// Whether a previous `.json` was kept (and is being served stale) rather than
    /// the table being missing outright.
    pub previous_output: bool,
}

/// This run's skips, with each table's `since` carried over from `previous` when
/// that table was already listed. A table absent from `previous` starts at `now`;
/// a table that verifies again drops out, so a later re-break starts a new `since`.
fn merge_skipped(
    previous: &SkippedFile,
    skipped: &[(String, bool)],
    now: &str,
    res_version: Option<&str>,
) -> SkippedFile {
    let tables = skipped
        .iter()
        .map(|(table, previous_output)| SkippedTable {
            table: table.clone(),
            since: previous
                .tables
                .iter()
                .find(|t| &t.table == table)
                .map_or_else(|| now.to_owned(), |t| t.since.clone()),
            res_version: res_version.map(str::to_owned),
            previous_output: *previous_output,
        })
        .collect();
    SkippedFile { tables }
}

/// Rewrite `gamedata/.skipped.json`, ALWAYS, empty list included. `run.mjs` sweeps
/// every file in the written subtree whose mtime predates the extract, so a file
/// left untouched on a clean run would be deleted, and one left stale would keep
/// reporting a table that has since recovered.
///
/// Only called when the walk reached `gamedata/excel/`: an export that never saw
/// the excel tables (a partial download with no gamedata bundles) knows nothing
/// about them, and writing an empty list there would erase a real stale signal.
/// A write failure is a warning, not a failed export: the tables themselves were
/// written, and a missing file reads as "nothing stale" downstream, which the
/// orphan sweep would produce anyway.
fn write_skipped(gamedata_dir: &Path, input_dir: &Path, skipped: &[(String, bool)]) {
    let path = gamedata_dir.join(".skipped.json");
    let previous: SkippedFile = std::fs::read(&path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    let res_version = std::fs::read_to_string(input_dir.join(".downloaded_version"))
        .ok()
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty());
    let merged = merge_skipped(
        &previous,
        skipped,
        &rfc3339_utc(SystemTime::now()),
        res_version.as_deref(),
    );

    let tmp = gamedata_dir.join(".skipped.json.tmp");
    let result = serde_json::to_vec_pretty(&merged)
        .map_err(io::Error::other)
        .and_then(|json| std::fs::write(&tmp, json))
        .and_then(|()| std::fs::rename(&tmp, &path));
    if let Err(e) = result {
        eprintln!("warning: could not write {}: {e}", path.display());
    }
}

/// `2026-10-14T05:00:00Z`. Whole seconds, UTC; no date crate in this binary.
fn rfc3339_utc(t: SystemTime) -> String {
    let secs = t.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let (days, rem) = (secs / 86_400, secs % 86_400);
    // Days since 1970-01-01 -> civil date (Howard Hinnant's `civil_from_days`).
    let z = days.cast_signed() + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3_600,
        rem % 3_600 / 60,
        rem % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(table: &str, since: &str) -> SkippedTable {
        SkippedTable {
            table: table.into(),
            since: since.into(),
            res_version: Some("old".into()),
            previous_output: true,
        }
    }

    #[test]
    fn since_carries_over_for_a_table_still_skipped() {
        let previous = SkippedFile {
            tables: vec![
                entry("activity_table", "2026-10-14T05:00:00Z"),
                entry("building_data", "2026-10-09T00:00:00Z"),
            ],
        };
        let skipped = vec![
            ("activity_table".to_owned(), true),
            ("gacha_table".to_owned(), false),
        ];
        let merged = merge_skipped(&previous, &skipped, "2026-10-15T00:00:00Z", Some("new"));
        assert_eq!(
            merged.tables,
            vec![
                SkippedTable {
                    table: "activity_table".into(),
                    since: "2026-10-14T05:00:00Z".into(),
                    res_version: Some("new".into()),
                    previous_output: true,
                },
                SkippedTable {
                    table: "gacha_table".into(),
                    since: "2026-10-15T00:00:00Z".into(),
                    res_version: Some("new".into()),
                    previous_output: false,
                },
            ],
            "activity_table keeps its first-skip time, gacha_table starts now, \
             building_data verified again and drops out"
        );
    }

    #[test]
    fn a_clean_run_writes_an_empty_list() {
        let previous = SkippedFile {
            tables: vec![entry("activity_table", "2026-10-14T05:00:00Z")],
        };
        let merged = merge_skipped(&previous, &[], "2026-10-15T00:00:00Z", None);
        assert_eq!(serde_json::to_string(&merged).unwrap(), r#"{"tables":[]}"#);
    }

    #[test]
    fn json_shape_is_camel_case_and_omits_a_missing_version() {
        let file = SkippedFile {
            tables: vec![SkippedTable {
                table: "activity_table".into(),
                since: "2026-10-14T05:00:00Z".into(),
                res_version: None,
                previous_output: true,
            }],
        };
        assert_eq!(
            serde_json::to_string(&file).unwrap(),
            r#"{"tables":[{"table":"activity_table","since":"2026-10-14T05:00:00Z","previousOutput":true}]}"#
        );
    }

    #[test]
    fn write_skipped_round_trips_through_the_file() {
        let root = std::env::temp_dir().join(format!("unpacker_skipped_{}", std::process::id()));
        let (input, gamedata) = (root.join("in"), root.join("out/gamedata"));
        std::fs::create_dir_all(&input).unwrap();
        std::fs::create_dir_all(&gamedata).unwrap();
        std::fs::write(
            input.join(".downloaded_version"),
            "26-10-03-16-06-39_0a6acf\n",
        )
        .unwrap();
        let read = || -> SkippedFile {
            serde_json::from_slice(&std::fs::read(gamedata.join(".skipped.json")).unwrap()).unwrap()
        };

        let first = previous_with(&gamedata, "activity_table", "2026-10-14T05:00:00Z");
        assert_eq!(first.tables.len(), 1);
        write_skipped(&gamedata, &input, &[("activity_table".to_owned(), true)]);
        let after = read();
        assert_eq!(after.tables[0].since, "2026-10-14T05:00:00Z");
        assert_eq!(
            after.tables[0].res_version.as_deref(),
            Some("26-10-03-16-06-39_0a6acf")
        );
        assert!(!gamedata.join(".skipped.json.tmp").exists());

        write_skipped(&gamedata, &input, &[]);
        assert!(read().tables.is_empty());
        std::fs::remove_dir_all(&root).unwrap();
    }

    /// Seeds `.skipped.json` as an earlier extract would have left it.
    fn previous_with(gamedata: &Path, table: &str, since: &str) -> SkippedFile {
        let file = SkippedFile {
            tables: vec![entry(table, since)],
        };
        std::fs::write(
            gamedata.join(".skipped.json"),
            serde_json::to_vec(&file).unwrap(),
        )
        .unwrap();
        file
    }

    #[test]
    fn rfc3339_formats_utc_seconds() {
        assert_eq!(rfc3339_utc(UNIX_EPOCH), "1970-01-01T00:00:00Z");
        // 2026-10-14T05:00:00Z and a leap day.
        let at = |s: u64| UNIX_EPOCH + std::time::Duration::from_secs(s);
        assert_eq!(rfc3339_utc(at(1_791_954_000)), "2026-10-14T05:00:00Z");
        assert_eq!(rfc3339_utc(at(1_709_208_000)), "2024-02-29T12:00:00Z");
    }
}
