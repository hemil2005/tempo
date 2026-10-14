use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Job {
    pub id: u64,
    pub input: PathBuf,
    pub status: JobStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JobStatus {
    Queued,
    Running(f32),
    Done,
    Failed(String),
    Cancelled,
}

impl JobStatus {
    pub fn label(&self) -> String {
        match self {
            JobStatus::Queued => "Queued".to_string(),
            JobStatus::Running(p) => format!("{:.0}%", p * 100.0),
            JobStatus::Done => "Done".to_string(),
            JobStatus::Failed(e) => format!("Failed: {}", short_error(e)),
            JobStatus::Cancelled => "Cancelled".to_string(),
        }
    }

    pub fn progress(&self) -> f32 {
        match self {
            JobStatus::Running(p) => *p,
            JobStatus::Done => 1.0,
            _ => 0.0,
        }
    }

    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            JobStatus::Done | JobStatus::Failed(_) | JobStatus::Cancelled
        )
    }

    pub fn is_queued(&self) -> bool {
        matches!(self, JobStatus::Queued)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, JobStatus::Failed(_))
    }
}

fn short_error(e: &str) -> String {
    const MAX: usize = 60;
    if e.len() > MAX {
        format!("{}...", &e[..MAX])
    } else {
        e.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct JobTask {
    pub id: u64,
    pub input: PathBuf,
    pub output: PathBuf,
    pub settings: converter_core::EncodeSettings,
    pub target_size_mb: Option<f64>,
}

pub fn output_path_for(input: &Path, out_dir: &Path, ext: &str, overwrite: bool) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let mut candidate = out_dir.join(format!("{}.{}", stem, ext));
    if overwrite {
        return candidate;
    }
    let mut n = 1u32;
    while candidate.exists() {
        n += 1;
        candidate = out_dir.join(format!("{} ({}).{}", stem, n, ext));
    }
    candidate
}
