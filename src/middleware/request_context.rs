use std::sync::Arc;

use actix_web::body::MessageBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::error::ErrorInternalServerError;
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{Error, HttpMessage, web};
use futures::future::{LocalBoxFuture, Ready, ok};
use log::error;
use serde_json::{Value as JsonValue, json};
use uuid::Uuid;

use crate::core::contracts::repository::audit_logs::AuditLogRepository;
use crate::core::entities::audit_logs::NewAuditLogEntry;
use crate::core::entities::auth::UserClaims;

const REQUEST_ID_HEADER: &str = "x-request-id";

#[derive(Clone, Debug)]
pub struct RequestContext {
    pub request_id: Uuid,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Clone)]
pub struct RequestContextAuditMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestContextAuditMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestContextAuditMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequestContextAuditMiddlewareService { service })
    }
}

pub struct RequestContextAuditMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestContextAuditMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let request_id = request_id_from_header(&req).unwrap_or_else(Uuid::new_v4);
        let context = RequestContext {
            request_id,
            ip_address: req
                .connection_info()
                .realip_remote_addr()
                .map(|value| value.to_string()),
            user_agent: req
                .headers()
                .get("User-Agent")
                .and_then(|value| value.to_str().ok())
                .map(|value| value.to_string()),
        };
        req.extensions_mut().insert(context);

        let fut = self.service.call(req);
        Box::pin(async move {
            let mut response = fut.await?;
            response.headers_mut().insert(
                HeaderName::from_static(REQUEST_ID_HEADER),
                HeaderValue::from_str(&request_id.to_string())
                    .map_err(ErrorInternalServerError)?,
            );

            if let Some(entry) = build_audit_entry(&response) {
                if let Some(repo) = response
                    .request()
                    .app_data::<web::Data<Arc<dyn AuditLogRepository>>>()
                {
                    if let Err(err) = repo.create_audit_log(entry.clone()).await {
                        error!("[Audit] Failed to persist audit log: {:?}", err);
                        if entry.details["strict"].as_bool().unwrap_or(false) {
                            return Err(ErrorInternalServerError("audit log failure"));
                        }
                    }
                } else if entry.details["strict"].as_bool().unwrap_or(false) {
                    error!("[Audit] Audit repository missing from app data");
                    return Err(ErrorInternalServerError("audit log unavailable"));
                }
            }

            Ok(response)
        })
    }
}

fn request_id_from_header(req: &ServiceRequest) -> Option<Uuid> {
    req.headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
}

fn build_audit_entry<B>(response: &ServiceResponse<B>) -> Option<NewAuditLogEntry> {
    let req = response.request();
    let method = req.method().as_str();
    let pattern = req.match_pattern().unwrap_or_else(|| req.path().to_string());
    let status = response.status().as_u16();
    let success = response.status().is_success();

    let spec = audit_spec(method, &pattern, success)?;
    let context = req.extensions().get::<RequestContext>().cloned()?;
    let claims = req.extensions().get::<UserClaims>().cloned();
    let user_id = claims
        .as_ref()
        .and_then(|claims| Uuid::parse_str(&claims.id).ok());
    let city_id = claims
        .as_ref()
        .and_then(|claims| claims.city_id.as_deref())
        .and_then(|city_id| Uuid::parse_str(city_id).ok());
    let entity_id = spec
        .entity_param
        .and_then(|param| req.match_info().get(param))
        .and_then(|value| Uuid::parse_str(value).ok());

    let mut details = spec.details;
    details["status"] = json!(status);
    details["success"] = json!(success);
    details["strict"] = json!(spec.strict);

    Some(NewAuditLogEntry {
        request_id: context.request_id,
        user_id,
        action: spec.action.to_string(),
        entity_type: spec.entity_type.to_string(),
        entity_id,
        city_id,
        ip_address: context.ip_address,
        user_agent: context.user_agent,
        details,
    })
}

struct AuditSpec {
    action: &'static str,
    entity_type: &'static str,
    entity_param: Option<&'static str>,
    strict: bool,
    details: JsonValue,
}

fn audit_spec(method: &str, pattern: &str, success: bool) -> Option<AuditSpec> {
    let auth_status = if success { "success" } else { "failure" };
    match (method, pattern) {
        ("POST", "/api/v1/auth/login") => {
            return Some(best_effort(
                if success {
                    "auth.login.success"
                } else {
                    "auth.login.failure"
                },
                "auth",
                None,
                json!({ "result": auth_status }),
            ));
        }
        ("POST", "/api/v1/auth/refresh") => {
            return Some(best_effort(
                if success {
                    "auth.refresh.success"
                } else {
                    "auth.refresh.failure"
                },
                "auth",
                None,
                json!({ "result": auth_status }),
            ));
        }
        ("POST", "/api/v1/auth/logout") => {
            return Some(best_effort(
                if success {
                    "auth.logout.success"
                } else {
                    "auth.logout.failure"
                },
                "auth",
                None,
                json!({ "result": auth_status }),
            ));
        }
        _ => {}
    }

    if !success {
        return None;
    }

    match (method, pattern) {
        ("PATCH", "/api/v1/users/password") => {
            Some(strict("user.password.change", "user", None, json!({})))
        }
        ("POST", "/api/v1/users/{id}/password/reset") => {
            Some(strict("user.password.reset", "user", Some("id"), json!({})))
        }
        ("POST", "/api/v1/users/{id}/policies/{policy}/cities") => Some(strict(
            "user.policy_cities.append",
            "user",
            Some("id"),
            json!({ "policy_path_param": "policy" }),
        )),
        ("DELETE", "/api/v1/users/{id}/policies/{policy}/cities") => Some(strict(
            "user.policy_cities.remove",
            "user",
            Some("id"),
            json!({ "policy_path_param": "policy" }),
        )),
        _ if pattern.starts_with("/api/v1/victims") => {
            sensitive_resource_spec(method, pattern, "victim")
        }
        _ if pattern.starts_with("/api/v1/offenders") => {
            sensitive_resource_spec(method, pattern, "offender")
        }
        _ if pattern.starts_with("/api/v1/protective-measures") => {
            sensitive_resource_spec(method, pattern, "protective_measure")
        }
        _ if pattern.starts_with("/api/v1/attendance-victims") => {
            sensitive_resource_spec(method, pattern, "attendance_victim")
        }
        _ if pattern.starts_with("/api/v1/attendance-offenders") => {
            sensitive_resource_spec(method, pattern, "attendance_offender")
        }
        _ => None,
    }
}

fn sensitive_resource_spec(method: &str, pattern: &str, entity_type: &'static str) -> Option<AuditSpec> {
    if pattern.ends_with("/search")
        || pattern.contains("/by-")
        || (method == "GET" && !pattern.contains("{id}"))
    {
        return None;
    }

    let (action, entity_param) = match method {
        "GET" => (resource_action(entity_type, "read")?, Some("id")),
        "POST" => (resource_action(entity_type, "create")?, None),
        "PUT" | "PATCH" => (resource_action(entity_type, "update")?, Some("id")),
        "DELETE" => (resource_action(entity_type, "delete")?, Some("id")),
        _ => return None,
    };

    if pattern.contains("/phones/{phone_id}") {
        return Some(strict(
            match method {
                "PUT" | "PATCH" => "phone.update",
                "DELETE" => "phone.delete",
                _ => return None,
            },
            "phone",
            Some("phone_id"),
            json!({ "parent_entity": entity_type }),
        ));
    }

    if pattern.contains("/phones") {
        return Some(strict(
            "phone.create",
            "phone",
            None,
            json!({ "parent_entity": entity_type }),
        ));
    }

    if pattern.contains("/addresses/{address_id}") {
        return Some(strict(
            match method {
                "PUT" | "PATCH" => "address.update",
                "DELETE" => "address.delete",
                _ => return None,
            },
            "address",
            Some("address_id"),
            json!({ "parent_entity": entity_type }),
        ));
    }

    if pattern.contains("/addresses") {
        return Some(strict(
            "address.create",
            "address",
            None,
            json!({ "parent_entity": entity_type }),
        ));
    }

    if pattern.contains("/members/{user_id}") {
        return Some(strict(
            "attendance_member.remove",
            "attendance_member",
            Some("user_id"),
            json!({ "parent_entity": entity_type }),
        ));
    }

    if pattern.contains("/members") {
        return Some(strict(
            match method {
                "GET" => "attendance_member.read",
                "POST" => "attendance_member.add",
                _ => return None,
            },
            "attendance_member",
            None,
            json!({ "parent_entity": entity_type }),
        ));
    }

    Some(strict(action, entity_type, entity_param, json!({})))
}

fn resource_action(entity_type: &str, operation: &str) -> Option<&'static str> {
    match (entity_type, operation) {
        ("victim", "read") => Some("victim.read"),
        ("victim", "create") => Some("victim.create"),
        ("victim", "update") => Some("victim.update"),
        ("victim", "delete") => Some("victim.delete"),
        ("offender", "read") => Some("offender.read"),
        ("offender", "create") => Some("offender.create"),
        ("offender", "update") => Some("offender.update"),
        ("offender", "delete") => Some("offender.delete"),
        ("protective_measure", "read") => Some("protective_measure.read"),
        ("protective_measure", "create") => Some("protective_measure.create"),
        ("protective_measure", "update") => Some("protective_measure.update"),
        ("protective_measure", "delete") => Some("protective_measure.delete"),
        ("attendance_victim", "read") => Some("attendance_victim.read"),
        ("attendance_victim", "create") => Some("attendance_victim.create"),
        ("attendance_victim", "update") => Some("attendance_victim.update"),
        ("attendance_victim", "delete") => Some("attendance_victim.delete"),
        ("attendance_offender", "read") => Some("attendance_offender.read"),
        ("attendance_offender", "create") => Some("attendance_offender.create"),
        ("attendance_offender", "update") => Some("attendance_offender.update"),
        ("attendance_offender", "delete") => Some("attendance_offender.delete"),
        _ => None,
    }
}

fn strict(
    action: &'static str,
    entity_type: &'static str,
    entity_param: Option<&'static str>,
    details: JsonValue,
) -> AuditSpec {
    AuditSpec {
        action,
        entity_type,
        entity_param,
        strict: true,
        details,
    }
}

fn best_effort(
    action: &'static str,
    entity_type: &'static str,
    entity_param: Option<&'static str>,
    details: JsonValue,
) -> AuditSpec {
    AuditSpec {
        action,
        entity_type,
        entity_param,
        strict: false,
        details,
    }
}
