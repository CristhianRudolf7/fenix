//! Opt-in, billable connectivity check for an explicitly selected provider/model.
//! See the provider documentation before running ignored tests.

use fenix_core::{ChatRequest, Message, ProviderId};
use fenix_credentials::{Credential, CredentialStore};
use fenix_providers::{ProviderClient, ProviderConfig, catalog::descriptor};

#[tokio::test]
#[ignore = "requires an explicit provider, model, and credential; sends one billable request"]
async fn explicitly_selected_live_provider_completes_one_request() {
    let provider: ProviderId = std::env::var("FENIX_LIVE_PROVIDER")
        .expect("set FENIX_LIVE_PROVIDER explicitly")
        .parse()
        .expect("FENIX_LIVE_PROVIDER must name a supported provider");
    let model = std::env::var("FENIX_LIVE_MODEL").expect("set FENIX_LIVE_MODEL explicitly");
    assert!(
        !model.trim().is_empty(),
        "FENIX_LIVE_MODEL must not be empty"
    );
    let mut config = ProviderConfig::new(provider);
    if let Ok(base_url) = std::env::var("FENIX_LIVE_BASE_URL") {
        config = config.with_base_url(base_url);
    }
    let mut credentials = CredentialStore::default();
    let variable = descriptor(provider).api_key_env;
    let project = std::env::var("FENIX_LIVE_PROJECT").ok();
    if provider != ProviderId::Ollama || std::env::var_os(variable).is_some() {
        let mut credential = Credential::environment(variable).expect("valid credential variable");
        credential.provider_project = std::env::var("FENIX_LIVE_PROVIDER_PROJECT").ok();
        credential.organization = std::env::var("FENIX_LIVE_ORGANIZATION").ok();
        if let Some(project) = project.as_deref() {
            credentials
                .set_project(project, provider, credential)
                .expect("valid internal project scope");
        } else {
            credentials.set_default(provider, credential);
        }
    }
    let client = ProviderClient::new(credentials).expect("valid client");
    let mut request = ChatRequest::new(model, vec![Message::user("Reply with the word OK.")]);
    request.max_output_tokens = match std::env::var("FENIX_LIVE_MAX_OUTPUT_TOKENS") {
        Ok(value) => value
            .parse()
            .expect("FENIX_LIVE_MAX_OUTPUT_TOKENS must be a positive integer"),
        Err(std::env::VarError::NotPresent) => 1024,
        Err(_) => panic!("FENIX_LIVE_MAX_OUTPUT_TOKENS must be valid text"),
    };
    assert!(
        request.max_output_tokens > 0,
        "FENIX_LIVE_MAX_OUTPUT_TOKENS must be positive"
    );
    let response = client
        .complete(&config, project.as_deref(), &request)
        .await
        .expect("live provider completion");
    assert!(
        !response.message.text_content().trim().is_empty(),
        "provider must return visible text"
    );
}
