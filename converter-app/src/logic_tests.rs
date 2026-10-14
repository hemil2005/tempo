#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::app::{App, Message};
    use crate::job::JobStatus;
    use crate::worker::WorkerEvent;
    use converter_core::OutputFormat;

    fn test_wav() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("repo root")
            .join("testdata")
            .join("tone.wav")
    }

    fn test_short_wav() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("repo root")
            .join("testdata")
            .join("short.wav")
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
        let out = std::env::temp_dir().join("tempo_gui_test_out");
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

    #[test]
    fn preset_selection_and_override() {
        let mut app = App::default();
        let _ = app.update(Message::PresetSelected(
            crate::settings::Preset::HighQuality,
        ));
        assert_eq!(app.settings().preset, crate::settings::Preset::HighQuality);
        assert_eq!(app.settings().mp3_bitrate, 320);
        assert_eq!(app.settings().opus_bitrate, 192);

        // Manually changing bitrate switches preset to Custom
        let _ = app.update(Message::OpusBitrateSelected(96));
        assert_eq!(app.settings().preset, crate::settings::Preset::Custom);
        assert_eq!(app.settings().opus_bitrate, 96);
    }

    #[test]
    fn recursive_folder_collection() {
        let temp_dir = std::env::temp_dir().join("tempo_test_recursive");
        let _ = std::fs::remove_dir_all(&temp_dir);
        let sub = temp_dir.join("subdir");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(temp_dir.join("root.wav"), b"").unwrap();
        std::fs::write(sub.join("nested.mp3"), b"").unwrap();
        std::fs::write(sub.join("ignore.txt"), b"").unwrap();

        let collected = crate::app::collect_audio_files_recursive(&temp_dir);
        assert_eq!(collected.len(), 2);
        assert!(collected.iter().any(|p| p.ends_with("root.wav")));
        assert!(collected.iter().any(|p| p.ends_with("nested.mp3")));
        assert!(!collected.iter().any(|p| p.ends_with("ignore.txt")));
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn reorder_jobs_up_and_down() {
        let mut app = App::default();
        let file1 = test_wav();
        let file2 = test_short_wav();
        let _ = app.update(Message::FilesAdded(vec![file1, file2]));
        assert_eq!(app.job_count(), 2);
        let ids = app.job_ids();
        let id1 = ids[0];
        let id2 = ids[1];

        // Move second job up
        let _ = app.update(Message::MoveJobUp(id2));
        assert_eq!(app.job_ids(), vec![id2, id1]);

        // Move first job (now id2) up (should be no-op since it's already top)
        let _ = app.update(Message::MoveJobUp(id2));
        assert_eq!(app.job_ids(), vec![id2, id1]);

        // Move id2 down
        let _ = app.update(Message::MoveJobDown(id2));
        assert_eq!(app.job_ids(), vec![id1, id2]);
    }

    #[test]
    fn retry_individual_and_all_failed_jobs() {
        let mut app = App::default();
        let file1 = test_wav();
        let file2 = test_short_wav();
        let _ = app.update(Message::FilesAdded(vec![file1, file2]));
        let ids = app.job_ids();
        let id1 = ids[0];
        let id2 = ids[1];

        app.apply_test_event(WorkerEvent::Finished(id1, Err("error 1".to_string())));
        app.apply_test_event(WorkerEvent::Finished(id2, Err("error 2".to_string())));
        assert!(matches!(app.job_status(id1), Some(JobStatus::Failed(_))));
        assert!(matches!(app.job_status(id2), Some(JobStatus::Failed(_))));

        // Retry individual job
        let _ = app.update(Message::RetryJob(id1));
        assert!(matches!(app.job_status(id1), Some(JobStatus::Queued)));
        assert!(matches!(app.job_status(id2), Some(JobStatus::Failed(_))));

        // Retry all remaining failed
        let _ = app.update(Message::RetryAllFailed);
        assert!(matches!(app.job_status(id1), Some(JobStatus::Queued)));
        assert!(matches!(app.job_status(id2), Some(JobStatus::Queued)));
    }

    #[test]
    fn pause_and_resume_state() {
        let mut app = App::default();
        let _ = app.update(Message::FilesAdded(vec![test_wav()]));
        assert!(!app.is_paused());
        // Pause when not running is a no-op
        let _ = app.update(Message::Pause);
        assert!(!app.is_paused());
    }

    #[test]
    fn target_presets_configuration() {
        let mut app = App::default();

        // Discord 10MB preset
        let _ = app.update(Message::PresetSelected(
            crate::settings::Preset::Discord10Mb,
        ));
        assert_eq!(app.settings().preset, crate::settings::Preset::Discord10Mb);
        assert_eq!(app.settings().format, OutputFormat::Opus);
        assert_eq!(app.settings().target_size_mb(), Some(9.5));

        // YouTube Music preset
        let _ = app.update(Message::PresetSelected(crate::settings::Preset::YouTube));
        assert_eq!(app.settings().preset, crate::settings::Preset::YouTube);
        assert_eq!(app.settings().format, OutputFormat::M4a);
        assert_eq!(app.settings().sample_rate, 48000);
        assert_eq!(app.settings().aac_bitrate, 256);

        // iPhone preset
        let _ = app.update(Message::PresetSelected(crate::settings::Preset::IPhone));
        assert_eq!(app.settings().preset, crate::settings::Preset::IPhone);
        assert_eq!(app.settings().format, OutputFormat::M4a);
        assert_eq!(app.settings().sample_rate, 44100);
        assert_eq!(app.settings().aac_bitrate, 256);
    }

    #[test]
    fn folder_watcher_lifecycle_and_auto_convert() {
        let mut app = App::default();
        assert!(!app.is_watching());
        assert!(!app.auto_convert_watched());

        let temp_dir = std::env::temp_dir().join("tempo_watch_test");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let _ = app.update(Message::WatchFolderChosen(Some(temp_dir.clone())));
        assert!(app.is_watching());

        let _ = app.update(Message::AutoConvertWatchedToggled(true));
        assert!(app.auto_convert_watched());

        let _ = app.update(Message::ClearWatchFolder);
        assert!(!app.is_watching());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
