use switchyard_core::{TraceEvent, TraceSink};

#[derive(Clone, Debug, Default)]
pub struct TraceLog {
    events: Vec<TraceEvent>,
}

impl TraceLog {
    pub fn events(&self) -> &[TraceEvent] {
        &self.events
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        for event in &self.events {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&render_event(*event));
        }
        out
    }
}

impl TraceSink for TraceLog {
    fn on_event(&mut self, event: TraceEvent) {
        self.events.push(event);
    }
}

fn render_event(event: TraceEvent) -> String {
    match event {
        TraceEvent::TickStarted { clock } => format!("tick_started clock={clock}"),
        TraceEvent::TickCompleted { report } => format!(
            "tick_completed clock={} actions={} progress={}",
            report.clock, report.actions_emitted, report.progress_made
        ),
        TraceEvent::SignalQueued { signal } => format!("signal_queued signal={}", signal.0),
        TraceEvent::TaskSpawned { task, program_id, parent, scope_root } => match parent {
            Some(parent) => format!(
                "task_spawned task={} program={} parent={} scope_root={}",
                task.0, program_id.0, parent.0, scope_root.0
            ),
            None => format!(
                "task_spawned task={} program={} parent=root scope_root={}",
                task.0, program_id.0, scope_root.0
            ),
        },
        TraceEvent::TaskWaiting { task, reason } => {
            format!("task_waiting task={} reason={}", task.0, render_wait_reason(reason))
        }
        TraceEvent::TaskWoken { task, reason } => {
            format!("task_woken task={} reason={}", task.0, render_wait_reason(reason))
        }
        TraceEvent::ActionEmitted { task, action } => {
            format!("action_emitted task={} action={}", task.0, action.0)
        }
        TraceEvent::TaskFinished { task, outcome } => {
            format!("task_finished task={} outcome={:?}", task.0, outcome)
        }
    }
}

fn render_wait_reason(reason: switchyard_core::WaitReason) -> String {
    match reason {
        switchyard_core::WaitReason::Ready => "ready".to_owned(),
        switchyard_core::WaitReason::Ticks { until_tick } => format!("ticks:{until_tick}"),
        switchyard_core::WaitReason::Signal(signal) => format!("signal:{}", signal.0),
        switchyard_core::WaitReason::Predicate(predicate) => format!("predicate:{}", predicate.0),
        switchyard_core::WaitReason::ChildrenAll => "children_all".to_owned(),
        switchyard_core::WaitReason::Race { left, right } => format!("race:{}:{}", left.0, right.0),
    }
}
