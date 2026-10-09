use std::net::{IpAddr, SocketAddr};

use actix_governor::governor::middleware::NoOpMiddleware;
use actix_governor::{GovernorConfig, GovernorConfigBuilder, KeyExtractor, SimpleKeyExtractionError};
use actix_web::dev::ServiceRequest;

/// Governor configuration applied to the `/auth` scope (login/refresh/logout).
pub type AuthRateLimiterConfig = GovernorConfig<RealIpKeyExtractor, NoOpMiddleware>;

/// Extracts the client IP for rate limiting. Honors proxy headers
/// (X-Forwarded-For / Forwarded, set by Traefik in production) via
/// `realip_remote_addr`, falling back to the TCP peer address when the API is
/// reached directly.
#[derive(Clone)]
pub struct RealIpKeyExtractor;

impl KeyExtractor for RealIpKeyExtractor {
    type Key = IpAddr;
    type KeyExtractionError = SimpleKeyExtractionError<&'static str>;

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        let info = req.connection_info();
        let realip = info.realip_remote_addr().ok_or_else(|| {
            SimpleKeyExtractionError::new("Could not extract client IP for rate limiting")
        })?;

        realip
            .parse::<IpAddr>()
            .or_else(|_| realip.parse::<SocketAddr>().map(|addr| addr.ip()))
            .map_err(|_| SimpleKeyExtractionError::new("Could not parse client IP for rate limiting"))
    }
}

/// Builds the rate limiter configuration for the auth endpoints.
/// Returns `None` when `requests_per_minute` is 0 (rate limiting disabled,
/// e.g. in tests).
pub fn build_auth_rate_limiter(requests_per_minute: u64) -> Option<AuthRateLimiterConfig> {
    if requests_per_minute == 0 {
        return None;
    }

    GovernorConfigBuilder::default()
        .requests_per_minute(requests_per_minute)
        .burst_size(requests_per_minute.min(u32::MAX as u64) as u32)
        .key_extractor(RealIpKeyExtractor)
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;

    #[test]
    fn build_auth_rate_limiter_disabled_when_zero() {
        assert!(build_auth_rate_limiter(0).is_none());
    }

    #[test]
    fn build_auth_rate_limiter_enabled_when_positive() {
        assert!(build_auth_rate_limiter(5).is_some());
    }

    #[test]
    fn real_ip_extractor_uses_forwarded_header() {
        let req = TestRequest::default()
            .insert_header(("X-Forwarded-For", "203.0.113.10"))
            .to_srv_request();
        let key = RealIpKeyExtractor.extract(&req).unwrap();
        assert_eq!(key, "203.0.113.10".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn real_ip_extractor_falls_back_to_peer_addr() {
        let req = TestRequest::default()
            .peer_addr("198.51.100.7:45000".parse().unwrap())
            .to_srv_request();
        let key = RealIpKeyExtractor.extract(&req).unwrap();
        assert_eq!(key, "198.51.100.7".parse::<IpAddr>().unwrap());
    }
}
