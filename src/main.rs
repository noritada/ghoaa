use anyhow::*;
use clap::Clap;
use graphql_client::{GraphQLQuery, Response};
use serde::*;
use std::collections::BTreeMap;
use std::io::Write;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema.graphql",
    query_path = "src/query.graphql",
    response_derives = "Debug"
)]
struct MembersView;

#[derive(Clap)]
#[clap()]
struct Opts {
    #[clap(short, long, default_value = "")]
    cache_file: String,
    #[clap(name = "ORGANIZATION")]
    org: String,
    #[clap(name = "OUT_FILE")]
    out_csv_file: String,
}

#[derive(Deserialize, Debug)]
struct Env {
    github_access_token: String,
}

fn main() -> std::result::Result<(), anyhow::Error> {
    let config: Env = envy::from_env().context("Failed to read necessary environment values")?;

    let opts = Opts::parse();
    let q = MembersView::build_query(members_view::Variables {
        organization: opts.org,
        members_cursor: None,
        saml_id_provider_cursor: None,
    });

    let client = reqwest::Client::new();
    let mut resp = client
        .post("https://api.github.com/graphql")
        .bearer_auth(config.github_access_token)
        .json(&q)
        .send()?;

    let resp_text = resp.text()?;
    let status_code = resp.status();
    if status_code.is_client_error() || status_code.is_server_error() {
        bail!(
            "Failed to get a successful data:\n    status code: {}\n    body: {}",
            status_code,
            resp_text
        );
    }

    if opts.cache_file.len() > 0 {
        let mut cache_file = std::fs::File::create(opts.cache_file)?;
        cache_file.write_all(resp_text.as_ref())?;
    }

    let json_root: Response<members_view::ResponseData> = serde_json::from_str(&resp_text)?;
    if let Some(errors) = json_root.errors {
        let messages = errors
            .iter()
            .map(|e| e.message.clone())
            .collect::<Vec<_>>()
            .join("\n    ");

        bail!("Resulted in output with errors:\n    {}", messages);
    }

    let organization = json_root
        .data
        .and_then(|d| d.organization)
        .expect("organization info not found in response");

    let saml_identities = organization
        .saml_identity_provider
        .and_then(|p| p.external_identities.edges)
        .expect("SAML identity list not found in response");

    let mut map = BTreeMap::new();

    for identity in saml_identities {
        if let Some(node) = identity.and_then(|i| i.node) {
            if node.user.is_none() {
                continue;
            }
            let user = node.user.unwrap();

            let saml_name_id = node.saml_identity.and_then(|i| i.name_id);
            map.insert(user.id, saml_name_id);
        }
    }

    let members = organization
        .members_with_role
        .edges
        .expect("members list not fouond in response");

    let mut writer = csv::Writer::from_path(opts.out_csv_file)?;
    writer.write_record(&[
        "id",
        "database_id",
        "login",
        "name",
        "role",
        "has_two_factor_enabled",
        "saml_name_id",
    ])?;

    for member in members {
        if let Some(member) = member {
            if member.node.is_none() {
                continue;
            }
            let node = member.node.unwrap();

            let saml_name_id = map.get(&node.id);

            writer.serialize((
                node.id,
                node.database_id,
                node.login,
                node.name,
                member.role,
                member.has_two_factor_enabled,
                saml_name_id,
            ))?;
        }
    }
    writer.flush()?;

    Ok(())
}
