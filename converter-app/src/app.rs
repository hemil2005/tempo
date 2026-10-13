use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{mpsc, Arc};

use iced::widget::{
    button, checkbox, column, container, pick_list, progress_bar, row, scrollable, svg, text,
    text_input, Space,
};
use iced::window;
use iced::{Element, Fill, Subscription, Task};

use converter_core::OutputFormat;

const LOGO_SVG: &str = r##"<svg width="32" height="32" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
<path d="M4 10V13" stroke="#38BDF8" stroke-width="2.2" stroke-linecap="round"/>
<path d="M16 10V13" stroke="#38BDF8" stroke-width="2.2" stroke-linecap="round"/>
<path d="M7 7L7 16" stroke="#38BDF8" stroke-width="2.2" stroke-linecap="round"/>
<path d="M13 7L13 16" stroke="#38BDF8" stroke-width="2.2" stroke-linecap="round"/>
<path d="M19 7L19 16" stroke="#38BDF8" stroke-width="2.2" stroke-linecap="round"/>
<path d="M10 4L10 19" stroke="#38BDF8" stroke-width="2.2" stroke-linecap="round"/>
</svg>"##;

use crate::job::{output_path_for, Job, JobStatus, JobTask};
use crate::settings::{
    Chans, GuiSettings, Kbps, Mp3Mode, Preset, Rate, VbrLevel, VorbisQ, AAC_RATES, CHAN_OPTS,
    MP3_RATES, OPUS_RATES, RATE_OPTS, VBR_LEVELS, VORBIS_QS,
};
use crate::worker::{spawn_workers, suggested_worker_count, WorkerEvent, WorkerPool};

const AUDIO_EXTS: [&str; 12] = [
    "opus", "ogg", "oga", "mp3", "wav", "wave", "flac", "m4a", "mp4", "aac", "aiff", "aif",
];

#[derive(Debug, Clone)]
pub enum Message {
    AddFiles,
    AddFolder,
    FilesAdded(Vec<PathBuf>),
    BrowseOutput,
    OutputChosen(Option<PathBuf>),
    OutputDirChanged(String),
    OverwriteToggled(bool),
    FormatSelected(OutputFormat),
    PresetSelected(Preset),
    Mp3ModeSelected(Mp3Mode),
    Mp3BitrateSelected(u32),
    Mp3VbrChanged(u8),
    OpusBitrateSelected(u32),
    AacBitrateSelected(u32),
    VorbisQualityChanged(u8),
    SampleRateSelected(u32),
    ChannelsSelected(u16),
    RemoveJob(u64),
    ClearFinished,
    Start,
    Stop,
    PollWorker,
    WindowEvent(window::Event),
}

pub struct App {
    jobs: Vec<Job>,
    settings: GuiSettings,
    log: Vec<String>,
    next_id: u64,
    running: bool,
    done_count: usize,
    failed_count: usize,
    worker_rx: Option<mpsc::Receiver<WorkerEvent>>,
    pool: Option<WorkerPool>,
    hover_drop: bool,
}

impl Default for App {
    fn default() -> Self {
        App {
            jobs: Vec::new(),
            settings: GuiSettings::default(),
            log: Vec::new(),
            next_id: 1,
            running: false,
            done_count: 0,
            failed_count: 0,
            worker_rx: None,
            pool: None,
            hover_drop: false,
        }
    }
}

impl App {
    pub fn title(&self) -> String {
        "Tempo".to_string()
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::events().map(|(_id, event)| Message::WindowEvent(event)),
            window::frames().map(|_| Message::PollWorker),
        ])
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AddFiles => {
                return Task::perform(pick_audio_files(), Message::FilesAdded);
            }
            Message::AddFolder => {
                return Task::perform(pick_audio_folder(), Message::FilesAdded);
            }
            Message::FilesAdded(paths) => {
                let mut added = 0;
                for path in paths {
                    if !is_audio_file(&path) {
                        self.push_log(format!("skipped (not audio): {}", path.display()));
                        continue;
                    }
                    if self.jobs.iter().any(|j| j.input == path) {
                        continue;
                    }
                    self.jobs.push(Job {
                        id: self.next_id,
                        input: path,
                        status: JobStatus::Queued,
                    });
                    self.next_id += 1;
                    added += 1;
                }
                if added > 0 {
                    self.push_log(format!("added {} file(s)", added));
                }
            }
            Message::BrowseOutput => {
                return Task::perform(pick_output_folder(), Message::OutputChosen);
            }
            Message::OutputChosen(path) => {
                if let Some(p) = path {
                    self.settings.output_dir = p.to_string_lossy().to_string();
                }
            }
            Message::OutputDirChanged(s) => {
                self.settings.output_dir = s;
            }
            Message::OverwriteToggled(v) => {
                self.settings.overwrite = v;
            }
            Message::FormatSelected(f) => {
                self.settings.format = f;
            }
            Message::PresetSelected(p) => {
                self.settings.apply_preset(p);
            }
            Message::Mp3ModeSelected(m) => {
                self.settings.mp3_mode = m;
                self.settings.preset = Preset::Custom;
            }
            Message::Mp3BitrateSelected(b) => {
                self.settings.mp3_bitrate = b;
                self.settings.preset = Preset::Custom;
            }
            Message::Mp3VbrChanged(v) => {
                self.settings.mp3_vbr = v.min(9);
                self.settings.preset = Preset::Custom;
            }
            Message::OpusBitrateSelected(b) => {
                self.settings.opus_bitrate = b;
                self.settings.preset = Preset::Custom;
            }
            Message::AacBitrateSelected(b) => {
                self.settings.aac_bitrate = b;
                self.settings.preset = Preset::Custom;
            }
            Message::VorbisQualityChanged(q) => {
                self.settings.vorbis_q = q.min(10);
                self.settings.preset = Preset::Custom;
            }
            Message::SampleRateSelected(r) => {
                self.settings.sample_rate = r;
            }
            Message::ChannelsSelected(c) => {
                self.settings.channels = c;
            }
            Message::RemoveJob(id) => {
                if !self.running {
                    self.jobs.retain(|j| j.id != id);
                }
            }
            Message::ClearFinished => {
                if !self.running {
                    self.jobs.retain(|j| !j.status.is_finished());
                }
            }
            Message::Start => {
                self.start_conversion();
            }
            Message::Stop => {
                if let Some(pool) = &self.pool {
                    pool.cancel.store(true, Ordering::SeqCst);
                    self.push_log("stopping after current file(s)...".to_string());
                }
            }
            Message::PollWorker => {
                self.drain_worker();
            }
            Message::WindowEvent(event) => match event {
                window::Event::FileHovered(_) => {
                    self.hover_drop = true;
                }
                window::Event::FileDropped(path) => {
                    self.hover_drop = false;
                    if path.is_dir() {
                        let files = collect_audio_files_recursive(&path);
                        let mut added = 0;
                        for p in files {
                            if !self.jobs.iter().any(|j| j.input == p) {
                                self.jobs.push(Job {
                                    id: self.next_id,
                                    input: p,
                                    status: JobStatus::Queued,
                                });
                                self.next_id += 1;
                                added += 1;
                            }
                        }
                        if added > 0 {
                            self.push_log(format!("added {} file(s) from folder", added));
                        }
                    } else if is_audio_file(&path) && !self.jobs.iter().any(|j| j.input == path) {
                        self.jobs.push(Job {
                            id: self.next_id,
                            input: path.clone(),
                            status: JobStatus::Queued,
                        });
                        self.next_id += 1;
                        self.push_log(format!("added {}", path.display()));
                    }
                }
                window::Event::FilesHoveredLeft => {
                    self.hover_drop = false;
                }
                _ => {}
            },
        }
        Task::none()
    }

    fn push_log(&mut self, line: String) {
        self.log.push(line);
        if self.log.len() > 300 {
            let excess = self.log.len() - 300;
            self.log.drain(..excess);
        }
    }

    fn drain_worker(&mut self) {
        let events: Vec<WorkerEvent> = match &self.worker_rx {
            Some(rx) => rx.try_iter().collect(),
            None => return,
        };
        for event in events {
            self.apply_worker_event(event);
        }
    }

    fn apply_worker_event(&mut self, event: WorkerEvent) {
        match event {
            WorkerEvent::Progress(id, p) => {
                if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
                    job.status = JobStatus::Running(p);
                }
            }
            WorkerEvent::Finished(id, result) => {
                let name = self
                    .jobs
                    .iter()
                    .find(|j| j.id == id)
                    .map(|j| {
                        j.input
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("?")
                            .to_string()
                    })
                    .unwrap_or_default();
                match result {
                    Ok(()) => {
                        self.done_count += 1;
                        if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
                            job.status = JobStatus::Done;
                        }
                        self.push_log(format!("done: {}", name));
                    }
                    Err(e) => {
                        if e == "cancelled" {
                            if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
                                job.status = JobStatus::Cancelled;
                            }
                        } else {
                            self.failed_count += 1;
                            if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
                                job.status = JobStatus::Failed(e.clone());
                            }
                            self.push_log(format!("FAILED {}: {}", name, e));
                        }
                    }
                }
                self.check_all_done();
            }
        }
    }

    fn check_all_done(&mut self) {
        if !self.running {
            return;
        }
        let pending = self
            .jobs
            .iter()
            .filter(|j| matches!(j.status, JobStatus::Queued | JobStatus::Running(_)))
            .count();
        if pending == 0 {
            self.running = false;
            self.pool = None;
            self.worker_rx = None;
            self.push_log(format!(
                "finished: {} done, {} failed",
                self.done_count, self.failed_count
            ));
        }
    }

    fn start_conversion(&mut self) {
        if self.running {
            return;
        }
        let queued: Vec<&Job> = self
            .jobs
            .iter()
            .filter(|j| matches!(j.status, JobStatus::Queued))
            .collect();
        if queued.is_empty() {
            self.push_log("nothing to convert (queue is empty)".to_string());
            return;
        }
        let settings = self.settings.to_encode_settings();
        let out_dir = PathBuf::from(&self.settings.output_dir);
        let overwrite = self.settings.overwrite;
        let ext = self.settings.format.extension();

        let mut tasks = Vec::new();
        for job in queued {
            let base_dir = if out_dir.as_os_str().is_empty() {
                job.input
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| PathBuf::from("."))
            } else {
                out_dir.clone()
            };
            if !base_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(&base_dir) {
                    self.push_log(format!("cannot create output folder: {}", e));
                    return;
                }
            }
            let output = output_path_for(&job.input, &base_dir, ext, overwrite);
            tasks.push(JobTask {
                id: job.id,
                input: job.input.clone(),
                output,
                settings,
            });
        }

        let (tx, rx) = mpsc::channel();
        let cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let n = suggested_worker_count();
        self.pool = Some(spawn_workers(tasks, tx, cancel, n));
        self.worker_rx = Some(rx);
        self.running = true;
        self.done_count = 0;
        self.failed_count = 0;
        self.push_log(format!(
            "converting {} file(s) to {} ({})...",
            self.jobs
                .iter()
                .filter(|j| matches!(j.status, JobStatus::Queued))
                .count(),
            self.settings.format.label(),
            self.settings.quality_summary()
        ));
    }

    pub fn view(&self) -> Element<'_, Message> {
        let brand_icon = svg(svg::Handle::from_memory(LOGO_SVG.as_bytes()))
            .width(32)
            .height(32);

        let brand_title = column![
            text("Tempo").size(24),
            text("Free & Open Source Audio Converter").size(12),
        ]
        .spacing(2);

        let header = row![
            brand_icon,
            brand_title,
            Space::new().width(Fill),
            text(format!(
                "Queue: {} | Converted: {} | Failed: {}",
                self.jobs.len(),
                self.done_count,
                self.failed_count
            ))
            .size(13),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);

        let toolbar = row![
            button("+ Add Files").on_press(Message::AddFiles),
            button("📁 Add Folder").on_press(Message::AddFolder),
            button("✕ Clear Finished")
                .on_press(Message::ClearFinished)
                .style(button::secondary),
            Space::new().width(Fill),
            if self.running {
                button("⏹ Stop Conversion")
                    .on_press(Message::Stop)
                    .style(button::danger)
            } else {
                button("▶ Start Conversion")
                    .on_press(Message::Start)
                    .style(button::primary)
            },
        ]
        .spacing(10);

        let settings = self.settings_view();

        let queue = self.queue_view();

        let total = self.total_progress();
        let total_row = row![
            text(format!(
                "Total Progress ({} done, {} failed):",
                self.done_count, self.failed_count
            ))
            .size(13),
            progress_bar(0.0..=1.0, total).length(Fill),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);

        let log_text = self
            .log
            .iter()
            .rev()
            .take(6)
            .rev()
            .fold(String::new(), |mut acc, line| {
                acc.push_str(line);
                acc.push('\n');
                acc
            });
        let log_view = container(text(log_text).size(12))
            .padding(8)
            .style(container::bordered_box);

        let drop_hint = if self.hover_drop {
            text("Release to drop files and folders into Tempo").size(13)
        } else {
            text("Tip: drag & drop audio files or whole music folders anywhere on this window")
                .size(12)
        };

        container(
            column![header, toolbar, settings, queue, total_row, log_view, drop_hint,]
                .spacing(10)
                .padding(16),
        )
        .into()
    }

    fn settings_view(&self) -> Element<'_, Message> {
        let s = &self.settings;
        let format_pick = pick_list(
            &OutputFormat::ALL[..],
            Some(s.format),
            Message::FormatSelected,
        )
        .placeholder("Format")
        .width(130);

        let preset_pick = pick_list(&Preset::ALL[..], Some(s.preset), Message::PresetSelected)
            .placeholder("Preset")
            .width(170);

        let mut row1: Vec<Element<'_, Message>> = vec![
            text("Format:").into(),
            format_pick.into(),
            text("Preset:").into(),
            preset_pick.into(),
        ];

        match s.format {
            OutputFormat::Mp3 => {
                let mode_pick = pick_list(
                    &Mp3Mode::ALL[..],
                    Some(s.mp3_mode),
                    Message::Mp3ModeSelected,
                )
                .width(90);
                row1.push(text("Mode:").into());
                row1.push(mode_pick.into());
                match s.mp3_mode {
                    Mp3Mode::Cbr => {
                        let br = pick_list(&MP3_RATES[..], Some(Kbps(s.mp3_bitrate)), |k: Kbps| {
                            Message::Mp3BitrateSelected(k.0)
                        })
                        .width(110);
                        row1.push(text("Bitrate:").into());
                        row1.push(br.into());
                    }
                    Mp3Mode::Vbr => {
                        let vbr =
                            pick_list(&VBR_LEVELS[..], Some(VbrLevel(s.mp3_vbr)), |v: VbrLevel| {
                                Message::Mp3VbrChanged(v.0)
                            })
                            .width(70);
                        row1.push(text("Quality:").into());
                        row1.push(vbr.into());
                    }
                }
            }
            OutputFormat::Opus => {
                let br = pick_list(&OPUS_RATES[..], Some(Kbps(s.opus_bitrate)), |k: Kbps| {
                    Message::OpusBitrateSelected(k.0)
                })
                .width(110);
                row1.push(text("Bitrate:").into());
                row1.push(br.into());
            }
            OutputFormat::M4a => {
                let br = pick_list(&AAC_RATES[..], Some(Kbps(s.aac_bitrate)), |k: Kbps| {
                    Message::AacBitrateSelected(k.0)
                })
                .width(110);
                row1.push(text("Bitrate:").into());
                row1.push(br.into());
            }
            OutputFormat::OggVorbis => {
                let q = pick_list(&VORBIS_QS[..], Some(VorbisQ(s.vorbis_q)), |v: VorbisQ| {
                    Message::VorbisQualityChanged(v.0)
                })
                .width(70);
                row1.push(text("Quality:").into());
                row1.push(q.into());
            }
            _ => {}
        }

        let sr = pick_list(&RATE_OPTS[..], Some(Rate(s.sample_rate)), |r: Rate| {
            Message::SampleRateSelected(r.0)
        })
        .width(140);

        let ch = pick_list(&CHAN_OPTS[..], Some(Chans(s.channels)), |c: Chans| {
            Message::ChannelsSelected(c.0)
        })
        .width(140);

        let row2 = row![text("Rate:"), sr, text("Channels:"), ch].spacing(8);

        let out_input = text_input("Output folder (empty = same as input)", &s.output_dir)
            .on_input(Message::OutputDirChanged)
            .width(Fill);
        let row3 = row![
            out_input,
            button("Browse...").on_press(Message::BrowseOutput),
            checkbox(self.settings.overwrite)
                .label("Overwrite")
                .on_toggle(Message::OverwriteToggled),
        ]
        .spacing(8);

        container(
            column![
                text("Conversion Settings").size(14),
                row(row1).spacing(8).align_y(iced::Alignment::Center),
                row2.align_y(iced::Alignment::Center),
                row3.align_y(iced::Alignment::Center),
            ]
            .spacing(10),
        )
        .padding(12)
        .style(container::bordered_box)
        .into()
    }

    fn queue_view(&self) -> Element<'_, Message> {
        if self.jobs.is_empty() {
            return container(
                column![
                    text("No audio files in queue").size(16),
                    text("Drag and drop audio tracks or folders anywhere, or click \"+ Add Files\" / \"📁 Add Folder\"")
                        .size(13),
                    text(format!("Supported input formats: {}", AUDIO_EXTS.join(", "))).size(11),
                ]
                .spacing(6)
                .align_x(iced::Alignment::Center),
            )
            .width(Fill)
            .height(240)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .style(container::bordered_box)
            .into();
        }

        let header_row = row![
            text("Source File").width(320).size(12),
            text("Format").width(110).size(12),
            text("Progress").width(120).size(12),
            text("Status").width(150).size(12),
            text("").width(30),
        ]
        .spacing(8);

        let mut list = column![header_row].spacing(6);
        for job in &self.jobs {
            let name = job
                .input
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();
            let in_ext = job
                .input
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("?")
                .to_uppercase();
            let out_ext = self.settings.format.extension().to_uppercase();
            let status = job.status.label();
            let bar = progress_bar(0.0..=1.0, job.status.progress()).length(120);
            let remove = if matches!(job.status, JobStatus::Queued) && !self.running {
                button("✕")
                    .on_press(Message::RemoveJob(job.id))
                    .style(button::secondary)
            } else {
                button("✕").style(button::secondary)
            };
            list = list.push(
                row![
                    text(name).width(320).size(13),
                    text(format!("{} → {}", in_ext, out_ext))
                        .width(110)
                        .size(12),
                    bar,
                    text(status).width(150).size(12),
                    remove,
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            );
        }
        container(scrollable(list).height(240))
            .padding(10)
            .style(container::bordered_box)
            .into()
    }

    fn total_progress(&self) -> f32 {
        if self.jobs.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.jobs.iter().map(|j| j.status.progress()).sum();
        sum / self.jobs.len() as f32
    }
}

#[cfg(test)]
impl App {
    pub fn job_count(&self) -> usize {
        self.jobs.len()
    }

    pub fn job_ids(&self) -> Vec<u64> {
        self.jobs.iter().map(|j| j.id).collect()
    }

    pub fn job_status(&self, id: u64) -> Option<JobStatus> {
        self.jobs
            .iter()
            .find(|j| j.id == id)
            .map(|j| j.status.clone())
    }

    pub fn selected_format(&self) -> OutputFormat {
        self.settings.format
    }

    pub fn encode_settings(&self) -> converter_core::EncodeSettings {
        self.settings.to_encode_settings()
    }

    pub fn settings(&self) -> &GuiSettings {
        &self.settings
    }

    pub fn apply_test_event(&mut self, event: WorkerEvent) {
        self.apply_worker_event(event);
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}

fn is_audio_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
    {
        Some(ext) => AUDIO_EXTS.contains(&ext.as_str()),
        None => false,
    }
}

async fn pick_audio_files() -> Vec<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title("Add audio files")
        .add_filter("Audio", &AUDIO_EXTS)
        .pick_files()
        .await
        .map(|handles| {
            handles
                .into_iter()
                .map(|h| h.path().to_path_buf())
                .collect()
        })
        .unwrap_or_default()
}

async fn pick_output_folder() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title("Choose output folder")
        .pick_folder()
        .await
        .map(|h| h.path().to_path_buf())
}

async fn pick_audio_folder() -> Vec<PathBuf> {
    if let Some(folder) = rfd::AsyncFileDialog::new()
        .set_title("Choose folder to add")
        .pick_folder()
        .await
    {
        collect_audio_files_recursive(folder.path())
    } else {
        Vec::new()
    }
}

pub fn collect_audio_files_recursive(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut dirs_to_visit = vec![dir.to_path_buf()];
    while let Some(current_dir) = dirs_to_visit.pop() {
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    dirs_to_visit.push(p);
                } else if is_audio_file(&p) {
                    files.push(p);
                }
            }
        }
    }
    files.sort();
    files
}
