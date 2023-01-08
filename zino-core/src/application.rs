use crate::{AsyncCronJob, CronJob, Job, JobScheduler, State};
use std::{collections::HashMap, sync::LazyLock, thread, time::Instant};
use toml::value::Table;

/// Application.
pub trait Application {
    /// Router.
    type Router;

    /// Creates a new application.
    fn new() -> Self;

    /// Returns the start time.
    fn start_time(&self) -> Instant;

    /// Registers routes.
    fn register(self, routes: HashMap<&'static str, Self::Router>) -> Self;

    /// Runs the application.
    fn run(self, async_jobs: HashMap<&'static str, AsyncCronJob>);

    /// Returns the application env.
    #[inline]
    fn env() -> &'static str {
        SHARED_STATE.env()
    }

    /// Returns a reference to the application scoped config.
    #[inline]
    fn config() -> &'static Table {
        SHARED_STATE.config()
    }

    /// Spawns a new thread to run cron jobs.
    fn spawn(self, jobs: HashMap<&'static str, CronJob>) -> Self
    where
        Self: Sized,
    {
        let mut scheduler = JobScheduler::new();
        for (cron_expr, exec) in jobs {
            scheduler.add(Job::new(cron_expr, exec));
        }
        thread::spawn(move || loop {
            scheduler.tick();
            thread::sleep(scheduler.time_till_next_job());
        });
        self
    }
}

/// Shared application state.
static SHARED_STATE: LazyLock<State> = LazyLock::new(State::default);