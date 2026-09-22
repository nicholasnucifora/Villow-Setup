mod common;
use common::*;
use villow_setup::{engine::Engine, error::Error, manager::Manager, model::*, store::Store};

#[test]
fn accepted_build_is_not_sign_in_ready_and_polling_never_redeploys() {
    let d = tempfile::tempdir().unwrap();
    let store = Store::open(d.path()).unwrap();
    let release = verified();
    let mut s = installation(&release);
    s.step = Step::Deployment;
    s.origin = Some(format!("https://{}.vercel.app", s.name));
    store.save(&s).unwrap();
    let vault = MemoryVault::default();
    let provider = Fake::new(d.path().join("remote.json"));
    let engine = Engine {
        store: &store,
        vault: &vault,
        providers: &provider,
    };
    let accepted = engine.advance(&release).unwrap();
    assert_eq!(accepted.step, Step::Deployment);
    assert_eq!(accepted.deployment_status, Some(DeploymentStatus::Queued));
    assert_eq!(provider.read().deploys, 1);
    for status in [
        DeploymentStatus::Queued,
        DeploymentStatus::Building,
        DeploymentStatus::AssigningAddress,
        DeploymentStatus::Failed,
        DeploymentStatus::Canceled,
        DeploymentStatus::AddressFailed,
    ] {
        provider.deployment_status.set(status);
        let state = engine.check_deployment(&release).unwrap();
        assert_eq!(state.step, Step::Deployment);
        assert_eq!(state.deployment_id, accepted.deployment_id);
        assert_eq!(state.deployment_status, Some(status));
        // A duplicate Continue also checks the saved ID instead of POSTing again.
        engine.advance(&release).unwrap();
        assert_eq!(provider.read().deploys, 1);
        assert_eq!(
            Manager::open(d.path()).unwrap().browser_url("app"),
            Err(Error::Precondition)
        );
    }
    provider.deployment_status.set(DeploymentStatus::Ready);
    assert_eq!(
        engine.check_deployment(&release).unwrap().step,
        Step::Health
    );
    assert_eq!(
        Manager::open(d.path()).unwrap().browser_url("app").unwrap(),
        s.origin.unwrap()
    );
    assert_eq!(provider.read().deploys, 1);
    // A previously ready build can stop being ready before the final app check.
    provider.deployment_status.set(DeploymentStatus::Canceled);
    assert_eq!(engine.advance(&release).unwrap().step, Step::Deployment);
    assert_eq!(
        Manager::open(d.path()).unwrap().browser_url("app"),
        Err(Error::Precondition)
    );
}

#[test]
fn older_health_checkpoint_requires_a_fresh_build_check_and_read_only_cannot_poll() {
    let d = tempfile::tempdir().unwrap();
    let store = Store::open(d.path()).unwrap();
    let release = verified();
    let mut s = installation(&release);
    s.step = Step::Health;
    s.deployment_id = Some("dpl_saved".into());
    s.origin = Some(format!("https://{}.vercel.app", s.name));
    let mut old = serde_json::to_value(&s).unwrap();
    old.as_object_mut().unwrap().remove("deployment_status");
    s = serde_json::from_value(old).unwrap();
    store.save(&s).unwrap();
    assert_eq!(
        Manager::open(d.path()).unwrap().browser_url("app"),
        Err(Error::Precondition)
    );
    let vault = MemoryVault::default();
    let provider = Fake::new(d.path().join("remote.json"));
    provider.deployment_status.set(DeploymentStatus::Building);
    let engine = Engine {
        store: &store,
        vault: &vault,
        providers: &provider,
    };
    let mut resumed = engine.check_deployment(&release).unwrap();
    assert_eq!(resumed.step, Step::Deployment);
    assert_eq!(resumed.deployment_id.as_deref(), Some("dpl_saved"));
    assert_eq!(provider.read().deploys, 0);
    resumed.read_only = true;
    store.save(&resumed).unwrap();
    assert_eq!(
        engine.check_deployment(&release).unwrap_err(),
        Error::RecoveryReadOnly
    );
    resumed.read_only = false;
    resumed.step = Step::Database;
    store.save(&resumed).unwrap();
    assert_eq!(
        engine.check_deployment(&release).unwrap_err(),
        Error::Precondition
    );
}
