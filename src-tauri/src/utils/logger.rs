use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tracing::Subscriber;
use tracing_subscriber::Layer;

#[derive(Debug, Clone, Serialize)]
pub struct LogEvent {
    pub timestamp: i64,
    pub level: String,
    pub message: String,
    pub target: String,
}

pub struct TauriLayer {
    app_handle: AppHandle,
}

impl TauriLayer {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

impl<S> Layer<S> for TauriLayer
where
    S: Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        use tracing::field::Visit;
        
        struct MessageVisitor(String);
        
        impl Visit for MessageVisitor {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.0 = format!("{:?}", value);
                    // Remove quotes
                    if self.0.starts_with('"') && self.0.ends_with('"') {
                        self.0 = self.0[1..self.0.len()-1].to_string();
                    }
                }
            }
        }
        
        let mut visitor = MessageVisitor(String::new());
        event.record(&mut visitor);
        
        let log_event = LogEvent {
            timestamp: chrono::Utc::now().timestamp_millis(),
            level: event.metadata().level().to_string().to_lowercase(),
            message: visitor.0,
            target: event.metadata().target().to_string(),
        };
        
        // 只发送应用相关的日志，过滤掉 tauri 内部日志
        if log_event.target.starts_with("patentupload") {
            let _ = self.app_handle.emit("rust-log", &log_event);
        }
    }
}
