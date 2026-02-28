use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskLifecycle {
    Starting,
    Running,
    Failed,
    RestartPlanned,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestartIntent {
    pub should_restart: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SupervisedTask {
    pub id: TaskId,
    pub state: TaskLifecycle,
}

#[derive(Debug, Default)]
pub struct Supervisor {
    tasks: HashMap<TaskId, SupervisedTask>,
}

impl Supervisor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, id: TaskId) {
        self.tasks.insert(
            id,
            SupervisedTask {
                id,
                state: TaskLifecycle::Starting,
            },
        );
    }

    pub fn mark_running(&mut self, id: TaskId) {
        if let Some(task) = self.tasks.get_mut(&id) {
            task.state = TaskLifecycle::Running;
        }
    }

    pub fn mark_failed(&mut self, id: TaskId) -> RestartIntent {
        if let Some(task) = self.tasks.get_mut(&id) {
            task.state = TaskLifecycle::RestartPlanned;
        }
        RestartIntent {
            should_restart: true,
        }
    }
}
