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

    pub fn mark_running(&mut self, id: TaskId) -> bool {
        self.transition_to(id, TaskLifecycle::Starting, TaskLifecycle::Running)
    }

    pub fn mark_failed(&mut self, id: TaskId) -> Option<RestartIntent> {
        if self.transition_to(id, TaskLifecycle::Running, TaskLifecycle::RestartPlanned) {
            Some(RestartIntent {
                should_restart: true,
            })
        } else {
            None
        }
    }

    pub fn mark_stopped(&mut self, id: TaskId) -> bool {
        if self.transition_to(id, TaskLifecycle::Running, TaskLifecycle::Stopped) {
            true
        } else {
            self.transition_to(id, TaskLifecycle::RestartPlanned, TaskLifecycle::Stopped)
        }
    }

    fn transition_to(&mut self, id: TaskId, from: TaskLifecycle, to: TaskLifecycle) -> bool {
        match self.tasks.get_mut(&id) {
            Some(task) if task.state == from => {
                task.state = to;
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Supervisor, TaskId, TaskLifecycle};

    #[test]
    fn legal_lifecycle_path_transitions_through_expected_states() {
        let mut supervisor = Supervisor::new();
        let task_id = TaskId(7);
        supervisor.register(task_id);

        assert!(supervisor.mark_running(task_id));

        let restart = supervisor.mark_failed(task_id);
        let task = supervisor.tasks.get(&task_id).copied().unwrap();

        assert_eq!(task.state, TaskLifecycle::RestartPlanned);
        assert!(restart.unwrap().should_restart);

        assert!(supervisor.mark_stopped(task_id));
        let task = supervisor.tasks.get(&task_id).copied().unwrap();
        assert_eq!(task.state, TaskLifecycle::Stopped);
    }

    #[test]
    fn illegal_transitions_return_failure_and_do_not_mutate_state() {
        let mut supervisor = Supervisor::new();
        let task_id = TaskId(11);
        supervisor.register(task_id);

        assert!(supervisor.mark_running(task_id));
        assert!(supervisor.mark_failed(task_id).is_some());

        let before = supervisor.tasks.get(&task_id).copied().unwrap();
        assert_eq!(before.state, TaskLifecycle::RestartPlanned);

        assert!(!supervisor.mark_running(task_id));
        let after = supervisor.tasks.get(&task_id).copied().unwrap();
        assert_eq!(after.state, TaskLifecycle::RestartPlanned);

        assert!(supervisor.mark_stopped(task_id));

        let stopped = supervisor.tasks.get(&task_id).copied().unwrap();
        assert_eq!(stopped.state, TaskLifecycle::Stopped);

        assert!(!supervisor.mark_running(task_id));
        let after_stopped = supervisor.tasks.get(&task_id).copied().unwrap();
        assert_eq!(after_stopped.state, TaskLifecycle::Stopped);
    }

    #[test]
    fn unknown_task_operations_remain_distinct() {
        let mut supervisor = Supervisor::new();
        let unknown = TaskId(99);

        let restart = supervisor.mark_failed(unknown);

        assert!(restart.is_none());
        assert!(!supervisor.mark_running(unknown));
        assert!(!supervisor.mark_stopped(unknown));
        assert!(!supervisor.tasks.contains_key(&unknown));
    }
}
