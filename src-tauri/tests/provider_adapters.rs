mod common;
use common::*;
use reqwest::Method;
use serde_json::{json, Value};
use std::{cell::RefCell, collections::VecDeque};
use villow_setup::{
    engine::Providers,
    error::{Error, Result},
    http::{Api, Provider},
    model::*,
    providers::LiveProviders,
    vault::Vault,
};

struct Recorder {
    responses: RefCell<VecDeque<Result<Value>>>,
    calls: RefCell<
        Vec<(
            Provider,
            Method,
            String,
            Vec<(String, String)>,
            Option<Value>,
        )>,
    >,
}
impl Recorder {
    fn new(responses: Vec<Value>) -> Self {
        Self {
            responses: RefCell::new(responses.into_iter().map(Ok).collect()),
            calls: RefCell::new(vec![]),
        }
    }
}
impl Api for Recorder {
    fn provider(
        &self,
        p: Provider,
        m: Method,
        path: &str,
        q: &[(&str, &str)],
        token: &str,
        b: Option<&Value>,
    ) -> Result<Value> {
        assert!(token.starts_with("SENTINEL-"));
        self.calls.borrow_mut().push((
            p,
            m,
            path.into(),
            q.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            b.cloned(),
        ));
        self.responses
            .borrow_mut()
            .pop_front()
            .ok_or(Error::Provider)?
    }
    fn upload(&self, _t: &str, _a: &str, _b: &[u8]) -> Result<String> {
        Ok("a".repeat(40))
    }
    fn health(&self, _o: &str, _t: &str, _n: &str) -> Result<Value> {
        Err(Error::Health)
    }
}
fn account_responses() -> Vec<Value> {
    vec![
        json!({"user":{"id":"u1"}}),
        json!({"teams":[{"id":"team_1","name":"Hosting","slug":"hosting"}],"pagination":{"next":null}}),
        json!({"gotrue_id":"s1","primary_email":"owner@example.test"}),
        json!([{"id":"org1","name":"Database","slug":"org1"}]),
    ]
}
fn credentials(s: &Installation) -> MemoryVault {
    let v = MemoryVault::default();
    v.put(&s.id, "vercel_token", "SENTINEL-VERCEL").unwrap();
    v.put(&s.id, "supabase_token", "SENTINEL-SUPABASE").unwrap();
    v.put(&s.id, "db_password", "SENTINEL-DATABASE").unwrap();
    v
}

#[test]
fn account_access_refusals_identify_the_exact_check_without_exposing_secrets() {
    let s = installation(&verified());
    let v = credentials(&s);
    for (index, expected, provider, path) in [
        (0, Error::VercelIdentityAccess, "Vercel", "/v2/user"),
        (1, Error::VercelTeamsAccess, "Vercel", "/v2/teams"),
        (2, Error::SupabaseIdentityAccess, "Supabase", "/v1/profile"),
        (
            3,
            Error::SupabaseOrganizationsAccess,
            "Supabase",
            "/v1/organizations",
        ),
    ] {
        let api = Recorder::new(account_responses());
        api.responses.borrow_mut()[index] = Err(Error::Authentication);
        let error = LiveProviders {
            http: &api,
            vault: &v,
        }
        .accounts(&s)
        .unwrap_err();
        assert_eq!(error, expected);
        let message = error.to_string();
        assert!(message.starts_with(provider));
        assert!(message.contains(path));
        assert!(!message.contains("SENTINEL"));
        assert!(!serde_json::to_string(&error).unwrap().contains("SENTINEL"));
        let calls = api.calls.borrow();
        assert_eq!(calls.len(), index + 1);
        assert!(calls
            .iter()
            .all(|call| call.1 == Method::GET && call.4.is_none()));
    }
}

#[test]
fn account_access_context_does_not_mislabel_other_errors_as_bad_tokens() {
    let s = installation(&verified());
    let v = credentials(&s);
    for failure in [
        Error::Offline,
        Error::RateLimited,
        Error::WrongTarget,
        Error::Provider,
        Error::Uncertain,
    ] {
        for index in 0..4 {
            let api = Recorder::new(account_responses());
            api.responses.borrow_mut()[index] = Err(failure.clone());
            assert_eq!(
                LiveProviders {
                    http: &api,
                    vault: &v
                }
                .accounts(&s)
                .unwrap_err(),
                failure
            );
        }
    }
}
#[test]
fn wrong_principal_is_rejected_using_real_adapter_response_shapes() {
    let s = installation(&verified());
    let v = credentials(&s);
    let mut responses = account_responses();
    responses[0] = json!({"user":{"id":"different-user"}});
    let api = Recorder::new(responses);
    let p = LiveProviders {
        http: &api,
        vault: &v,
    };
    assert_eq!(p.verify_targets(&s), Err(Error::WrongTarget));
    assert!(api.calls.borrow().iter().all(|c| c.1 == Method::GET))
}
#[test]
fn wrong_project_account_never_reaches_environment_write() {
    let mut s = installation(&verified());
    s.vercel = Some(Resource {
        id: "prj_expected".into(),
        account_id: "team_1".into(),
        name: s.name.clone(),
        operation_id: s.operation_id.clone(),
        evidence: "created".into(),
    });
    let v = credentials(&s);
    let mut responses = account_responses();
    responses.push(json!({"id":"prj_expected","accountId":"other-team","name":s.name}));
    let api = Recorder::new(responses);
    assert_eq!(
        LiveProviders {
            http: &api,
            vault: &v
        }
        .verify_targets(&s),
        Err(Error::WrongTarget)
    );
    assert!(api.calls.borrow().iter().all(|c| c.1 == Method::GET))
}
#[test]
fn creation_pins_team_framework_and_does_not_link_git() {
    let s = installation(&verified());
    let v = credentials(&s);
    let response = json!({"id":"prj_created","accountId":"team_1","name":s.name});
    let api = Recorder::new(vec![response.clone(), response]);
    let r = LiveProviders {
        http: &api,
        vault: &v,
    }
    .create_vercel(&s)
    .unwrap();
    assert_eq!(r.id, "prj_created");
    let c = api.calls.borrow();
    assert_eq!(c[0].0, Provider::Vercel);
    assert_eq!(c[0].2, "/v10/projects");
    assert_eq!(c[0].3, vec![("teamId".into(), "team_1".into())]);
    let body = c[0].4.as_ref().unwrap();
    assert_eq!(body["framework"], "vite");
    assert_eq!(body["installCommand"], "npm ci");
    assert!(body.get("gitRepository").is_none())
}
#[test]
fn supabase_creation_uses_org_slug_and_no_plan_or_paid_compute() {
    let s = installation(&verified());
    let v = credentials(&s);
    let api = Recorder::new(vec![
        json!({"id":"provider-project-id","ref":"abcdefghijklmnopqrst","name":s.name,"organization_id":"org1"}),
    ]);
    let r = LiveProviders {
        http: &api,
        vault: &v,
    }
    .create_database(&s)
    .unwrap();
    assert_eq!(r.id, "abcdefghijklmnopqrst");
    let c = api.calls.borrow();
    let body = c[0].4.as_ref().unwrap();
    assert_eq!(body["organization_slug"], "org1");
    assert_eq!(body["db_pass"], "SENTINEL-DATABASE");
    assert!(body.get("plan").is_none());
    assert!(body.get("desired_instance_size").is_none())
}
#[test]
fn uncertain_live_creation_does_not_adopt_name_only_or_repeat_post() {
    let s = installation(&verified());
    let v = credentials(&s);
    let api = Recorder::new(vec![
        json!({"id":"prj_found","accountId":"team_1","name":s.name}),
    ]);
    assert!(LiveProviders {
        http: &api,
        vault: &v
    }
    .reconcile_vercel(&s)
    .unwrap()
    .is_none());
    assert_eq!(api.calls.borrow().len(), 1);
    assert_eq!(api.calls.borrow()[0].1, Method::GET);
}

#[test]
fn changed_supabase_principal_with_same_organization_is_rejected() {
    let s = installation(&verified());
    let v = credentials(&s);
    let mut responses = account_responses();
    responses[2] = json!({"gotrue_id":"other-owner"});
    let api = Recorder::new(responses);
    assert_eq!(
        LiveProviders {
            http: &api,
            vault: &v
        }
        .verify_targets(&s),
        Err(Error::WrongTarget)
    );
    assert!(api.calls.borrow().iter().all(|c| c.1 == Method::GET));
}
