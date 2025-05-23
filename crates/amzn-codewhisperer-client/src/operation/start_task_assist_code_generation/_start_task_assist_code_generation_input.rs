// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// Structure to represent start code generation request.
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct StartTaskAssistCodeGenerationInput {
    /// Structure to represent the current state of a chat conversation.
    pub conversation_state: ::std::option::Option<crate::types::ConversationState>,
    /// Represents a Workspace state uploaded to S3 for Async Code Actions
    pub workspace_state: ::std::option::Option<crate::types::WorkspaceState>,
    #[allow(missing_docs)] // documentation missing in model
    pub task_assist_plan: ::std::option::Option<::std::vec::Vec<crate::types::TaskAssistPlanStep>>,
    /// ID which represents a single code generation in a conversation
    pub code_generation_id: ::std::option::Option<::std::string::String>,
    /// ID which represents a single code generation in a conversation
    pub current_code_generation_id: ::std::option::Option<::std::string::String>,
    #[allow(missing_docs)] // documentation missing in model
    pub intent: ::std::option::Option<crate::types::Intent>,
    #[allow(missing_docs)] // documentation missing in model
    pub intent_context: ::std::option::Option<crate::types::IntentContext>,
    #[allow(missing_docs)] // documentation missing in model
    pub profile_arn: ::std::option::Option<::std::string::String>,
}
impl StartTaskAssistCodeGenerationInput {
    /// Structure to represent the current state of a chat conversation.
    pub fn conversation_state(&self) -> ::std::option::Option<&crate::types::ConversationState> {
        self.conversation_state.as_ref()
    }

    /// Represents a Workspace state uploaded to S3 for Async Code Actions
    pub fn workspace_state(&self) -> ::std::option::Option<&crate::types::WorkspaceState> {
        self.workspace_state.as_ref()
    }

    #[allow(missing_docs)] // documentation missing in model
    /// If no value was sent for this field, a default will be set. If you want to determine if no
    /// value was sent, use `.task_assist_plan.is_none()`.
    pub fn task_assist_plan(&self) -> &[crate::types::TaskAssistPlanStep] {
        self.task_assist_plan.as_deref().unwrap_or_default()
    }

    /// ID which represents a single code generation in a conversation
    pub fn code_generation_id(&self) -> ::std::option::Option<&str> {
        self.code_generation_id.as_deref()
    }

    /// ID which represents a single code generation in a conversation
    pub fn current_code_generation_id(&self) -> ::std::option::Option<&str> {
        self.current_code_generation_id.as_deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn intent(&self) -> ::std::option::Option<&crate::types::Intent> {
        self.intent.as_ref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn intent_context(&self) -> ::std::option::Option<&crate::types::IntentContext> {
        self.intent_context.as_ref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn profile_arn(&self) -> ::std::option::Option<&str> {
        self.profile_arn.as_deref()
    }
}
impl StartTaskAssistCodeGenerationInput {
    /// Creates a new builder-style object to manufacture
    /// [`StartTaskAssistCodeGenerationInput`](crate::operation::start_task_assist_code_generation::StartTaskAssistCodeGenerationInput).
    pub fn builder()
    -> crate::operation::start_task_assist_code_generation::builders::StartTaskAssistCodeGenerationInputBuilder {
        crate::operation::start_task_assist_code_generation::builders::StartTaskAssistCodeGenerationInputBuilder::default()
    }
}

/// A builder for
/// [`StartTaskAssistCodeGenerationInput`](crate::operation::start_task_assist_code_generation::StartTaskAssistCodeGenerationInput).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct StartTaskAssistCodeGenerationInputBuilder {
    pub(crate) conversation_state: ::std::option::Option<crate::types::ConversationState>,
    pub(crate) workspace_state: ::std::option::Option<crate::types::WorkspaceState>,
    pub(crate) task_assist_plan: ::std::option::Option<::std::vec::Vec<crate::types::TaskAssistPlanStep>>,
    pub(crate) code_generation_id: ::std::option::Option<::std::string::String>,
    pub(crate) current_code_generation_id: ::std::option::Option<::std::string::String>,
    pub(crate) intent: ::std::option::Option<crate::types::Intent>,
    pub(crate) intent_context: ::std::option::Option<crate::types::IntentContext>,
    pub(crate) profile_arn: ::std::option::Option<::std::string::String>,
}
impl StartTaskAssistCodeGenerationInputBuilder {
    /// Structure to represent the current state of a chat conversation.
    /// This field is required.
    pub fn conversation_state(mut self, input: crate::types::ConversationState) -> Self {
        self.conversation_state = ::std::option::Option::Some(input);
        self
    }

    /// Structure to represent the current state of a chat conversation.
    pub fn set_conversation_state(mut self, input: ::std::option::Option<crate::types::ConversationState>) -> Self {
        self.conversation_state = input;
        self
    }

    /// Structure to represent the current state of a chat conversation.
    pub fn get_conversation_state(&self) -> &::std::option::Option<crate::types::ConversationState> {
        &self.conversation_state
    }

    /// Represents a Workspace state uploaded to S3 for Async Code Actions
    /// This field is required.
    pub fn workspace_state(mut self, input: crate::types::WorkspaceState) -> Self {
        self.workspace_state = ::std::option::Option::Some(input);
        self
    }

    /// Represents a Workspace state uploaded to S3 for Async Code Actions
    pub fn set_workspace_state(mut self, input: ::std::option::Option<crate::types::WorkspaceState>) -> Self {
        self.workspace_state = input;
        self
    }

    /// Represents a Workspace state uploaded to S3 for Async Code Actions
    pub fn get_workspace_state(&self) -> &::std::option::Option<crate::types::WorkspaceState> {
        &self.workspace_state
    }

    /// Appends an item to `task_assist_plan`.
    ///
    /// To override the contents of this collection use
    /// [`set_task_assist_plan`](Self::set_task_assist_plan).
    pub fn task_assist_plan(mut self, input: crate::types::TaskAssistPlanStep) -> Self {
        let mut v = self.task_assist_plan.unwrap_or_default();
        v.push(input);
        self.task_assist_plan = ::std::option::Option::Some(v);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_task_assist_plan(
        mut self,
        input: ::std::option::Option<::std::vec::Vec<crate::types::TaskAssistPlanStep>>,
    ) -> Self {
        self.task_assist_plan = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_task_assist_plan(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::TaskAssistPlanStep>> {
        &self.task_assist_plan
    }

    /// ID which represents a single code generation in a conversation
    pub fn code_generation_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.code_generation_id = ::std::option::Option::Some(input.into());
        self
    }

    /// ID which represents a single code generation in a conversation
    pub fn set_code_generation_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.code_generation_id = input;
        self
    }

    /// ID which represents a single code generation in a conversation
    pub fn get_code_generation_id(&self) -> &::std::option::Option<::std::string::String> {
        &self.code_generation_id
    }

    /// ID which represents a single code generation in a conversation
    pub fn current_code_generation_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.current_code_generation_id = ::std::option::Option::Some(input.into());
        self
    }

    /// ID which represents a single code generation in a conversation
    pub fn set_current_code_generation_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.current_code_generation_id = input;
        self
    }

    /// ID which represents a single code generation in a conversation
    pub fn get_current_code_generation_id(&self) -> &::std::option::Option<::std::string::String> {
        &self.current_code_generation_id
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn intent(mut self, input: crate::types::Intent) -> Self {
        self.intent = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_intent(mut self, input: ::std::option::Option<crate::types::Intent>) -> Self {
        self.intent = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_intent(&self) -> &::std::option::Option<crate::types::Intent> {
        &self.intent
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn intent_context(mut self, input: crate::types::IntentContext) -> Self {
        self.intent_context = ::std::option::Option::Some(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_intent_context(mut self, input: ::std::option::Option<crate::types::IntentContext>) -> Self {
        self.intent_context = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_intent_context(&self) -> &::std::option::Option<crate::types::IntentContext> {
        &self.intent_context
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn profile_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.profile_arn = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_profile_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.profile_arn = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_profile_arn(&self) -> &::std::option::Option<::std::string::String> {
        &self.profile_arn
    }

    /// Consumes the builder and constructs a
    /// [`StartTaskAssistCodeGenerationInput`](crate::operation::start_task_assist_code_generation::StartTaskAssistCodeGenerationInput).
    pub fn build(
        self,
    ) -> ::std::result::Result<
        crate::operation::start_task_assist_code_generation::StartTaskAssistCodeGenerationInput,
        ::aws_smithy_types::error::operation::BuildError,
    > {
        ::std::result::Result::Ok(
            crate::operation::start_task_assist_code_generation::StartTaskAssistCodeGenerationInput {
                conversation_state: self.conversation_state,
                workspace_state: self.workspace_state,
                task_assist_plan: self.task_assist_plan,
                code_generation_id: self.code_generation_id,
                current_code_generation_id: self.current_code_generation_id,
                intent: self.intent,
                intent_context: self.intent_context,
                profile_arn: self.profile_arn,
            },
        )
    }
}
