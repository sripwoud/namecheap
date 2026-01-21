//! ### `domains.dns.getHosts` Implementation
//!
//! This module provides the implementation for the `domains.dns.getHosts` method of the NameCheap API.
//!
//! It retrieves DNS host record settings for the requested domain.
//!

use serde_json::{ Value, json };
use std::error::Error;
use tracing::{ info, error };

// crate imports
use crate::{ NameCheapClient, Host };
use crate::utils::request_builder::Request;

impl NameCheapClient {
    /// Retrieves DNS host records for a given domain.
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
    ///     let host_records = client.domains_dns_get_hosts("domain", "com").await.unwrap();
    ///     println!("Host Records: {:?}", host_records);
    /// }
    /// ```
    pub async fn domains_dns_get_hosts(
        &self,
        sld: &str,
        tld: &str
    ) -> Result<Value, Box<dyn Error>> {
        let command = "namecheap.domains.dns.getHosts";
        let params = json!({ "SLD": sld, "TLD": tld });

        let response = Request::new(
            self.clone(),
            command.to_string(),
            Some(1),
            None,
            Some(params),

        ).send().await?;
        info!("Response: {:#?}", response);
        eprintln!("DEBUG get_hosts: Full API response: {:#?}", response);

        let hosts = response
            .pointer("/ApiResponse/CommandResponse/DomainDNSGetHostsResult/host")
            .cloned()
            .unwrap_or_else(|| {
                eprintln!("DEBUG get_hosts: No host records found at expected path");
                eprintln!("DEBUG get_hosts: Checking CommandResponse: {:#?}",
                    response.pointer("/ApiResponse/CommandResponse"));
                json!([])
            });

        eprintln!("DEBUG get_hosts: Extracted hosts: {:#?}", hosts);
        Ok(hosts)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use serde_json::json;
    use tracing::info;

    #[tokio::test]
    async fn test_domains_dns_get_hosts() {
        dotenv().ok();

        let client = NameCheapClient::new_from_env().unwrap();

        let host_records = client.domains_dns_get_hosts("xylex", "ai").await.unwrap();
        info!("Host Records: {:#?}", host_records);
        
        // Check if host_records is an array with at least two items
        assert!(host_records.as_array().map_or(false, |arr| arr.len() >= 2), "Expected at least two host records");
    }
}
