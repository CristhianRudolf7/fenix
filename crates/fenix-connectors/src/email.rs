use super::*;
use lettre::{Message, SmtpTransport, Transport, transport::smtp::authentication::Credentials};
use std::{
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};

pub(crate) async fn execute(c: Connection, operation: &str, args: Value) -> Result<Value> {
    let operation = operation.to_owned();
    ensure!(
        ["test", "mail_list", "mail_read", "mail_send"].contains(&operation.as_str()),
        "Unknown email operation"
    );
    tokio::task::spawn_blocking(move || call(&c, &operation, &args)).await?
}
fn host(c: &Connection, key: &str) -> Result<String> {
    let host = c.settings[key]
        .as_str()
        .context("Configure IMAP and SMTP server names first")?;
    ensure!(
        !host.is_empty()
            && host.len() <= 253
            && host
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b".-".contains(&b)),
        "Invalid mail server name"
    );
    Ok(host.into())
}
fn smtp(c: &Connection, username: &str, password: &str) -> Result<SmtpTransport> {
    let host = host(c, "smtp_host")?;
    let builder = if c.settings["smtp_starttls"] == true {
        SmtpTransport::starttls_relay(&host)?
    } else {
        SmtpTransport::relay(&host)?
    };
    Ok(builder
        .credentials(Credentials::new(username.into(), password.into()))
        .timeout(Some(Duration::from_secs(20)))
        .build())
}
fn call(c: &Connection, operation: &str, args: &Value) -> Result<Value> {
    let username = c.credentials["username"]
        .as_str()
        .context("Email username required")?;
    let password = c.credentials["password"]
        .as_str()
        .context("Email application password required")?;
    if operation == "mail_send" {
        ensure!(c.allow_write, "Enable write access before sending email");
        let email = Message::builder()
            .from(username.parse()?)
            .to(args["to"].as_str().context("Recipient required")?.parse()?)
            .subject(args["subject"].as_str().context("Subject required")?)
            .body(
                args["text"]
                    .as_str()
                    .context("Message text required")?
                    .to_owned(),
            )?;
        let response=smtp(c,username,password)?.send(&email).map_err(|_|anyhow::anyhow!("SMTP send failed; delivery may be uncertain. Check the destination before retrying."))?;
        return Ok(
            json!({"accepted":response.is_positive(),"server_code":response.code().to_string()}),
        );
    }
    let domain = host(c, "imap_host")?;
    let address = (domain.as_str(), 993)
        .to_socket_addrs()?
        .next()
        .context("Mail server has no address")?;
    let tcp = TcpStream::connect_timeout(&address, Duration::from_secs(15))?;
    tcp.set_read_timeout(Some(Duration::from_secs(20)))?;
    tcp.set_write_timeout(Some(Duration::from_secs(20)))?;
    let tls = native_tls::TlsConnector::new()?
        .connect(&domain, tcp)
        .map_err(|_| anyhow::anyhow!("Cannot establish verified IMAP TLS"))?;
    let mut client = imap::Client::new(tls);
    client
        .read_greeting()
        .map_err(|_| anyhow::anyhow!("Invalid IMAP greeting"))?;
    let mut session=client.login(username,password).map_err(|_|anyhow::anyhow!("IMAP authentication failed; use an application password or connect through Google/Microsoft OAuth"))?;
    let result = (|| -> Result<Value> {
        if operation == "test" {
            session.noop()?;
            ensure!(
                smtp(c, username, password)?
                    .test_connection()
                    .map_err(|_| anyhow::anyhow!("SMTP connection check failed"))?,
                "SMTP server rejected connection"
            );
            return Ok(json!({"imap":true,"smtp":true}));
        }
        let folder = args["folder"].as_str().unwrap_or("INBOX");
        ensure!(!folder.contains(['\r', '\n']), "Invalid mailbox name");
        let mailbox = session.examine(folder)?; // EXAMINE and BODY.PEEK never mark messages read.
        let mut items = vec![];
        if operation == "mail_list" {
            if mailbox.exists == 0 {
                return Ok(json!({"messages":items,"uid_validity":mailbox.uid_validity}));
            }
            let start = mailbox.exists.saturating_sub(19).max(1);
            for mail in session.fetch(format!("{start}:*"),"(UID RFC822.SIZE BODY.PEEK[HEADER.FIELDS (DATE FROM TO SUBJECT MESSAGE-ID)]<0.16384>)")?.iter() {
                items.push(json!({"uid":mail.uid,"bytes":mail.size,"headers":mail.header().map(String::from_utf8_lossy)}));
            }
        } else {
            let uid = args["uid"].as_u64().context("Message UID required")?;
            ensure!(uid > 0 && uid <= u64::from(u32::MAX), "Invalid message UID");
            let expected = args["uid_validity"]
                .as_u64()
                .context("uid_validity from mail_list is required")?;
            ensure!(
                mailbox.uid_validity.map(u64::from) == Some(expected),
                "Mailbox UIDs changed; list the mailbox again"
            );
            for mail in session
                .uid_fetch(uid.to_string(), "(UID RFC822.SIZE BODY.PEEK[]<0.131072>)")?
                .iter()
            {
                items.push(json!({"uid":mail.uid,"bytes":mail.size,"raw":mail.body().map(String::from_utf8_lossy),"truncated":mail.size.is_some_and(|size|size>131072)}));
            }
        }
        Ok(json!({"messages":items,"uid_validity":mailbox.uid_validity,"total":mailbox.exists}))
    })();
    let _ = session.logout();
    result
}
