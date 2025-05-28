use std::{
    fmt,
    io::Cursor,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::layout::get_next_id;
use anyhow::Result;
use eframe::egui;
use reqwest;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

pub struct AudioPlayer {
    pub id: String,
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Arc<Mutex<Option<Sink>>>,
    duration: Arc<Mutex<Duration>>,
    progress: Arc<Mutex<Duration>>,
    is_playing: Arc<Mutex<bool>>,
    audio_data: Arc<Vec<u8>>,
    last_play_instant: Arc<Mutex<Option<Instant>>>,
    volume: Arc<Mutex<f32>>,
    should_loop: bool,
    show_controls: bool,
}

impl AudioPlayer {
    pub fn new(
        url: String,
        autoplay: bool,
        should_loop: bool,
        show_controls: bool,
    ) -> Result<Self> {
        let id = get_next_id().to_string();

        let response = reqwest::blocking::get(&url)?;
        let bytes = response.bytes()?.to_vec();
        let audio_data_arc = Arc::new(bytes);

        let (stream, stream_handle) = OutputStream::try_default()?;

        let initial_sink: Option<Sink>;
        let initial_is_playing: bool;
        let initial_last_play_instant: Option<Instant>;
        let initial_duration: Duration;

        if autoplay {
            // Create the sink and start playing
            let sink = Sink::try_new(&stream_handle)?;
            let audio_cursor = Cursor::new(audio_data_arc.as_ref().clone());
            let decoder = Decoder::new(audio_cursor)?;
            let total = decoder.total_duration().unwrap_or(Duration::ZERO);
            initial_duration = total;
            sink.append(decoder);
            initial_sink = Some(sink);
            initial_is_playing = true;
            initial_last_play_instant = Some(Instant::now());
        } else {
            // Initialize without playing
            initial_sink = None;
            initial_is_playing = false;
            initial_last_play_instant = None;
            initial_duration = Duration::ZERO;
        }

        let sink = Arc::new(Mutex::new(initial_sink));
        let duration = Arc::new(Mutex::new(initial_duration)); // Duration should ideally be derived from the audio source
        let progress = Arc::new(Mutex::new(Duration::ZERO));
        let is_playing = Arc::new(Mutex::new(initial_is_playing));
        let last_play_instant = Arc::new(Mutex::new(initial_last_play_instant));
        let volume = Arc::new(Mutex::new(1.0));

        Ok(Self {
            id,
            _stream: stream,
            stream_handle,
            sink,
            duration,
            progress,
            is_playing,
            audio_data: audio_data_arc,
            last_play_instant,
            volume,
            should_loop,
            show_controls,
        })
    }

    fn create_sink(&self, seek_to: Duration) -> Result<Sink> {
        let cursor = Cursor::new(self.audio_data.as_ref().clone());
        let decoder = Decoder::new(cursor)?.skip_duration(seek_to);

        let sink = Sink::try_new(&self.stream_handle)?;
        sink.pause();
        let total = decoder.total_duration().unwrap_or(Duration::ZERO);
        *self.duration.lock().unwrap() = total;
        sink.append(decoder);
        sink.set_volume(*self.volume.lock().unwrap());
        Ok(sink)
    }

    pub fn toggle_playback(&self) {
        let mut is_playing = self.is_playing.lock().unwrap();
        let mut sink_guard = self.sink.lock().unwrap();
        let mut last_play = self.last_play_instant.lock().unwrap();

        if *is_playing {
            if let Some(sink) = sink_guard.as_ref() {
                sink.pause();
                if let Some(start) = *last_play {
                    let elapsed = start.elapsed();
                    let mut progress = self.progress.lock().unwrap();
                    *progress += elapsed;
                }
            }
            *last_play = None;
            *is_playing = false;
        } else {
            let seek_to = *self.progress.lock().unwrap();
            if sink_guard.is_none() || sink_guard.as_ref().unwrap().empty() {
                if let Ok(new_sink) = self.create_sink(seek_to) {
                    new_sink.play();
                    *sink_guard = Some(new_sink);
                }
            } else if let Some(sink) = sink_guard.as_ref() {
                sink.play();
            }
            *last_play = Some(Instant::now());
            *is_playing = true;
        }
    }

    pub fn update_progress(&self, ctx: &egui::Context) {
        let mut is_playing = self.is_playing.lock().unwrap();
        let mut progress = self.progress.lock().unwrap();
        let sink_lock = self.sink.lock().unwrap();
        let mut last_play = self.last_play_instant.lock().unwrap(); // Lock last_play_instant here

        if *is_playing {
            // Calculate elapsed time since last update
            let now = Instant::now();
            if let Some(last_play_time) = last_play.as_mut() {
                let elapsed = last_play_time.elapsed();
                *progress += elapsed;
                *last_play_time = now; // Update last_play_instant to now
            } else {
                // If last_play_instant is None but is_playing is true,
                // this might indicate playback just started outside toggle_playback.
                // Set last_play_instant to now.
                *last_play = Some(now);
            }

            // If playback is over, stop
            if let Some(sink) = sink_lock.as_ref() {
                if sink.empty() {
                    if self.should_loop {
                        drop(sink_lock);
                        if let Ok(new_sink) = self.create_sink(Duration::ZERO) {
                            new_sink.play();
                            *last_play = Some(Instant::now());
                            *self.sink.lock().unwrap() = Some(new_sink);
                            *progress = Duration::ZERO;
                        }
                    } else {
                        *is_playing = false;
                    }
                }
            }

            // Request a repaint for next frame
            ctx.request_repaint();
        }
    }

    pub fn ui(&self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.update_progress(ctx);
        if !self.show_controls {
            return;
        }
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                let is_playing = *self.is_playing.lock().unwrap();
                let current_progress = *self.progress.lock().unwrap();
                let total_duration = *self.duration.lock().unwrap();

                // Check if audio has ended (not playing, progress is non-zero and near total duration)
                let audio_ended = !is_playing
                    && current_progress != Duration::ZERO
                    && total_duration != Duration::ZERO
                    && current_progress >= total_duration;

                if audio_ended {
                    // Show Restart button
                    if ui.button("↻").clicked() {
                        // Restart logic: reset progress, create new sink, play
                        *self.progress.lock().unwrap() = Duration::ZERO;
                        // Drop the current sink to ensure a new one is created
                        *self.sink.lock().unwrap() = None;
                        if let Ok(new_sink) = self.create_sink(Duration::ZERO) {
                            new_sink.play();
                            *self.is_playing.lock().unwrap() = true;
                            *self.sink.lock().unwrap() = Some(new_sink);
                            *self.last_play_instant.lock().unwrap() = Some(Instant::now());
                            // Duration should remain the same
                        }
                    }
                } else {
                    // Show Play/Pause button
                    if ui.button(if is_playing { "⏸" } else { "▶" }).clicked() {
                        self.toggle_playback();
                    }
                }

                let duration = *self.duration.lock().unwrap();
                let mut current = self.progress.lock().unwrap();
                let total_secs = duration.as_secs_f32();
                let mut pos_secs = current.as_secs_f32();

                ui.add_space(10.0);
                let older_slider_width = ui.style_mut().spacing.slider_width;
                ui.style_mut().spacing.slider_width = 400.0;
                let changed = ui
                    .add(egui::Slider::new(&mut pos_secs, 0.0..=total_secs).show_value(false))
                    .drag_stopped();
                ui.style_mut().spacing.slider_width = older_slider_width;

                if changed {
                    *current = Duration::from_secs_f32(pos_secs);
                    *self.sink.lock().unwrap() = None; // Drop the old sink
                    let last_is_playing = *self.is_playing.lock().unwrap();
                    if let Ok(new_sink) = self.create_sink(*current) {
                        if last_is_playing {
                            new_sink.play();
                        } // sink pause when created
                        *self.is_playing.lock().unwrap() = last_is_playing;
                        *self.sink.lock().unwrap() = Some(new_sink);
                        *self.last_play_instant.lock().unwrap() = Some(Instant::now());
                        *self.duration.lock().unwrap() = duration;
                    }
                }

                ui.label(format!(
                    "{} / {}",
                    format_duration(*current),
                    format_duration(duration)
                ));

                ui.add_space(10.0); // Add space after duration label

                ui.horizontal(|ui| {
                    ui.label("🔊");
                    let mut vol = *self.volume.lock().unwrap();
                    if ui.add(egui::Slider::new(&mut vol, 0.0..=6.)).drag_stopped() {
                        if let Some(sink) = self.sink.lock().unwrap().as_ref() {
                            sink.set_volume(vol);
                        }
                        *self.volume.lock().unwrap() = vol;
                    }
                });
            });
        });
    }
}

fn format_duration(dur: Duration) -> String {
    let secs = dur.as_secs();
    let minutes = secs / 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

impl fmt::Debug for AudioPlayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let is_playing = *self.is_playing.lock().unwrap();
        let progress = *self.progress.lock().unwrap();
        let duration = *self.duration.lock().unwrap();
        let sink_status = self
            .sink
            .lock()
            .unwrap()
            .as_ref()
            .map(|sink| {
                if sink.empty() {
                    "Finished"
                } else if sink.is_paused() {
                    "Paused"
                } else {
                    "Playing"
                }
            })
            .unwrap_or("None");

        f.debug_struct("AudioPlayer")
            .field("id", &self.id)
            .field("is_playing", &is_playing)
            .field("progress", &format_duration(progress))
            .field("duration", &format_duration(duration))
            .field("sink_status", &sink_status)
            .field("audio_data_len", &self.audio_data.len())
            .field("_stream", &"<rodio::OutputStream>")
            .field("stream_handle", &"<rodio::OutputStreamHandle>")
            .finish()
    }
}
