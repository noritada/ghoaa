use anyhow::*;
use graphql_client::{GraphQLQuery, Response};
use std::collections::BTreeMap;
use std::io::Write;

use crate::common::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema.graphql",
    query_path = "src/members_query.graphql",
    response_derives = "Debug"
)]
struct MembersView;

fn query(
    config: &Config,
    members_cursor: Option<String>,
    ext_ids_cursor: Option<String>,
    iter_num: u8,
) -> std::result::Result<Response<members_view::ResponseData>, anyhow::Error> {
    let q = members_view::Variables {
        organization: config.org.clone(),
        members_cursor,
        ext_ids_cursor,
    };

    print_progress(Progress::Downloading)?;
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post("https://api.github.com/graphql")
        .bearer_auth(&config.github_access_token)
        .json(&q)
        .send()?;

    resp.error_for_status_ref()?;
    print_progress(Progress::Downloaded)?;

    let resp_text = resp.text()?;
    if let Some(cache_file_prefix) = &config.cache_file_prefix {
        let cache_file_path = format!("{}.{:02}", cache_file_prefix, iter_num);
        let mut cache_file = std::fs::File::create(cache_file_path)?;
        cache_file.write_all(resp_text.as_ref())?;
    }

    let json_root: Response<members_view::ResponseData> = serde_json::from_str(&resp_text)?;

    Ok(json_root)
}

fn extract(
    json_root: Response<members_view::ResponseData>,
) -> std::result::Result<
    (
        Vec<Option<members_view::MembersViewOrganizationMembersWithRoleEdges>>,
        members_view::MembersViewOrganizationMembersWithRolePageInfo,
        Vec<
            Option<
                members_view::MembersViewOrganizationSamlIdentityProviderExternalIdentitiesEdges,
            >,
        >,
        members_view::MembersViewOrganizationSamlIdentityProviderExternalIdentitiesPageInfo,
    ),
    anyhow::Error,
> {
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
        .ok_or_else(|| anyhow!("organization info not found"))?;

    let members = organization
        .members_with_role
        .edges
        .ok_or_else(|| anyhow!("members list not found"))?;

    let members_page_info = organization.members_with_role.page_info;

    let ext_ids_root = organization
        .saml_identity_provider
        .map(|p| p.external_identities)
        .ok_or_else(|| anyhow!("external identity info not found"))?;

    let ext_ids = ext_ids_root
        .edges
        .ok_or_else(|| anyhow!("SAML identity list not found"))?;

    let ext_ids_page_info = ext_ids_root.page_info;

    Ok((members, members_page_info, ext_ids, ext_ids_page_info))
}

pub(crate) fn process(config: &Config) -> std::result::Result<(), anyhow::Error> {
    let mut members_list = Vec::new();
    let mut ext_ids_list = Vec::new();
    let mut members_cursor = None;
    let mut ext_ids_cursor = None;
    let mut num = 0;

    let local: chrono::prelude::DateTime<chrono::prelude::Local> = chrono::prelude::Local::now();
    let out_fname = local.format(&config.out_csv_file).to_string();

    loop {
        let json_root = query(config, members_cursor, ext_ids_cursor, num)?;
        let (members, members_page_info, ext_ids, ext_ids_page_info) = extract(json_root)?;
        members_list.push(members);
        ext_ids_list.push(ext_ids);

        if !members_page_info.has_next_page && !ext_ids_page_info.has_next_page {
            break;
        }

        members_cursor = members_page_info.end_cursor;
        ext_ids_cursor = ext_ids_page_info.end_cursor;
        num += 1;
    }

    let mut map = BTreeMap::new();

    for ext_ids in ext_ids_list {
        for identity in ext_ids {
            if let Some(node) = identity.and_then(|i| i.node) {
                if node.user.is_none() {
                    continue;
                }
                let user = node.user.unwrap();

                let saml_name_id = node.saml_identity.and_then(|i| i.name_id);
                map.insert(user.id, saml_name_id);
            }
        }
    }

    let mut writer = csv::Writer::from_path(out_fname)?;
    writer.write_record(&[
        "id",
        "database_id",
        "login",
        "name",
        "role",
        "has_two_factor_enabled",
        "saml_name_id",
    ])?;

    for members in members_list {
        for member in members.into_iter().flatten() {
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
