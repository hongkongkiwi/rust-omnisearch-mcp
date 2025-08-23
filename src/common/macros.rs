//! Common macros for providers

/// Macro to create a simple provider with basic HTTP client
#[macro_export]
macro_rules! create_simple_provider {
    ($struct_name:ident, $timeout:expr) => {
        impl $struct_name {
            pub fn new() -> Self {
                let client = $crate::common::http::create_http_client($timeout);
                Self { client }
            }
        }
    };
}

/// Macro to handle common HTTP error responses
#[macro_export]
macro_rules! handle_provider_http_error {
    ($status:expr, $error_message:expr, $self:expr, $rate_limit_msg:expr, $auth_error_msg:expr, $forbidden_msg:expr, $internal_error_msg:expr) => {
        $crate::common::http::handle_http_error(
            $status,
            $error_message,
            $self.name(),
            $rate_limit_msg,
            $auth_error_msg,
            $forbidden_msg,
            $internal_error_msg,
        )
    };
}