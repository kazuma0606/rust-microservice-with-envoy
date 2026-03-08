use crate::domain::value_object::TenantId;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Tenant {
    pub id: TenantId,
}

impl Tenant {
    #[allow(dead_code)]
    pub fn new(id: TenantId) -> Self {
        Self { id }
    }
}
