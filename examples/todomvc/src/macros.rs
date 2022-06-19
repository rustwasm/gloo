macro_rules! handle_err {
    ($app:expr, $body:expr) => {
        match $body {
            Ok(v) => v,
            Err(e) => {
                $app.set(crate::app::App::Error(::std::sync::Arc::new(e.to_string())));
                return;
            }
        }
    };
}
