use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::transcode_task::TranscodeTask;
use crate::transcoder::Transcoder;

pub enum QueueStatus {
    Idle,
    Processing,
}

#[derive(Clone, Debug)]
pub struct TranscodingMessage {
    pub frames: usize,
    pub total_frames: usize,
    pub progress: f64,
    pub current_size: usize,
    pub predicted_size: usize,
    pub task: TranscodeTask,
}

#[derive(Clone, Debug)]
pub enum QueueProcessorMessage {
    Idle,
    TranscodeStart(TranscodeTask),
    Transcoding(TranscodingMessage),
    TranscodeEnd(TranscodeTask, usize),
}

pub struct QueueProcessor {
    queue: Arc<Mutex<VecDeque<TranscodeTask>>>,
    stop: Arc<Mutex<bool>>,
    subscribers: Vec<Sender<QueueProcessorMessage>>,
    pub status: QueueStatus,
}

impl QueueProcessor {
    pub fn new(queue: Arc<Mutex<VecDeque<TranscodeTask>>>, stop: Arc<Mutex<bool>>) -> Self {
        QueueProcessor {
            queue,
            stop,
            subscribers: vec![],
            status: QueueStatus::Idle,
        }
    }

    pub fn enqueue(&mut self, transcode_task: TranscodeTask) {
        let mut q = self.queue.lock().unwrap();
        q.push_back(transcode_task);
    }

    fn should_continue(&self) -> bool { !self.should_stop() }

    fn should_stop(&self) -> bool { *self.stop.lock().unwrap() }

    fn next_item(&self) -> Option<TranscodeTask> {
        let mut q = self.queue.lock().unwrap();
        match q.pop_front() {
            None => None,
            Some(item) => Some(item),
        }
    }

    pub fn stop(&mut self) {
        let mut s = self.stop.lock().unwrap();
        *s = true;
    }

    pub fn subscribe(&mut self) -> Receiver<QueueProcessorMessage> {
        let (tx, rx) = mpsc::channel();
        self.subscribers.push(tx);
        rx
    }

    fn publish(&self, msg: QueueProcessorMessage) {
        for tx in &self.subscribers {
            let _ = tx.send(msg.clone());
        }
    }

    pub fn forever(&mut self) {
        while self.should_continue() {
            match self.next_item() {
                None => thread::sleep(Duration::from_secs(1)),
                Some(task) => {
                    self.status = QueueStatus::Processing;
                    self.process_transcode_task(task);
                    self.status = QueueStatus::Idle;
                },
            };
        }
    }

    fn process_transcode_task(&self, transcode_task: TranscodeTask) {
        self.publish(QueueProcessorMessage::TranscodeStart(transcode_task.clone()));
        let mut compressed_size: usize = 0;
        let (tx, rx) = mpsc::channel();

        let transcoder = Transcoder::default(Some(Arc::clone(&self.stop)))
            .codec(transcode_task.codec.clone())
            .container(transcode_task.container)
            .quality(transcode_task.quality);
        let source = transcode_task.source.clone();
        let metadata = transcode_task.metadata.clone();
        let destination = transcode_task.destination.clone();
        let transcoder_thread = thread::spawn(move || {
            let _ = transcoder.transcode(
                source,
                Some(metadata),
                destination,
                Some(tx));
        });

        for msg in rx {
            compressed_size = msg.predicted_compressed_size;
            self.publish(QueueProcessorMessage::Transcoding(TranscodingMessage {
                frames: msg.frames,
                total_frames: msg.total_frames,
                progress: msg.progress,
                current_size: msg.current_size,
                predicted_size: compressed_size,
                task: transcode_task.clone(),
            }));
        }

        let _ = transcoder_thread.join();

        self.publish(QueueProcessorMessage::TranscodeEnd(transcode_task.clone(), compressed_size));
    }
}
