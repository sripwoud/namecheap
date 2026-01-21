use serde_json::{ Value, json };
use std::error::Error;
use tracing::{ info, error };

// crate imports
use crate::{ NameCheapClient };
use crate::utils::request_builder::Request;
use crate::utils::xml_parser::parse_xml_to_json;

/// Represents the parameters required for setting DNS host records.
#[derive(Debug, Clone)]
pub struct HostRequest {
    pub host_name: String,
    pub record_type: String,
    pub address: String,
    pub mx_pref: Option<String>,
    pub email_type: Option<String>,
    pub ttl: Option<String>,
    pub flag: Option<String>,
    pub tag: Option<String>,
}

impl HostRequest {
    pub fn new(
        host_name: String,
        record_type: String,
        address: String,
        mx_pref: Option<String>,
        email_type: Option<String>,
        ttl: Option<String>,
        flag: Option<String>,
        tag: Option<String>
    ) -> Self {
        HostRequest {
            host_name,
            record_type,
            address,
            mx_pref,
            email_type,
            ttl,
            flag,
            tag,
        }
    }
}

impl NameCheapClient {
    /// Sets DNS host records for a given domain.
    ///
    /// # Example
    ///
    /// ```rust
    /// use namecheap::NameCheapClient;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = NameCheapClient::new(
    ///         "api_user".to_string(),
    ///         "api_key".to_string(),
    ///         "client_ip".to_string(),
    ///         "user_name".to_string(),
    ///         false
    ///     );
    ///
    ///     let host_records = vec![
    ///         HostRequest {
    ///             host_name: "@".to_string(),
    ///             record_type: "A".to_string(),
    ///             address: "213.87.128.103".to_string(),
    ///             mx_pref: None,
    ///             email_type: None,
    ///             ttl: Some("3600".to_string()),
    ///             flag: None,
    ///             tag: None,
    ///         },
    ///         HostRequest {
    ///             host_name: "www".to_string(),
    ///             record_type: "CNAME".to_string(),
    ///             address: "example.com.".to_string(),
    ///             mx_pref: None,
    ///             email_type: None,
    ///             ttl: Some("1200".to_string()),
    ///             flag: None,
    ///             tag: None,
    ///         }
    ///     ];
    ///
    ///     let result = client.domains_dns_set_hosts("domain", "com", host_records).await.unwrap();
    ///     println!("Set Hosts Result: {:?}", result);
    /// }
    /// ```
    pub async fn domains_dns_set_hosts(
        &self,
        sld: &str,
        tld: &str,
        new_hosts: Vec<HostRequest>
    ) -> Result<Value, Box<dyn Error>> {
        // Retrieve existing hosts
        let existing_hosts = self.domains_dns_get_hosts(sld, tld).await?;
        info!("Existing Hosts: {:#?}", existing_hosts);

        // Check if there are existing hosts
        let combined_hosts: Vec<Value> = if
            let Some(existing_hosts_array) = existing_hosts.as_array()
        {
            // Combine existing and new hosts
            existing_hosts_array
                .iter()
                .cloned()
                .chain(
                    new_hosts
                        .iter()
                        .map(|host| {
                            json!({
                        "HostName": host.host_name,
                        "RecordType": host.record_type,
                        "Address": host.address,
                        "MXPref": host.mx_pref,
                        "EmailType": host.email_type,
                        "TTL": host.ttl,
                        "Flag": host.flag,
                        "Tag": host.tag
                    })
                        })
                )
                .collect()
        } else {
            // If no existing hosts, use only new hosts
            new_hosts
                .iter()
                .map(|host| {
                    json!({
                    "HostName": host.host_name,
                    "RecordType": host.record_type,
                    "Address": host.address,
                    "MXPref": host.mx_pref,
                    "EmailType": host.email_type,
                    "TTL": host.ttl,
                    "Flag": host.flag,
                    "Tag": host.tag
                })
                })
                .collect()
        };

        info!("Combined Hosts: {:#?}", combined_hosts);

        let request_values: Vec<Value> = combined_hosts
            .iter()
            .enumerate()
            .flat_map(|(index, host)| {
                let idx = index + 1;
                vec![
                    json!({
                        "Key": format!("HostName{}", idx),
                        "Value": host.get("HostName").unwrap_or(&Value::String("".to_string())).as_str().unwrap().to_string()
                    }),
                    json!({
                        "Key": format!("RecordType{}", idx),
                        "Value": host.get("RecordType").unwrap_or(&Value::String("".to_string())).as_str().unwrap().to_string()
                    }),
                    json!({
                        "Key": format!("Address{}", idx),
                        "Value": host.get("Address").unwrap_or(&Value::String("".to_string())).as_str().unwrap().to_string()
                    }),
                    json!({
                        "Key": format!("TTL{}", idx),
                        "Value": host.get("TTL").unwrap_or(&Value::String("".to_string())).as_str().unwrap().to_string()
                    })
                ]
            })
            .collect();

        let body: Value =
            json!({
            "authDetails": {
                "ParentUserType": "",
                "ParentUserId": 0,
                "UserId": "",
                "UserName": self.user_name,
                "ClientIp": self.client_ip,
                "EndUserIp": "",
                "AdminUserName": self.user_name,
                "DisableSecurityNotification": true,
                "AllowWhenDomainLocked": true,
                "ProceedWhenDomainLockedFlag": true,
                "DefaultChargeForUserName": "",
                "Roles": ["User"]
            },
            "request": {
                "RequestValues": request_values,
                "SLD": sld,
                "TLD": tld
            }
        });
        info!("Request Body: {:#?}", body);

        let xml_body =
            r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <ApiRequest>
              <Command>namecheap.domains.dns.setHosts</Command>
              <ClientIp>{client_ip}</ClientIp>
              <UserName>{user_name}</UserName>
              <ApiUser>{api_user}</ApiUser>
              <ApiKey>{api_key}</ApiKey>
              <SLD>{sld}</SLD>
              <TLD>{tld}</TLD>
              {hosts}
            </ApiRequest>
        "#;

        let hosts_xml: String = combined_hosts
            .iter()
            .enumerate()
            .map(|(index, host)| {
                format!(
                    r#"<Host HostId="{}" Name="{}" Type="{}" Address="{}" TTL="{}" />"#,
                    index + 1,
                    host
                        .get("HostName")
                        .unwrap_or(&Value::String("".to_string()))
                        .as_str()
                        .unwrap(),
                    host
                        .get("RecordType")
                        .unwrap_or(&Value::String("".to_string()))
                        .as_str()
                        .unwrap(),
                    host.get("Address").unwrap_or(&Value::String("".to_string())).as_str().unwrap(),
                    host.get("TTL").unwrap_or(&Value::String("".to_string())).as_str().unwrap()
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        let xml_body = xml_body
            .replace("{client_ip}", &self.client_ip)
            .replace("{user_name}", &self.user_name)
            .replace("{api_user}", &self.api_user)
            .replace("{api_key}", &self.api_key)
            .replace("{sld}", sld)
            .replace("{tld}", tld)
            .replace("{hosts}", &hosts_xml);

        let client = reqwest::Client::new();
        let url = format!(
            "https://api.namecheap.com/xml.response?ApiUser={api_user}&ApiKey={api_key}&UserName={user_name}&Command={command}&ClientIp={client_ip}&SLD={sld}&TLD={tld}",
            api_user = self.api_user,
            api_key = self.api_key,
            user_name = self.user_name,
            command = "namecheap.domains.dns.setHosts",
            client_ip = self.client_ip,
            sld = sld,
            tld = tld
        );
        let response = client
            .post(&url)
            .header("Content-Type", "application/xml")
            .body(xml_body)
            .send().await?;

        let response_text = response.text().await?;
        let json_value: Value = parse_xml_to_json(&response_text)?;
        info!("Response: {:#?}", json_value);
        eprintln!("DEBUG: Response JSON: {:#?}", json_value);

        let status = json_value
            .pointer("/ApiResponse/Status")
            .and_then(Value::as_str)
            .unwrap_or("UNKNOWN");

        eprintln!("DEBUG: API Status: {}", status);

        if status == "ERROR" {
            error!("Namecheap API returned error status");
            error!("Raw XML response: {}", response_text);

            let errors = json_value
                .pointer("/ApiResponse/Errors/Error")
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .map(|err| {
                            let number = err.get("Number").and_then(Value::as_str).unwrap_or("Unknown");
                            let description = err.get("$text").and_then(Value::as_str).unwrap_or("No description");
                            format!("Error {}: {}", number, description)
                        })
                        .collect::<Vec<_>>()
                        .join("; ")
                })
                .unwrap_or_else(|| "Unknown error".to_string());

            return Err(format!("Namecheap API error: {}", errors).into());
        }

        let result = json_value
            .pointer("/ApiResponse/CommandResponse/DomainDNSSetHostsResult")
            .ok_or("Failed to set host records")?
            .clone();

        if result.get("IsSuccess").and_then(Value::as_bool).unwrap_or(false) {
            info!("Set Hosts operation was successful.");
        } else {
            error!("Set Hosts operation failed.");
        }

        info!("Set Hosts Result: {:#?}", result);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use serde_json::json;
    use tracing::info;

    #[tokio::test]
    async fn test_domains_dns_set_hosts() {
        dotenv().ok();

        let client = NameCheapClient::new_from_env().unwrap();

        let new_host1 = HostRequest {
            host_name: "@".to_string(),
            record_type: "A".to_string(),
            address: "213.87.128.103".to_string(),
            mx_pref: None,
            email_type: None,
            ttl: Some("3600".to_string()),
            flag: None,
            tag: None,
        };

        let new_host2 = HostRequest {
            host_name: "www".to_string(),
            record_type: "CNAME".to_string(),
            address: "example.com.".to_string(),
            mx_pref: None,
            email_type: None,
            ttl: Some("1200".to_string()),
            flag: None,
            tag: None,
        };

        let new_hosts = vec![new_host1, new_host2];

        let result = client.domains_dns_set_hosts("xylex", "ai", new_hosts).await.unwrap();
        info!("Set Hosts Result: {:#?}", result);

        // Check if the operation was successful
        assert!(
            result.get("IsSuccess").and_then(Value::as_bool).unwrap_or(false),
            "Expected successful host record setting"
        );
    }
}
