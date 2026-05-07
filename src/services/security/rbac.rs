//! RBAC 权限模型
//!
//! 基于角色的访问控制 (Role-Based Access Control):
//! - Roles: Admin, Operator, User, Guest
//! - Permissions: read, write, execute, admin
//!
//! 特性:
//! - 权限检查中间件
//! - 默认角色权限
//! - 动态角色分配

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// RBAC 错误类型
#[derive(Error, Debug)]
pub enum RbacError {
    #[error("锁被毒化: {0}")]
    LockPoisoned(String),
    #[error("角色不存在: {0}")]
    RoleNotFound(String),
    #[error("权限不足: {0}")]
    PermissionDenied(String),
    #[error("用户不存在: {0}")]
    UserNotFound(String),
    #[error("无效操作: {0}")]
    InvalidOperation(String),
}

/// 权限类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    /// 读取权限
    Read,
    /// 写入权限
    Write,
    /// 执行权限
    Execute,
    /// 管理权限
    Admin,
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Permission::Read => write!(f, "read"),
            Permission::Write => write!(f, "write"),
            Permission::Execute => write!(f, "execute"),
            Permission::Admin => write!(f, "admin"),
        }
    }
}

/// 角色类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// 管理员 - 完全访问
    Admin,
    /// 操作员 - 运维操作
    Operator,
    /// 普通用户 - 基本访问
    User,
    /// 访客 - 最小权限
    Guest,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::Operator => write!(f, "operator"),
            Role::User => write!(f, "user"),
            Role::Guest => write!(f, "guest"),
        }
    }
}

/// 用户角色分配
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    /// 用户 ID
    pub user_id: String,
    /// 用户名
    pub username: String,
    /// 角色
    pub role: Role,
    /// 自定义权限 (可选)
    pub custom_permissions: Option<Vec<Permission>>,
    /// 有效期 (可选)
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl UserRole {
    /// 创建新的用户角色
    pub fn new(user_id: impl Into<String>, username: impl Into<String>, role: Role) -> Self {
        Self {
            user_id: user_id.into(),
            username: username.into(),
            role,
            custom_permissions: None,
            expires_at: None,
        }
    }

    /// 检查是否过期
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            expires < chrono::Utc::now()
        } else {
            false
        }
    }

    /// 获取有效权限
    pub fn get_permissions(&self) -> Vec<Permission> {
        let mut perms = get_default_permissions(self.role);

        if let Some(ref custom) = self.custom_permissions {
            perms.extend(custom.clone());
        }

        perms
    }
}

/// 获取角色默认权限
pub fn get_default_permissions(role: Role) -> Vec<Permission> {
    match role {
        Role::Admin => vec![
            Permission::Read,
            Permission::Write,
            Permission::Execute,
            Permission::Admin,
        ],
        Role::Operator => vec![Permission::Read, Permission::Write, Permission::Execute],
        Role::User => vec![Permission::Read, Permission::Write],
        Role::Guest => vec![Permission::Read],
    }
}

/// 权限检查结果
#[derive(Debug, Clone)]
pub struct PermissionCheckResult {
    /// 是否允许
    pub allowed: bool,
    /// 原因
    pub reason: String,
    /// 需要的权限
    pub required_permission: Option<Permission>,
}

impl PermissionCheckResult {
    /// 允许访问
    pub fn allowed() -> Self {
        Self {
            allowed: true,
            reason: "权限检查通过".to_string(),
            required_permission: None,
        }
    }

    /// 拒绝访问
    pub fn denied(reason: impl Into<String>, required: Option<Permission>) -> Self {
        Self {
            allowed: false,
            reason: reason.into(),
            required_permission: required,
        }
    }
}

/// 资源操作定义
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ResourceAction {
    /// 资源类型
    pub resource_type: String,
    /// 操作
    pub action: String,
}

impl ResourceAction {
    /// 创建新的资源操作
    pub fn new(resource_type: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            action: action.into(),
        }
    }
}

/// 权限规则定义
#[derive(Debug, Clone)]
pub struct PermissionRule {
    /// 资源操作
    pub resource_action: ResourceAction,
    /// 所需权限
    pub required_permission: Permission,
}

/// RBAC 服务内部状态
struct RbacServiceInner {
    /// 用户角色映射
    user_roles: HashMap<String, UserRole>,
    /// 权限规则
    rules: Vec<PermissionRule>,
}

/// RBAC 服务
pub struct RbacService {
    inner: parking_lot::RwLock<RbacServiceInner>,
}

impl Default for RbacService {
    fn default() -> Self {
        Self::new()
    }
}

impl RbacService {
    /// 创建新的 RBAC 服务
    pub fn new() -> Self {
        let default_rules = vec![
            PermissionRule {
                resource_action: ResourceAction::new("user", "create"),
                required_permission: Permission::Admin,
            },
            PermissionRule {
                resource_action: ResourceAction::new("user", "delete"),
                required_permission: Permission::Admin,
            },
            PermissionRule {
                resource_action: ResourceAction::new("user", "list"),
                required_permission: Permission::Admin,
            },
            PermissionRule {
                resource_action: ResourceAction::new("command", "execute"),
                required_permission: Permission::Execute,
            },
            PermissionRule {
                resource_action: ResourceAction::new("command", "execute_admin"),
                required_permission: Permission::Admin,
            },
            PermissionRule {
                resource_action: ResourceAction::new("config", "read"),
                required_permission: Permission::Read,
            },
            PermissionRule {
                resource_action: ResourceAction::new("config", "write"),
                required_permission: Permission::Admin,
            },
            PermissionRule {
                resource_action: ResourceAction::new("apikey", "create"),
                required_permission: Permission::Admin,
            },
            PermissionRule {
                resource_action: ResourceAction::new("apikey", "list"),
                required_permission: Permission::Admin,
            },
            PermissionRule {
                resource_action: ResourceAction::new("audit", "read"),
                required_permission: Permission::Admin,
            },
            PermissionRule {
                resource_action: ResourceAction::new("audit", "write"),
                required_permission: Permission::Read,
            },
        ];

        Self {
            inner: parking_lot::RwLock::new(RbacServiceInner {
                user_roles: HashMap::new(),
                rules: default_rules,
            }),
        }
    }

    /// 分配角色给用户
    pub fn assign_role(&self, user_id: &str, role: Role) -> Result<(), RbacError> {
        let mut inner = self.inner.write();
        if let Some(existing) = inner.user_roles.get_mut(user_id) {
            existing.role = role;
        } else {
            inner
                .user_roles
                .insert(user_id.to_string(), UserRole::new(user_id, user_id, role));
        }
        Ok(())
    }

    /// 获取用户角色
    pub fn get_user_role(&self, user_id: &str) -> Option<UserRole> {
        let inner = self.inner.read();
        inner.user_roles.get(user_id).cloned()
    }

    /// 检查用户是否有权限
    pub fn check_permission(
        &self,
        user_id: &str,
        resource_type: &str,
        action: &str,
    ) -> PermissionCheckResult {
        let inner = self.inner.read();

        // 获取用户角色
        let user_role = match inner.user_roles.get(user_id) {
            Some(r) => r,
            None => return PermissionCheckResult::denied("用户不存在", None),
        };

        // 检查是否过期
        if user_role.is_expired() {
            return PermissionCheckResult::denied("用户角色已过期", None);
        }

        // 获取用户权限
        let user_permissions = user_role.get_permissions();

        // 查找所需规则
        let required_permission = inner
            .rules
            .iter()
            .find(|r| {
                r.resource_action.resource_type == resource_type
                    && r.resource_action.action == action
            })
            .map(|r| r.required_permission);

        // 如果没有规则, 默认拒绝 (安全默认值)
        let required = match required_permission {
            Some(p) => p,
            None => return PermissionCheckResult::denied("未定义的资源操作，默认拒绝", None),
        };

        // 检查权限
        if user_permissions.contains(&required) {
            PermissionCheckResult::allowed()
        } else {
            PermissionCheckResult::denied(
                format!("用户 {} 缺少权限: {}", user_role.role, required),
                Some(required),
            )
        }
    }

    /// 检查多个权限 (AND)
    pub fn check_all_permissions(
        &self,
        user_id: &str,
        requirements: Vec<(String, String)>,
    ) -> PermissionCheckResult {
        for (resource_type, action) in requirements {
            let result = self.check_permission(user_id, &resource_type, &action);
            if !result.allowed {
                return result;
            }
        }
        PermissionCheckResult::allowed()
    }

    /// 检查任意权限 (OR)
    pub fn check_any_permission(
        &self,
        user_id: &str,
        requirements: Vec<(String, String)>,
    ) -> PermissionCheckResult {
        for (resource_type, action) in requirements {
            let result = self.check_permission(user_id, &resource_type, &action);
            if result.allowed {
                return result;
            }
        }

        let inner = self.inner.read();
        if let Some(user_role) = inner.user_roles.get(user_id) {
            PermissionCheckResult::denied(
                format!("用户 {} 没有满足要求的权限", user_role.username),
                None,
            )
        } else {
            PermissionCheckResult::denied("用户不存在", None)
        }
    }

    /// 注册权限规则
    pub fn register_rule(&self, rule: PermissionRule) -> Result<(), RbacError> {
        let mut inner = self.inner.write();
        inner.rules.push(rule);
        Ok(())
    }

    /// 获取所有规则
    pub fn get_rules(&self) -> Vec<PermissionRule> {
        let inner = self.inner.read();
        inner.rules.clone()
    }

    /// 移除用户角色
    pub fn remove_user_role(&self, user_id: &str) -> Result<(), RbacError> {
        let mut inner = self.inner.write();
        inner.user_roles.remove(user_id);
        Ok(())
    }

    /// 获取所有用户角色
    pub fn get_all_user_roles(&self) -> HashMap<String, UserRole> {
        let inner = self.inner.read();
        inner.user_roles.clone()
    }
}

/// 权限中间件 trait
pub trait PermissionMiddleware: Send + Sync {
    /// 检查权限
    fn check(&self, user_id: &str, resource: &str, action: &str) -> bool;
}

/// RBAC 中间件实现
pub struct RbacMiddleware {
    rbac_service: std::sync::Arc<RbacService>,
}

impl RbacMiddleware {
    /// 创建新的 RBAC 中间件
    pub fn new(rbac_service: std::sync::Arc<RbacService>) -> Self {
        Self { rbac_service }
    }
}

impl PermissionMiddleware for RbacMiddleware {
    fn check(&self, user_id: &str, resource: &str, action: &str) -> bool {
        self.rbac_service
            .check_permission(user_id, resource, action)
            .allowed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_permissions() {
        assert!(get_default_permissions(Role::Admin).contains(&Permission::Admin));
        assert!(get_default_permissions(Role::Operator).contains(&Permission::Execute));
        assert!(get_default_permissions(Role::User).contains(&Permission::Read));
        assert!(get_default_permissions(Role::Guest).contains(&Permission::Read));
        assert!(!get_default_permissions(Role::Guest).contains(&Permission::Write));
    }

    #[test]
    fn test_assign_role() {
        let rbac = RbacService::new();

        rbac.assign_role("user1", Role::User).unwrap();
        let role = rbac.get_user_role("user1").unwrap();

        assert_eq!(role.role, Role::User);
        assert!(role.get_permissions().contains(&Permission::Read));
    }

    #[test]
    fn test_permission_check() {
        let rbac = RbacService::new();

        rbac.assign_role("admin1", Role::Admin).unwrap();
        rbac.assign_role("user1", Role::User).unwrap();

        // Admin 可以执行命令
        let result = rbac.check_permission("admin1", "command", "execute");
        assert!(result.allowed);

        // User 不可以执行管理命令
        let result = rbac.check_permission("user1", "command", "execute_admin");
        assert!(!result.allowed);

        // Admin 可以管理用户
        let result = rbac.check_permission("admin1", "user", "create");
        assert!(result.allowed);
    }

    #[test]
    fn test_check_all_permissions() {
        let rbac = RbacService::new();

        rbac.assign_role("admin1", Role::Admin).unwrap();

        let result = rbac.check_all_permissions(
            "admin1",
            vec![("command".to_string(), "execute".to_string())],
        );
        assert!(result.allowed);

        let result = rbac.check_all_permissions(
            "admin1",
            vec![
                ("command".to_string(), "execute".to_string()),
                ("user".to_string(), "list".to_string()),
            ],
        );
        assert!(result.allowed);
    }

    #[test]
    fn test_user_role_expiry() {
        let mut user_role = UserRole::new("user1", "test", Role::User);
        user_role.expires_at = Some(chrono::Utc::now() - chrono::Duration::hours(1));

        assert!(user_role.is_expired());
    }

    #[test]
    fn test_rbac_middleware() {
        let rbac = RbacService::new();
        rbac.assign_role("user1", Role::Admin).unwrap();

        let middleware = RbacMiddleware::new(std::sync::Arc::new(rbac));

        assert!(middleware.check("user1", "command", "execute"));
        assert!(middleware.check("user1", "user", "create"));
    }
}
