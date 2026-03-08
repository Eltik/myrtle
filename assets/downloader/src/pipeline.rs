use crate::download::Downloader;
use crate::extract;
use crate::manifest::Manifest;
use crate::progress::ProgressTracker;
use crate::types::{DownloadTask, PipelineStats};
use std::path::PathBuf;
use std::sync::Arc;

pub async fn run(
    downloader: Arc<Downloader>,
    tasks: Vec<DownloadTask>,
    savedir: PathBuf,
    manifest: &mut Manifest,
) -> anyhow::Result<PipelineStats> {
    let mut stats = PipelineStats::default();
    let progress = ProgressTracker::new(tasks.len() as u64);

    let mut handles = Vec::new();
    for task in tasks {
        let dl = Arc::clone(&downloader); // Downloader needs Clone (Arc internals)
        let dir = savedir.clone();
        let handle = tokio::spawn(async move {
            let result = dl.download(&task).await;
            (task, dir, result)
        });
        handles.push(handle);
    }

    for handle in handles {
        let (task, dir, result) = handle.await?;
        match result {
            Ok(data) => {
                let savedir = dir.clone();
                let data_clone = data;
                tokio::task::spawn_blocking(move || extract::extract_zip(&data_clone, &savedir))
                    .await??;

                manifest.update(&task.filename, &task.md5);
                manifest.save()?;
                stats.downloaded += 1;
                stats.total_bytes += task.total_size;
                progress.println(&format!("  {} ({})", task.filename, task.total_size));
            }
            Err(e) => {
                progress.println(&format!("  FAILED: {} - {}", task.filename, e));
                stats.failed += 1;
            }
        }
        progress.inc();
    }

    progress.finish();
    Ok(stats)
}
