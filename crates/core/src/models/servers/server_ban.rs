use crate::auto_derived;

use super::server_member::MemberCompositeKey;

auto_derived!(
    pub struct ServerBan {
        #[cfg_attr(feature = "serde", serde(rename = "_id"))]
        pub id: MemberCompositeKey,
        pub reason: Option<String>,
    }
);
