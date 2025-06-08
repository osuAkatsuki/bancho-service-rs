#[macro_export]
macro_rules! cron_tasks {
    ($ctx:expr, $($t:path),* $(,)?) => {
        $({
            const TASK_NAME: &str = const_str::convert_ascii_case!(upper_camel, stringify!($t));
            let now = std::time::Instant::now();
            tracing::info!("Starting Task {TASK_NAME}");
            match ($t)($ctx).await {
                Ok(v) => tracing::info!("Completed Task {TASK_NAME} in {:?} with result {v:?}", now.elapsed()),
                Err(e) => tracing::error!("Error occurred in {TASK_NAME}: {e:?}"),
            }
        })*
    };
}
