// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// This exception is thrown when the user does not have sufficient access to perform this action.
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct AccessDeniedError {
    #[allow(missing_docs)] // documentation missing in model
    pub message: ::std::string::String,
    /// Reason for AccessDeniedException
    pub reason: ::std::option::Option<crate::types::AccessDeniedExceptionReason>,
    pub(crate) meta: ::aws_smithy_types::error::ErrorMetadata,
}
impl AccessDeniedError {
    /// Reason for AccessDeniedException
    pub fn reason(&self) -> ::std::option::Option<&crate::types::AccessDeniedExceptionReason> {
        self.reason.as_ref()
    }
}
impl AccessDeniedError {
    /// Returns the error message.
    pub fn message(&self) -> &str {
        &self.message
    }
}
impl ::std::fmt::Display for AccessDeniedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ::std::write!(f, "AccessDeniedError")?;
        {
            ::std::write!(f, ": {}", &self.message)?;
        }
        Ok(())
    }
}
impl ::std::error::Error for AccessDeniedError {}
impl ::aws_types::request_id::RequestId for crate::types::error::AccessDeniedError {
    fn request_id(&self) -> Option<&str> {
        use ::aws_smithy_types::error::metadata::ProvideErrorMetadata;
        self.meta().request_id()
    }
}
impl ::aws_smithy_types::error::metadata::ProvideErrorMetadata for AccessDeniedError {
    fn meta(&self) -> &::aws_smithy_types::error::ErrorMetadata {
        &self.meta
    }
}
impl AccessDeniedError {
    /// Creates a new builder-style object to manufacture
    /// [`AccessDeniedError`](crate::types::error::AccessDeniedError).
    pub fn builder() -> crate::types::error::builders::AccessDeniedErrorBuilder {
        crate::types::error::builders::AccessDeniedErrorBuilder::default()
    }
}

/// A builder for [`AccessDeniedError`](crate::types::error::AccessDeniedError).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct AccessDeniedErrorBuilder {
    pub(crate) message: ::std::option::Option<::std::string::String>,
    pub(crate) reason: ::std::option::Option<crate::types::AccessDeniedExceptionReason>,
    meta: std::option::Option<::aws_smithy_types::error::ErrorMetadata>,
}
impl AccessDeniedErrorBuilder {
    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn message(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.message = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_message(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.message = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_message(&self) -> &::std::option::Option<::std::string::String> {
        &self.message
    }

    /// Reason for AccessDeniedException
    pub fn reason(mut self, input: crate::types::AccessDeniedExceptionReason) -> Self {
        self.reason = ::std::option::Option::Some(input);
        self
    }

    /// Reason for AccessDeniedException
    pub fn set_reason(mut self, input: ::std::option::Option<crate::types::AccessDeniedExceptionReason>) -> Self {
        self.reason = input;
        self
    }

    /// Reason for AccessDeniedException
    pub fn get_reason(&self) -> &::std::option::Option<crate::types::AccessDeniedExceptionReason> {
        &self.reason
    }

    /// Sets error metadata
    pub fn meta(mut self, meta: ::aws_smithy_types::error::ErrorMetadata) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Sets error metadata
    pub fn set_meta(&mut self, meta: std::option::Option<::aws_smithy_types::error::ErrorMetadata>) -> &mut Self {
        self.meta = meta;
        self
    }

    /// Consumes the builder and constructs a
    /// [`AccessDeniedError`](crate::types::error::AccessDeniedError). This method will fail if
    /// any of the following fields are not set:
    /// - [`message`](crate::types::error::builders::AccessDeniedErrorBuilder::message)
    pub fn build(
        self,
    ) -> ::std::result::Result<crate::types::error::AccessDeniedError, ::aws_smithy_types::error::operation::BuildError>
    {
        ::std::result::Result::Ok(crate::types::error::AccessDeniedError {
            message: self.message.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "message",
                    "message was not specified but it is required when building AccessDeniedError",
                )
            })?,
            reason: self.reason,
            meta: self.meta.unwrap_or_default(),
        })
    }
}
