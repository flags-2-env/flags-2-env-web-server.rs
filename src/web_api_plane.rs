//! Four-avenue web ↔ API binding for `flags-2-env`.
//!
//! 1. Direct read-only DB via `*-lib-core` named queries (no migrations).
//! 2. Stateless HTTP from `app.flags-2-env.dev` to `api.flags-2-env.dev`.
//! 3. Stateful TLS 1.3/mTLS TCP to `api.flags-2-env.dev:7443`.
//! 4. JetStream: in-cluster producers publish directly to
//!    `nats://dd-nats.messaging.svc.cluster.local:4222`. External producers
//!    use named HTTPS routes on `dd-nats-bridge` (not raw subjects). The
//!    `dd-remote-queue-consumer` in k8s-cluster is the agent-task consumer,
//!    not the product-web producer path.

use k8s_web_api_data_plane::{
    DataPlaneCapabilities, DataPlaneError, DirectDatabasePolicy, InteractionMode, JetStreamPolicy,
    OrgIdentity, StatefulMtlsTcpPolicy, StatelessHttpPolicy,
};

pub const GITHUB_ORG: &str = "flags-2-env";
pub const ORG_SLUG: &str = "flags-2-env";
pub const DNS_ZONE: &str = "flags-2-env.dev";

type Policies = (
    DirectDatabasePolicy,
    StatelessHttpPolicy,
    StatefulMtlsTcpPolicy,
    JetStreamPolicy,
);

pub fn identity() -> Result<OrgIdentity, DataPlaneError> {
    OrgIdentity::new(GITHUB_ORG, ORG_SLUG, DNS_ZONE)
}

pub fn capabilities() -> Result<DataPlaneCapabilities, DataPlaneError> {
    identity().map(|identity| DataPlaneCapabilities::for_identity(&identity))
}

fn policies_for(identity: &OrgIdentity) -> Policies {
    (
        DirectDatabasePolicy::for_identity(identity),
        StatelessHttpPolicy::for_identity(identity),
        StatefulMtlsTcpPolicy::for_identity(identity, 7443),
        JetStreamPolicy::for_identity(identity),
    )
}

pub fn policies() -> Result<Policies, DataPlaneError> {
    identity().map(|identity| policies_for(&identity))
}

pub fn validate_four_avenues() -> Result<(), DataPlaneError> {
    let identity = identity()?;
    let (db, http, tcp, nats) = policies_for(&identity);
    db.validate(&identity)?;
    http.validate()?;
    tcp.validate()?;
    nats.validate()?;
    debug_assert_eq!(InteractionMode::ALL.len(), 4);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_avenues_are_named_and_fail_closed() -> Result<(), DataPlaneError> {
        validate_four_avenues()?;
        let caps = capabilities()?;
        assert_eq!(caps.app_host, format!("app.{DNS_ZONE}"));
        assert_eq!(caps.api_host, format!("api.{DNS_ZONE}"));
        assert_eq!(
            caps.nats_request_subject,
            format!("dd.remote.web_api.{ORG_SLUG}.request")
        );
        assert_eq!(
            caps.nats_url_in_cluster,
            "nats://dd-nats.messaging.svc.cluster.local:4222"
        );
        Ok(())
    }
}
