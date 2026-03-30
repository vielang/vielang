use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Khớp với bảng `tb_user` (đặt tên tb_user để tránh conflict với từ khóa SQL).
/// Java: org.thingsboard.server.common.data.User
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,
    pub customer_id: Option<Uuid>,

    pub email: String,
    pub authority: Authority,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,

    pub additional_info: Option<serde_json::Value>,

    pub version: i64,
}

/// Java: org.thingsboard.server.common.data.security.Authority
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Authority {
    SysAdmin,
    TenantAdmin,
    CustomerUser,
    RefreshToken,
    PreVerificationToken,
}

impl User {
    pub fn get_title(&self) -> String {
        match (&self.first_name, &self.last_name) {
            (Some(f), Some(l)) if !f.is_empty() || !l.is_empty() => {
                format!("{} {}", f, l).trim().to_string()
            }
            _ => self.email.clone(),
        }
    }

    pub fn is_system_admin(&self) -> bool {
        self.authority == Authority::SysAdmin
    }

    pub fn is_tenant_admin(&self) -> bool {
        self.authority == Authority::TenantAdmin
    }

    pub fn is_customer_user(&self) -> bool {
        self.authority == Authority::CustomerUser
    }
}

/// Credentials lưu riêng (bảng `user_credentials`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentials {
    pub id: Uuid,
    pub created_time: i64,
    pub user_id: Uuid,
    pub enabled: bool,
    /// Hashed bằng Argon2 (thay bcrypt của Java)
    pub password: Option<String>,
    /// Token kích hoạt tài khoản
    pub activate_token: Option<String>,
    /// Token reset mật khẩu
    pub reset_token: Option<String>,
    pub additional_info: Option<serde_json::Value>,
}
