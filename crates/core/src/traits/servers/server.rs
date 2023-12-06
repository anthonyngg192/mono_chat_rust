use crate::models::server::{FieldsRole, FieldsServer, PartialRole, PartialServer, Role, Server};
use crate::Result;

#[async_trait]
pub trait AbstractServer: Sync + Send {
    async fn fetch_server(&self, id: &str) -> Result<Server>;
    async fn fetch_servers<'a>(&self, ids: &'a [String]) -> Result<Vec<Server>>;
    async fn insert_server(&self, server: &Server) -> Result<()>;
    async fn update_server(
        &self,
        id: &str,
        server: &PartialServer,
        remove: Vec<FieldsServer>,
    ) -> Result<()>;
    async fn delete_server(&self, server: &Server) -> Result<()>;
    async fn insert_role(&self, server_id: &str, role_id: &str, role: &Role) -> Result<()>;
    async fn update_role(
        &self,
        server_id: &str,
        role_id: &str,
        role: &PartialRole,
        remove: Vec<FieldsRole>,
    ) -> Result<()>;
    async fn delete_role(&self, server_id: &str, role_id: &str) -> Result<()>;
}
