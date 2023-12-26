use revolt_okapi::openapi3::SchemaObject;
use revolt_rocket_okapi::revolt_okapi::openapi3;
use rocket::{
    http::{ContentType, Status},
    response::{self, Responder},
    Response,
};
use schemars::schema::Schema;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Cursor;
use validator::ValidationErrors;

use crate::permissions::defn::{ChannelPermission, UserPermission};

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub enum Error {
    LabelMe,
    AlreadyOnboard,
    UsernameTaken,
    InvalidUsername,
    UnknownUser,
    AlreadyFriends,
    AlreadySentRequest,
    Blocked,
    BlockedByOther,
    NotFriends,

    UnknownChannel,
    UnknownAttachment,
    UnknownMessage,
    CannotEditMessage,
    CannotJoinCall,
    TooManyAttachments,
    TooManyReplies,
    EmptyMessage,
    PayloadTooLarge,
    CannotRemoveYourself,
    GroupTooLarge {
        max: usize,
    },
    AlreadyInGroup,
    NotInGroup,

    UnknownServer,
    InvalidRole,
    Banned,
    TooManyServers {
        max: usize,
    },
    TooManyEmoji,

    TooManyChannels {
        max: usize,
    },

    TooManyRoles {
        max: usize,
    },

    ReachedMaximumBots,
    IsBot,
    BotIsPrivate,

    CannotReportYourself,

    MissingPermission {
        permission: ChannelPermission,
    },
    MissingUserPermission {
        permission: UserPermission,
    },

    NotElevated,
    NotPrivileged,
    CannotGiveMissingPermissions,
    NotOwner,

    DatabaseError {
        operation: &'static str,
        with: &'static str,
    },
    InternalError,
    InvalidOperation,
    InvalidCredentials,
    InvalidProperty,
    InvalidSession,
    DuplicateNonce,
    VosoUnavailable,
    NotFound,
    NoEffect,
    FailedValidation {
        #[serde(skip_serializing, skip_deserializing)]
        error: ValidationErrors,
    },

    AlreadyInServer,
}

impl Error {
    pub fn from_invalid<T>(validation_error: ValidationErrors) -> Result<T> {
        Err(Error::FailedValidation {
            error: validation_error,
        })
    }

    pub fn from_permission<T>(permission: ChannelPermission) -> Result<T> {
        Err(if let ChannelPermission::ViewChannel = permission {
            Error::NotFound
        } else {
            Error::MissingPermission { permission }
        })
    }

    pub fn from_user_permission<T>(permission: UserPermission) -> Result<T> {
        Err(if let UserPermission::Access = permission {
            Error::NotFound
        } else {
            Error::MissingUserPermission { permission }
        })
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> response::Result<'static> {
        let status = match self {
            Error::LabelMe => Status::InternalServerError,

            Error::AlreadyOnboard => Status::Forbidden,

            Error::UnknownUser => Status::NotFound,
            Error::InvalidUsername => Status::BadRequest,
            Error::UsernameTaken => Status::Conflict,
            Error::AlreadyFriends => Status::Conflict,
            Error::AlreadySentRequest => Status::Conflict,
            Error::Blocked => Status::Conflict,
            Error::BlockedByOther => Status::Forbidden,
            Error::NotFriends => Status::Forbidden,

            Error::UnknownChannel => Status::NotFound,
            Error::UnknownMessage => Status::NotFound,
            Error::UnknownAttachment => Status::BadRequest,
            Error::CannotEditMessage => Status::Forbidden,
            Error::CannotJoinCall => Status::BadRequest,

            Error::EmptyMessage => Status::UnprocessableEntity,
            Error::PayloadTooLarge => Status::UnprocessableEntity,
            Error::CannotRemoveYourself => Status::BadRequest,
            Error::GroupTooLarge { .. } => Status::Forbidden,
            Error::AlreadyInGroup => Status::Conflict,
            Error::NotInGroup => Status::NotFound,

            Error::UnknownServer => Status::NotFound,
            Error::InvalidRole => Status::NotFound,
            Error::Banned => Status::Forbidden,

            Error::TooManyServers { .. } => Status::Forbidden,
            Error::TooManyChannels { .. } => Status::BadRequest,
            Error::TooManyRoles { .. } => Status::BadRequest,
            Error::TooManyEmoji => Status::BadRequest,
            Error::TooManyAttachments => Status::BadRequest,
            Error::TooManyReplies => Status::BadRequest,

            Error::ReachedMaximumBots => Status::BadRequest,
            Error::IsBot => Status::BadRequest,
            Error::BotIsPrivate => Status::Forbidden,

            Error::CannotReportYourself => Status::BadRequest,

            Error::MissingPermission { .. } => Status::Forbidden,
            Error::MissingUserPermission { .. } => Status::Forbidden,
            Error::NotElevated => Status::Forbidden,
            Error::NotPrivileged => Status::Forbidden,
            Error::CannotGiveMissingPermissions => Status::Forbidden,
            Error::NotOwner => Status::Forbidden,

            Error::DatabaseError { .. } => Status::InternalServerError,
            Error::InternalError => Status::InternalServerError,
            Error::InvalidOperation => Status::BadRequest,
            Error::InvalidCredentials => Status::Unauthorized,
            Error::InvalidProperty => Status::BadRequest,
            Error::InvalidSession => Status::Unauthorized,
            Error::DuplicateNonce => Status::Conflict,
            Error::VosoUnavailable => Status::BadRequest,
            Error::NotFound => Status::NotFound,
            Error::NoEffect => Status::Ok,
            Error::FailedValidation { .. } => Status::BadRequest,

            Error::AlreadyInServer => Status::Conflict,
        };

        let string = json!(self).to_string();

        Response::build()
            .sized_body(string.len(), Cursor::new(string))
            .header(ContentType::new("application", "json"))
            .status(status)
            .ok()
    }
}

impl revolt_rocket_okapi::response::OpenApiResponderInner for Error {
    fn responses(
        gen: &mut revolt_rocket_okapi::gen::OpenApiGenerator,
    ) -> std::result::Result<openapi3::Responses, revolt_rocket_okapi::OpenApiError> {
        let mut content = revolt_okapi::Map::new();

        let settings = schemars::gen::SchemaSettings::default().with(|s| {
            s.option_nullable = true;
            s.option_add_null_type = false;
            s.definitions_path = "#/components/schemas/".to_string();
        });

        let mut schema_generator = settings.into_generator();
        let schema = schema_generator.root_schema_for::<Error>();

        let definitions = gen.schema_generator().definitions_mut();
        for (key, value) in schema.definitions {
            definitions.insert(key, value);
        }

        definitions.insert("Error".to_string(), Schema::Object(schema.schema));

        content.insert(
            "application/json".to_string(),
            openapi3::MediaType {
                schema: Some(SchemaObject {
                    reference: Some("#/components/schemas/Error".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );

        Ok(openapi3::Responses {
            default: Some(openapi3::RefOr::Object(openapi3::Response {
                content,
                description: "An error occurred.".to_string(),
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}
