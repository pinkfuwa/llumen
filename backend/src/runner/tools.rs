//! Tools and utilities for Lua code execution including SQLite, HTTP, and CSV support.

use anyhow::Result;
use mlua::{Lua, Value};
use sqlx::{Column, Row, SqlitePool};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Check if an IP address is in a private network range
pub fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(addr) => {
            // RFC 1918 private IPv4 ranges
            addr.is_private()
                || addr.is_loopback()
                || addr.is_link_local()
                || addr.is_broadcast()
                || addr.is_documentation()
                || addr.is_unspecified()
        }
        IpAddr::V6(addr) => {
            // RFC 4193 private IPv6 ranges
            addr.is_loopback()
                || addr.is_unspecified()
                || (addr.segments()[0] & 0xfe00) == 0xfc00 // fc00::/7
                || addr.is_multicast()
        }
    }
}

/// Validate a URL and ensure it doesn't point to private IP addresses
pub async fn validate_url(url: &str) -> Result<()> {
    let parsed = reqwest::Url::parse(url)?;

    // Check if the host is an IP address
    if let Some(host) = parsed.host_str() {
        // Try to parse as IP address
        if let Ok(ip) = host.parse::<IpAddr>() {
            if is_private_ip(&ip) {
                anyhow::bail!("Access to private IP addresses is not allowed");
            }
        } else {
            // Resolve hostname to check IP addresses
            match tokio::net::lookup_host(format!("{}:{}", host, parsed.port().unwrap_or(80))).await
            {
                Ok(addrs) => {
                    for addr in addrs {
                        if is_private_ip(&addr.ip()) {
                            anyhow::bail!(
                                "Hostname resolves to private IP address, access not allowed"
                            );
                        }
                    }
                }
                Err(e) => {
                    anyhow::bail!("Failed to resolve hostname: {}", e);
                }
            }
        }
    }

    Ok(())
}

/// SQLite context for Lua
pub struct SqliteContext {
    pool: Arc<Mutex<Option<SqlitePool>>>,
}

impl SqliteContext {
    pub fn new() -> Self {
        Self {
            pool: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn init_pool(&self) -> Result<()> {
        let mut pool_guard = self.pool.lock().await;
        if pool_guard.is_none() {
            let pool = SqlitePool::connect(":memory:").await?;
            *pool_guard = Some(pool);
        }
        Ok(())
    }

    pub async fn execute_query(&self, query: &str) -> Result<Vec<serde_json::Value>> {
        let pool_guard = self.pool.lock().await;
        let pool = pool_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("SQLite pool not initialized"))?;

        let mut results = Vec::new();
        let rows = sqlx::query(query).fetch_all(pool).await?;

        for row in rows {
            let mut obj = serde_json::Map::new();
            for (i, column) in row.columns().iter().enumerate() {
                let value: serde_json::Value = match row.try_get_raw(i) {
                    Ok(_) => {
                        // Try to get different types
                        if let Ok(v) = row.try_get::<i64, _>(i) {
                            serde_json::Value::Number(v.into())
                        } else if let Ok(v) = row.try_get::<f64, _>(i) {
                            serde_json::Number::from_f64(v)
                                .map(serde_json::Value::Number)
                                .unwrap_or(serde_json::Value::Null)
                        } else if let Ok(v) = row.try_get::<String, _>(i) {
                            serde_json::Value::String(v)
                        } else if let Ok(v) = row.try_get::<bool, _>(i) {
                            serde_json::Value::Bool(v)
                        } else {
                            serde_json::Value::Null
                        }
                    }
                    Err(_) => serde_json::Value::Null,
                };
                obj.insert(column.name().to_string(), value);
            }
            results.push(serde_json::Value::Object(obj));
        }

        Ok(results)
    }

    pub async fn load_csv(&self, csv_data: &str, table_name: &str) -> Result<()> {
        let pool_guard = self.pool.lock().await;
        let pool = pool_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("SQLite pool not initialized"))?;

        let mut reader = csv::Reader::from_reader(csv_data.as_bytes());
        let headers = reader.headers()?.clone();

        // Create table with all columns as TEXT for simplicity
        let columns: Vec<String> = headers.iter().map(|h| format!("{} TEXT", h)).collect();
        let create_table = format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            table_name,
            columns.join(", ")
        );
        sqlx::query(&create_table).execute(pool).await?;

        // Insert data
        for result in reader.records() {
            let record = result?;
            let placeholders: Vec<String> =
                (0..record.len()).map(|i| format!("${}", i + 1)).collect();
            let insert_query = format!(
                "INSERT INTO {} VALUES ({})",
                table_name,
                placeholders.join(", ")
            );

            let mut query = sqlx::query(&insert_query);
            for field in record.iter() {
                query = query.bind(field);
            }
            query.execute(pool).await?;
        }

        Ok(())
    }
}

/// Register SQL functions for Lua
pub fn register_sql_functions(lua: &Lua, ctx: Arc<SqliteContext>) -> Result<()> {
    let globals = lua.globals();

    // Create sql table
    let sql_table = lua.create_table()?;

    // sql.query function
    let ctx_query = ctx.clone();
    let query_fn = lua.create_async_function(move |lua, query: String| {
        let ctx = ctx_query.clone();
        async move {
            // Ensure pool is initialized
            ctx.init_pool()
                .await
                .map_err(|e| mlua::Error::external(e))?;

            // Execute query
            let results = ctx
                .execute_query(&query)
                .await
                .map_err(|e| mlua::Error::external(e))?;

            // Convert to Lua value
            let json_str = serde_json::to_string(&results).map_err(|e| mlua::Error::external(e))?;
            let lua_value: Value = lua
                .load(&format!("return {}", json_str))
                .eval()
                .unwrap_or(Value::Nil);

            Ok(lua_value)
        }
    })?;
    sql_table.set("query", query_fn)?;

    // sql.load_csv function
    let ctx_load_csv = ctx.clone();
    let load_csv_fn =
        lua.create_async_function(move |_lua, (csv_data, table_name): (String, String)| {
            let ctx = ctx_load_csv.clone();
            async move {
                // Ensure pool is initialized
                ctx.init_pool()
                    .await
                    .map_err(|e| mlua::Error::external(e))?;

                // Load CSV
                ctx.load_csv(&csv_data, &table_name)
                    .await
                    .map_err(|e| mlua::Error::external(e))?;

                Ok(format!("Loaded CSV data into table '{}'", table_name))
            }
        })?;
    sql_table.set("load_csv", load_csv_fn)?;

    globals.set("sql", sql_table)?;
    Ok(())
}

/// Register HTTP functions for Lua
pub fn register_http_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // Create http table
    let http_table = lua.create_table()?;

    // http.get function
    let get_fn = lua.create_async_function(|_lua, url: String| async move {
        // Validate URL
        validate_url(&url)
            .await
            .map_err(|e| mlua::Error::external(e))?;

        // Make HTTP request
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| mlua::Error::external(e))?;

        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|e| mlua::Error::external(e))?;

        Ok((status, body))
    })?;
    http_table.set("get", get_fn)?;

    // http.post function
    let post_fn = lua.create_async_function(|_lua, (url, body): (String, String)| async move {
        // Validate URL
        validate_url(&url)
            .await
            .map_err(|e| mlua::Error::external(e))?;

        // Make HTTP request
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .body(body)
            .send()
            .await
            .map_err(|e| mlua::Error::external(e))?;

        let status = response.status().as_u16();
        let response_body = response
            .text()
            .await
            .map_err(|e| mlua::Error::external(e))?;

        Ok((status, response_body))
    })?;
    http_table.set("post", post_fn)?;

    globals.set("http", http_table)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_ip_detection() {
        // IPv4 private addresses
        assert!(is_private_ip(&"192.168.1.1".parse().unwrap()));
        assert!(is_private_ip(&"10.0.0.1".parse().unwrap()));
        assert!(is_private_ip(&"172.16.0.1".parse().unwrap()));
        assert!(is_private_ip(&"127.0.0.1".parse().unwrap()));

        // IPv4 public addresses
        assert!(!is_private_ip(&"8.8.8.8".parse().unwrap()));
        assert!(!is_private_ip(&"1.1.1.1".parse().unwrap()));

        // IPv6 private addresses
        assert!(is_private_ip(&"::1".parse().unwrap()));
        assert!(is_private_ip(&"fc00::1".parse().unwrap()));

        // IPv6 public addresses
        assert!(!is_private_ip(&"2001:4860:4860::8888".parse().unwrap()));
    }

    #[tokio::test]
    async fn test_sqlite_context() {
        let ctx = SqliteContext::new();
        ctx.init_pool().await.unwrap();

        // Create table
        ctx.execute_query("CREATE TABLE test (id INTEGER, name TEXT)")
            .await
            .unwrap();

        // Insert data
        ctx.execute_query("INSERT INTO test VALUES (1, 'Alice')")
            .await
            .unwrap();

        // Query data
        let results = ctx.execute_query("SELECT * FROM test").await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_load_csv() {
        let ctx = SqliteContext::new();
        ctx.init_pool().await.unwrap();

        let csv_data = "id,name\n1,Alice\n2,Bob";
        ctx.load_csv(csv_data, "people").await.unwrap();

        let results = ctx.execute_query("SELECT * FROM people").await.unwrap();
        assert_eq!(results.len(), 2);
    }
}
