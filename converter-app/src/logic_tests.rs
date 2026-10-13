#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::app::{App, Message};
    use crate::job::JobStatus;
    use crate::worker::WorkerEvent;
    use converter_core::OutputFormat;

    fn test_wav() -> PathBuf {
        PathBuf::from(r"D:\rust-audio-converter\testdata\tone.wav")
    }

    #[test]
    fn add_and_remove_job() {
        let mut app = App::default();
        let _ = app.update(Message::FilesAdded(vec![test_wav()]));
        assert_eq!(app.job_count(), 1);
        let id = app.job_ids()[0];
        let _ = app.update(Message::RemoveJob(id));
        assert_eq!(app.job_count(), 0);
    }

    #[test]
    fn duplicate_files_ignored() {
        let mut app = App::default();
        let _ = app.update(Message::FilesAdded(vec![test_wav(), test_wav()]));
        assert_eq!(app.job_count(), 1);
    }

    #[test]
    fn format_selection_changes_settings() {
        let mut app = App::default();
        let _ = app.update(Message::FormatSelected(OutputFormat::Opus));
        assert_eq!(app.selected_format(), OutputFormat::Opus);
        let _ = app.update(Message::OpusBitrateSelected(96));
        let settings = app.encode_settings();
        assert_eq!(settings.format, OutputFormat::Opus);
    }

    #[test]
    fn worker_events_update_status() {
        let mut app = App::default();
        let _ = app.update(Message::FilesAdded(vec![test_wav()]));
        let id = app.job_ids()[0];
        app.apply_test_event(WorkerEvent::Progress(id, 0.5));
        assert!(matches!(app.job_status(id), Some(JobStatus::Running(_))));
        app.apply_test_event(WorkerEvent::Finished(id, Ok(())));
        assert!(matches!(app.job_status(id), Some(JobStatus::Done)));
    }

    #[test]
    fn worker_failure_recorded() {
        let mut app = App::default();
        let _ = app.update(Message::FilesAdded(vec![test_wav()]));
        let id = app.job_ids()[0];
        app.apply_test_event(WorkerEvent::Finished(id, Err("boom".to_string())));
        assert!(matches!(app.job_status(id), Some(JobStatus::Failed(_))));
    }

    #[test]
    fn end_to_end_gui_conversion() {
        let out = PathBuf::from(r"D:\rust-audio-converter\testdata\gui-out");
        let _ = std::fs::remove_dir_all(&out);
        let mut app = App::default();
        let _ = app.update(Message::FilesAdded(vec![test_wav()]));
        let _ = app.update(Message::FormatSelected(OutputFormat::Opus));
        let _ = app.update(Message::OutputDirChanged(out.to_string_lossy().to_string()));
        let _ = app.update(Message::Start);
        assert!(app.is_running());
        for _ in 0..600 {
            let _ = app.update(Message::PollWorker);
            if !app.is_running() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        assert!(!app.is_running());
        let id = app.job_ids()[0];
        assert!(
            matches!(app.job_status(id), Some(JobStatus::Done)),
            "status was {:?}",
            app.job_status(id)
        );
        assert!(out.join("tone.opus").exists());
        let _ = std::fs::remove_dir_all(&out);
    }
}
