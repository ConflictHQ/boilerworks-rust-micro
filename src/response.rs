use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            ok: true,
            message: None,
            data: Some(data),
            errors: vec![],
        }
    }

    pub fn success_with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            ok: true,
            message: Some(message.into()),
            data: Some(data),
            errors: vec![],
        }
    }
}

impl ApiResponse<()> {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            message: Some(message.into()),
            data: None,
            errors: vec![],
        }
    }

    pub fn error_with_details(message: impl Into<String>, errors: Vec<String>) -> Self {
        Self {
            ok: false,
            message: Some(message.into()),
            data: None,
            errors,
        }
    }
}
