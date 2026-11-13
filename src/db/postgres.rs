use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Clone, Default)]
pub struct PostgresClient {
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PgConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: Option<String>,
    pub dbname: String,
}

pub fn parse_pg_url(url: &str) -> Result<PgConfig, String> {
    let trimmed = url.trim();
    let rest = if let Some(s) = trimmed.strip_prefix("postgres://") {
        s
    } else if let Some(s) = trimmed.strip_prefix("postgresql://") {
        s
    } else {
        return Err("Connection string must start with postgres:// or postgresql://".to_string());
    };

    // Strip query parameters (?sslmode=... etc.)
    let (rest, query) = match rest.find('?') {
        Some(idx) => (&rest[..idx], &rest[idx + 1..]),
        None => (rest, ""),
    };

    let mut q_user: Option<String> = None;
    let mut q_password: Option<String> = None;
    if !query.is_empty() {
        for param in query.split('&') {
            if let Some(idx) = param.find('=') {
                let k = &param[..idx];
                let v = &param[idx + 1..];
                if k.eq_ignore_ascii_case("user") {
                    q_user = Some(v.to_string());
                } else if k.eq_ignore_ascii_case("password") {
                    q_password = Some(v.to_string());
                }
            }
        }
    }

    let (auth_part, path_part) = match rest.find('/') {
        Some(idx) => (&rest[..idx], &rest[idx + 1..]),
        None => (rest, ""),
    };

    let dbname = if path_part.is_empty() {
        "postgres".to_string()
    } else {
        path_part.to_string()
    };

    let (user_part, host_port_part) = match auth_part.rfind('@') {
        Some(idx) => (&auth_part[..idx], &auth_part[idx + 1..]),
        None => ("", auth_part),
    };

    let (user, password) = if user_part.is_empty() {
        (
            q_user.unwrap_or_else(|| "postgres".to_string()),
            q_password,
        )
    } else {
        match user_part.find(':') {
            Some(idx) => (
                user_part[..idx].to_string(),
                Some(user_part[idx + 1..].to_string()),
            ),
            None => (user_part.to_string(), q_password),
        }
    };

    let (host, port) = if host_port_part.is_empty() {
        ("127.0.0.1".to_string(), 5432)
    } else {
        match host_port_part.rfind(':') {
            Some(idx) => {
                let h = &host_port_part[..idx];
                let p = host_port_part[idx + 1..]
                    .parse::<u16>()
                    .map_err(|_| format!("Invalid port in database URL: '{}'", &host_port_part[idx + 1..]))?;
                let clean_h = if h.is_empty() || h.eq_ignore_ascii_case("localhost") {
                    "127.0.0.1"
                } else {
                    h
                };
                (clean_h.to_string(), p)
            }
            None => {
                let clean_h = if host_port_part.eq_ignore_ascii_case("localhost") {
                    "127.0.0.1"
                } else {
                    host_port_part
                };
                (clean_h.to_string(), 5432)
            }
        }
    };

    Ok(PgConfig {
        host,
        port,
        user,
        password,
        dbname,
    })
}

// RFC 1321 MD5 Implementation in pure Rust
pub fn md5_hex(data: &[u8]) -> String {
    let digest = md5_digest(data);
    let mut out = String::with_capacity(32);
    for b in digest {
        use std::fmt::Write;
        let _ = write!(&mut out, "{:02x}", b);
    }
    out
}

pub fn md5_digest(data: &[u8]) -> [u8; 16] {
    const S: [u32; 64] = [
        7, 12, 17, 22,  7, 12, 17, 22,  7, 12, 17, 22,  7, 12, 17, 22,
        5,  9, 14, 20,  5,  9, 14, 20,  5,  9, 14, 20,  5,  9, 14, 20,
        4, 11, 16, 23,  4, 11, 16, 23,  4, 11, 16, 23,  4, 11, 16, 23,
        6, 10, 15, 21,  6, 10, 15, 21,  6, 10, 15, 21,  6, 10, 15, 21,
    ];

    const K: [u32; 64] = [
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee,
        0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
        0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be,
        0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
        0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa,
        0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
        0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
        0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
        0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c,
        0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
        0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05,
        0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
        0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039,
        0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1,
        0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
    ];

    let mut a0: u32 = 0x67452301;
    let mut b0: u32 = 0xefcdab89;
    let mut c0: u32 = 0x98badcfe;
    let mut d0: u32 = 0x10325476;

    let orig_len = data.len();
    let bit_len = (orig_len as u64) * 8;

    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_le_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut m = [0u32; 16];
        for i in 0..16 {
            m[i] = u32::from_le_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }

        let mut a = a0;
        let mut b = b0;
        let mut c = c0;
        let mut d = d0;

        for i in 0..64 {
            let (f, g) = if i < 16 {
                ((b & c) | ((!b) & d), i)
            } else if i < 32 {
                ((d & b) | ((!d) & c), (5 * i + 1) % 16)
            } else if i < 48 {
                (b ^ c ^ d, (3 * i + 5) % 16)
            } else {
                (c ^ (b | (!d)), (7 * i) % 16)
            };

            let temp = d;
            d = c;
            c = b;
            let sum = a.wrapping_add(f).wrapping_add(K[i]).wrapping_add(m[g]);
            b = b.wrapping_add(sum.rotate_left(S[i]));
            a = temp;
        }

        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }

    let mut result = [0u8; 16];
    result[0..4].copy_from_slice(&a0.to_le_bytes());
    result[4..8].copy_from_slice(&b0.to_le_bytes());
    result[8..12].copy_from_slice(&c0.to_le_bytes());
    result[12..16].copy_from_slice(&d0.to_le_bytes());
    result
}

fn parse_error_response(body: &[u8]) -> String {
    let mut message = String::new();
    let mut code = String::new();
    let mut offset = 0;
    while offset < body.len() {
        let field_type = body[offset];
        offset += 1;
        if field_type == 0 {
            break;
        }
        let start = offset;
        while offset < body.len() && body[offset] != 0 {
            offset += 1;
        }
        let val = String::from_utf8_lossy(&body[start..offset]);
        if offset < body.len() {
            offset += 1; // skip null
        }
        match field_type {
            b'M' => message = val.to_string(),
            b'C' => code = val.to_string(),
            _ => {}
        }
    }
    if !code.is_empty() {
        format!("[{}] {}", code, message)
    } else if !message.is_empty() {
        message
    } else {
        "Unknown PostgreSQL error".to_string()
    }
}

pub async fn execute_raw_postgres_query(
    cfg: &PgConfig,
    sql: &str,
    timeout_duration: Duration,
) -> Result<Vec<Value>, String> {
    let connect_fut = TcpStream::connect((cfg.host.as_str(), cfg.port));
    let mut stream = tokio::time::timeout(timeout_duration, connect_fut)
        .await
        .map_err(|_| format!("Connection timeout: could not reach PostgreSQL at {}:{} within {:?}", cfg.host, cfg.port, timeout_duration))?
        .map_err(|e| format!("Could not connect to PostgreSQL at {}:{} ({}). Please check that your PostgreSQL service is running and listening on port {}.", cfg.host, cfg.port, e, cfg.port))?;

    // 1. Send StartupMessage (Protocol 3.0: 196608)
    let mut params_payload = Vec::new();
    params_payload.extend_from_slice(&196608i32.to_be_bytes());
    params_payload.extend_from_slice(b"user\0");
    params_payload.extend_from_slice(cfg.user.as_bytes());
    params_payload.push(0);
    params_payload.extend_from_slice(b"database\0");
    params_payload.extend_from_slice(cfg.dbname.as_bytes());
    params_payload.push(0);
    params_payload.extend_from_slice(b"client_encoding\0UTF8\0");
    params_payload.push(0);

    let packet_len = (params_payload.len() + 4) as i32;
    let mut startup_packet = Vec::with_capacity(packet_len as usize);
    startup_packet.extend_from_slice(&packet_len.to_be_bytes());
    startup_packet.extend_from_slice(&params_payload);

    stream.write_all(&startup_packet).await
        .map_err(|e| format!("Failed to send PostgreSQL startup message: {}", e))?;

    // 2. Authentication & Ready loop
    loop {
        let tag = stream.read_u8().await
            .map_err(|e| format!("Failed reading server message tag: {}", e))?;
        let len = stream.read_i32().await
            .map_err(|e| format!("Failed reading server message length: {}", e))?;
        if len < 4 {
            return Err("Invalid message length from PostgreSQL server".to_string());
        }
        let body_len = (len - 4) as usize;
        let mut body = vec![0u8; body_len];
        if body_len > 0 {
            stream.read_exact(&mut body).await
                .map_err(|e| format!("Failed reading message body: {}", e))?;
        }

        match tag {
            b'R' => {
                // Authentication
                if body.len() < 4 {
                    return Err("Malformed authentication message from PostgreSQL".to_string());
                }
                let auth_type = i32::from_be_bytes([body[0], body[1], body[2], body[3]]);
                match auth_type {
                    0 => {
                        // AuthenticationOk, continue waiting for ReadyForQuery
                    }
                    3 => {
                        // Cleartext password
                        let pass = cfg.password.as_deref().unwrap_or("");
                        let mut pass_packet = Vec::new();
                        pass_packet.push(b'p');
                        let plen = (4 + pass.len() + 1) as i32;
                        pass_packet.extend_from_slice(&plen.to_be_bytes());
                        pass_packet.extend_from_slice(pass.as_bytes());
                        pass_packet.push(0);
                        stream.write_all(&pass_packet).await
                            .map_err(|e| format!("Failed to send cleartext password: {}", e))?;
                    }
                    5 => {
                        // MD5 Password
                        if body.len() < 8 {
                            return Err("Malformed MD5 auth request from PostgreSQL".to_string());
                        }
                        let salt = &body[4..8];
                        let pass = cfg.password.as_deref().unwrap_or("");

                        // step 1: md5(password + user)
                        let mut step1_input = Vec::new();
                        step1_input.extend_from_slice(pass.as_bytes());
                        step1_input.extend_from_slice(cfg.user.as_bytes());
                        let step1_hex = md5_hex(&step1_input);

                        // step 2: md5(step1_hex + salt)
                        let mut step2_input = Vec::new();
                        step2_input.extend_from_slice(step1_hex.as_bytes());
                        step2_input.extend_from_slice(salt);
                        let step2_hex = md5_hex(&step2_input);

                        let response_str = format!("md5{}", step2_hex);
                        let mut pass_packet = Vec::new();
                        pass_packet.push(b'p');
                        let plen = (4 + response_str.len() + 1) as i32;
                        pass_packet.extend_from_slice(&plen.to_be_bytes());
                        pass_packet.extend_from_slice(response_str.as_bytes());
                        pass_packet.push(0);
                        stream.write_all(&pass_packet).await
                            .map_err(|e| format!("Failed to send MD5 password: {}", e))?;
                    }
                    10 => {
                        return Err("PostgreSQL server requires SASL/SCRAM-SHA-256. Please configure user to md5 or trust in pg_hba.conf.".to_string());
                    }
                    other => {
                        return Err(format!("Unsupported PostgreSQL authentication type: {}", other));
                    }
                }
            }
            b'K' | b'S' => {
                // BackendKeyData or ParameterStatus, ignore
            }
            b'Z' => {
                // ReadyForQuery!
                break;
            }
            b'E' => {
                let err_msg = parse_error_response(&body);
                return Err(format!("PostgreSQL authentication failed: {}", err_msg));
            }
            _ => {}
        }
    }

    // 3. Send Query
    let sql_bytes = sql.as_bytes();
    let qlen = (4 + sql_bytes.len() + 1) as i32;
    let mut qpacket = Vec::with_capacity((qlen + 1) as usize);
    qpacket.push(b'Q');
    qpacket.extend_from_slice(&qlen.to_be_bytes());
    qpacket.extend_from_slice(sql_bytes);
    qpacket.push(0);
    stream.write_all(&qpacket).await
        .map_err(|e| format!("Failed to send SQL query: {}", e))?;

    // 4. Collect Query Results
    let mut columns: Vec<String> = Vec::new();
    let mut rows: Vec<Value> = Vec::new();

    loop {
        let tag = stream.read_u8().await
            .map_err(|e| format!("Failed reading query response tag: {}", e))?;
        let len = stream.read_i32().await
            .map_err(|e| format!("Failed reading query response length: {}", e))?;
        let body_len = if len >= 4 { (len - 4) as usize } else { 0 };
        let mut body = vec![0u8; body_len];
        if body_len > 0 {
            stream.read_exact(&mut body).await
                .map_err(|e| format!("Failed reading query response body: {}", e))?;
        }

        match tag {
            b'T' => {
                // RowDescription
                if body.len() >= 2 {
                    let num_fields = i16::from_be_bytes([body[0], body[1]]) as usize;
                    columns.clear();
                    let mut offset = 2;
                    for _ in 0..num_fields {
                        let start = offset;
                        while offset < body.len() && body[offset] != 0 {
                            offset += 1;
                        }
                        let name = String::from_utf8_lossy(&body[start..offset]).to_string();
                        columns.push(name.to_ascii_uppercase());
                        offset += 1; // skip null
                        offset += 18; // table_oid(4) + col_attr(2) + type_oid(4) + type_len(2) + type_mod(4) + format(2)
                    }
                }
            }
            b'D' => {
                // DataRow
                if body.len() >= 2 {
                    let num_fields = i16::from_be_bytes([body[0], body[1]]) as usize;
                    let mut offset = 2;
                    let mut map = serde_json::Map::new();
                    for col_idx in 0..num_fields {
                        let col_name = columns.get(col_idx).cloned().unwrap_or_else(|| format!("COL{}", col_idx));
                        if offset + 4 <= body.len() {
                            let flen = i32::from_be_bytes([body[offset], body[offset + 1], body[offset + 2], body[offset + 3]]);
                            offset += 4;
                            if flen == -1 {
                                map.insert(col_name, Value::Null);
                            } else if flen >= 0 {
                                let flen = flen as usize;
                                if offset + flen <= body.len() {
                                    let raw_str = String::from_utf8_lossy(&body[offset..offset + flen]);
                                    if let Ok(i) = raw_str.parse::<i64>() {
                                        map.insert(col_name, serde_json::json!(i));
                                    } else if let Ok(f) = raw_str.parse::<f64>() {
                                        map.insert(col_name, serde_json::json!(f));
                                    } else if raw_str.eq_ignore_ascii_case("t") || raw_str.eq_ignore_ascii_case("true") {
                                        map.insert(col_name, Value::Bool(true));
                                    } else if raw_str.eq_ignore_ascii_case("f") || raw_str.eq_ignore_ascii_case("false") {
                                        map.insert(col_name, Value::Bool(false));
                                    } else {
                                        map.insert(col_name, Value::String(raw_str.to_string()));
                                    }
                                    offset += flen;
                                }
                            }
                        }
                    }
                    rows.push(Value::Object(map));
                }
            }
            b'C' => {
                // CommandComplete, keep reading until ReadyForQuery
            }
            b'Z' => {
                // ReadyForQuery
                break;
            }
            b'E' => {
                let err_msg = parse_error_response(&body);
                return Err(format!("PostgreSQL query error: {}", err_msg));
            }
            _ => {}
        }
    }

    // 5. Send Terminate
    let _ = stream.write_all(&[b'X', 0, 0, 0, 4]).await;
    Ok(rows)
}

impl PostgresClient {
    pub fn new(url: Option<String>) -> Self {
        Self { url }
    }

    pub async fn is_connected(&self) -> bool {
        if let Some(ref url) = self.url {
            self.execute_scoped_query(url, "SELECT 1").await.is_ok()
        } else {
            false
        }
    }

    pub fn translate_open_sql_select(
        table: &str,
        fields: &[String],
        where_clause: Option<&str>,
        single: bool,
    ) -> String {
        let field_list = if fields.is_empty() || (fields.len() == 1 && fields[0] == "*") {
            "*".to_string()
        } else {
            fields.join(", ")
        };

        let mut sql = format!("SELECT {} FROM {}", field_list, table);
        if let Some(w) = where_clause {
            if !w.trim().is_empty() {
                sql.push_str(" WHERE ");
                sql.push_str(w);
            }
        }
        if single {
            sql.push_str(" LIMIT 1");
        }
        sql
    }

    pub async fn execute_query(&self, sql: &str) -> Result<Vec<Value>, String> {
        if let Some(ref url) = self.url {
            self.execute_scoped_query(url, sql).await
        } else {
            Err("No database URL configured for PostgresClient.".to_string())
        }
    }

    pub async fn execute_scoped_query(&self, db_url: &str, sql: &str) -> Result<Vec<Value>, String> {
        let trimmed = db_url.trim();
        if trimmed.is_empty() {
            return Err("Empty database URL provided.".to_string());
        }

        let cfg = parse_pg_url(trimmed)?;
        execute_raw_postgres_query(&cfg, sql, Duration::from_millis(3000)).await
    }

    pub async fn fetch_all_public_tables(&self, db_url: &str) -> Result<HashMap<String, Vec<Value>>, String> {
        let tables_query = "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' ORDER BY table_name LIMIT 30;";
        let rows = self.execute_scoped_query(db_url, tables_query).await?;
        let mut result_map = HashMap::new();

        for row in rows {
            if let Some(table_name) = row.get("TABLE_NAME").and_then(|v| v.as_str()) {
                let safe_table = table_name.replace('"', "").replace(';', "");
                let fetch_sql = format!("SELECT * FROM \"{}\" LIMIT 500;", safe_table);
                if let Ok(data_rows) = self.execute_scoped_query(db_url, &fetch_sql).await {
                    result_map.insert(safe_table.to_ascii_uppercase(), data_rows);
                }
            }
        }

        Ok(result_map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rfc1321_md5_vectors() {
        assert_eq!(md5_hex(b""), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(md5_hex(b"a"), "0cc175b9c0f1b6a831c399e269772661");
        assert_eq!(md5_hex(b"abc"), "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(md5_hex(b"message digest"), "f96b697d7cb7938d525a2f31aaf161d0");
        assert_eq!(md5_hex(b"abcdefghijklmnopqrstuvwxyz"), "c3fcd3d76192e4007dfb496cca67e13b");
        assert_eq!(
            md5_hex(b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"),
            "d174ab98d277d9f5a5611c2c9f419d9f"
        );
        assert_eq!(
            md5_hex(b"12345678901234567890123456789012345678901234567890123456789012345678901234567890"),
            "57edf4a22be3c955ac49da2e2107b67a"
        );
    }

    #[test]
    fn test_parse_pg_url_standard() {
        let url = "postgres://postgres:password123@localhost:5432/postgres";
        let cfg = parse_pg_url(url).expect("Should parse URL");
        assert_eq!(cfg.host, "127.0.0.1");
        assert_eq!(cfg.port, 5432);
        assert_eq!(cfg.user, "postgres");
        assert_eq!(cfg.password.as_deref(), Some("password123"));
        assert_eq!(cfg.dbname, "postgres");
    }

    #[test]
    fn test_parse_pg_url_custom_host() {
        let url = "postgresql://myuser:secret@db.neon.tech:5432/main_db?sslmode=require";
        let cfg = parse_pg_url(url).expect("Should parse Neon URL");
        assert_eq!(cfg.host, "db.neon.tech");
        assert_eq!(cfg.port, 5432);
        assert_eq!(cfg.user, "myuser");
        assert_eq!(cfg.password.as_deref(), Some("secret"));
        assert_eq!(cfg.dbname, "main_db");
    }

    #[test]
    fn test_parse_pg_url_query_credentials() {
        let url = "postgresql://db.neon.tech:5432/main_db?user=myuser&password=secret&sslmode=require";
        let cfg = parse_pg_url(url).expect("Should parse query credentials");
        assert_eq!(cfg.host, "db.neon.tech");
        assert_eq!(cfg.port, 5432);
        assert_eq!(cfg.user, "myuser");
        assert_eq!(cfg.password.as_deref(), Some("secret"));
        assert_eq!(cfg.dbname, "main_db");
    }
}
