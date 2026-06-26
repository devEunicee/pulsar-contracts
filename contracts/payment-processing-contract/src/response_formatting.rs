use soroban_sdk::{Env, String};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResponseMetadata {
    pub timestamp: u64,
    pub version: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaginationInfo {
    pub limit: u32,
    pub offset: u32,
    pub total: u32,
    pub has_more: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiResponse {
    pub success: bool,
    pub status_code: u32,
    pub message: String,
    pub data: Option<String>,
    pub metadata: ResponseMetadata,
    pub pagination: Option<PaginationInfo>,
    pub error_code: Option<String>,
}

pub fn format_success_response(
    env: &Env,
    message: &str,
    data: Option<&str>,
    pagination: Option<PaginationInfo>,
) -> ApiResponse {
    ApiResponse {
        success: true,
        status_code: 200,
        message: String::from_str(env, message),
        data: data.map(|value| String::from_str(env, value)),
        metadata: ResponseMetadata {
            timestamp: 0,
            version: 1,
        },
        pagination,
        error_code: None,
    }
}

pub fn format_error_response(env: &Env, message: &str, status_code: u32) -> ApiResponse {
    ApiResponse {
        success: false,
        status_code,
        message: String::from_str(env, message),
        data: None,
        metadata: ResponseMetadata {
            timestamp: 0,
            version: 1,
        },
        pagination: None,
        error_code: Some(String::from_str(env, "request_failed")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_success_payload_with_metadata_and_pagination() {
        let env = Env::default();
        let response = format_success_response(
            &env,
            "ok",
            Some("payment-created"),
            Some(PaginationInfo {
                limit: 10,
                offset: 0,
                total: 25,
                has_more: true,
            }),
        );

        assert!(response.success);
        assert_eq!(response.status_code, 200);
        assert!(response.pagination.is_some());
        assert_eq!(response.metadata.version, 1);
    }

    #[test]
    fn formats_error_payload_with_status_code() {
        let env = Env::default();
        let response = format_error_response(&env, "invalid request", 400);

        assert!(!response.success);
        assert_eq!(response.status_code, 400);
        assert!(response.error_code.is_some());
    }
}
