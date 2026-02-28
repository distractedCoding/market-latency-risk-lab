use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskLifecycle {
    Starting,
    Running,
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

    pub fn mark_failed(&mut self, id: TaskId) -> Option<RestartIntent> {
        self.tasks.get_mut(&id).map(|task| {
            task.state = TaskLifecycle::RestartPlanned;
            RestartIntent {
                should_restart: true,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Supervisor, TaskId, TaskLifecycle};

    #[test]
    fn mark_failed_returns_none_for_unknown_task() {
        let mut supervisor = Supervisor::new();

        let restart = supervisor.mark_failed(TaskId(99));

        assert!(restart.is_none());
    }

    #[test]
    fn known_task_transitions_to_restart_planned_when_failed() {
        let mut supervisor = Supervisor::new();
        let task_id = TaskId(7);
        supervisor.register(task_id);
        supervisor.mark_running(task_id);

        let restart = supervisor.mark_failed(task_id);
        let task = supervisor.tasks.get(&task_id).copied().unwrap();

        assert_eq!(task.state, TaskLifecycle::RestartPlanned);
        assert_eq!(restart.unwrap().should_restart, true);
    }
}
