use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::models::{mask_santri_name, VisitDetailView};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseMessage {
    pub event_name: String,
    pub target_dormitory_id: Option<String>,
    pub visit_id: String,
    pub ticket_code: String,
    pub current_status: String,
    pub santri_name: String,
    pub room_name: String,
    pub meeting_spot: Option<String>,
    pub has_titipan: bool,
    pub titipan_description: Option<String>,
    pub timestamp: String,
}

pub struct SseBroadcaster {
    tx: broadcast::Sender<SseMessage>,
}

impl SseBroadcaster {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(256);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SseMessage> {
        self.tx.subscribe()
    }

    pub fn emit_dormitory_event(&self, event_name: &str, detail: &VisitDetailView) {
        let msg = SseMessage {
            event_name: event_name.to_string(),
            target_dormitory_id: Some(detail.dormitory_id.clone()),
            visit_id: detail.id.clone(),
            ticket_code: detail.ticket_code.clone(),
            current_status: detail.current_status.clone(),
            santri_name: detail.santri_name.clone(),
            room_name: detail.room_name.clone(),
            meeting_spot: detail.meeting_spot.clone(),
            has_titipan: detail.has_titipan == 1,
            titipan_description: detail.titipan_description.clone(),
            timestamp: detail.updated_at.clone(),
        };
        let _ = self.tx.send(msg);
    }

    pub fn emit_public_flow_event(&self, detail: &VisitDetailView) {
        let msg = SseMessage {
            event_name: "public_flow_updated".to_string(),
            target_dormitory_id: None,
            visit_id: detail.id.clone(),
            ticket_code: detail.ticket_code.clone(),
            current_status: detail.current_status.clone(),
            santri_name: mask_santri_name(&detail.santri_name),
            room_name: String::new(),
            meeting_spot: detail.meeting_spot.clone(),
            has_titipan: detail.has_titipan == 1,
            titipan_description: None,
            timestamp: detail.updated_at.clone(),
        };
        let _ = self.tx.send(msg);
    }
}
