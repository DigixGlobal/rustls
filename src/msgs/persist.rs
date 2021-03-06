use msgs::handshake::SessionID;
use msgs::enums::CipherSuite;
use msgs::codec::{Reader, Codec};
use msgs::handshake::CertificatePayload;
use msgs::base::{PayloadU8, PayloadU16};

use std::mem;

/* These are the keys and values we store in session storage. */

/* --- Client types --- */
/// Keys for session resumption and tickets.
/// Matching value is a `ClientSessionValue`.
#[derive(Debug)]
pub struct ClientSessionKey {
  dns_name: PayloadU8
}

impl Codec for ClientSessionKey {
  fn encode(&self, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(b"session");
    self.dns_name.encode(bytes);
  }

  /* Don't need to read these. */
  fn read(_r: &mut Reader) -> Option<ClientSessionKey> {
    None
  }
}

impl ClientSessionKey {
  pub fn for_dns_name(dns_name: &str) -> ClientSessionKey {
    ClientSessionKey {
      dns_name: PayloadU8::new(dns_name.as_bytes().to_vec())
    }
  }
}

#[derive(Debug)]
pub struct ClientSessionValue {
  pub cipher_suite: CipherSuite,
  pub session_id: SessionID,
  pub ticket: PayloadU16,
  pub master_secret: PayloadU8
}

impl Codec for ClientSessionValue {
  fn encode(&self, bytes: &mut Vec<u8>) {
    self.cipher_suite.encode(bytes);
    self.session_id.encode(bytes);
    self.ticket.encode(bytes);
    self.master_secret.encode(bytes);
  }

  fn read(r: &mut Reader) -> Option<ClientSessionValue> {
    let cs = try_ret!(CipherSuite::read(r));
    let sid = try_ret!(SessionID::read(r));
    let ticket = try_ret!(PayloadU16::read(r));
    let ms = try_ret!(PayloadU8::read(r));

    Some(ClientSessionValue {
      cipher_suite: cs,
      session_id: sid,
      ticket: ticket,
      master_secret: ms
    })
  }
}

impl ClientSessionValue {
  pub fn new(cs: &CipherSuite,
             sessid: &SessionID,
             ticket: Vec<u8>,
             ms: Vec<u8>) -> ClientSessionValue {
    ClientSessionValue {
      cipher_suite: *cs,
      session_id: sessid.clone(),
      ticket: PayloadU16::new(ticket),
      master_secret: PayloadU8::new(ms)
    }
  }

  pub fn take_ticket(&mut self) -> Vec<u8> {
    let new_ticket = PayloadU16::new(Vec::new());
    let old_ticket = mem::replace(&mut self.ticket, new_ticket);
    old_ticket.0
  }
}

/* --- Server types --- */
pub type ServerSessionKey = SessionID;

#[derive(Debug)]
pub struct ServerSessionValue {
  pub cipher_suite: CipherSuite,
  pub master_secret: PayloadU8,
  pub client_cert_chain: Option<CertificatePayload>
}

impl Codec for ServerSessionValue {
  fn encode(&self, bytes: &mut Vec<u8>) {
    self.cipher_suite.encode(bytes);
    self.master_secret.encode(bytes);
    if self.client_cert_chain.is_some() {
      self.client_cert_chain.as_ref().unwrap().encode(bytes);
    }
  }

  fn read(r: &mut Reader) -> Option<ServerSessionValue> {
    let cs = try_ret!(CipherSuite::read(r));
    let ms = try_ret!(PayloadU8::read(r));
    let ccert = if r.any_left() {
      CertificatePayload::read(r)
    } else {
      None
    };

    Some(ServerSessionValue {
      cipher_suite: cs,
      master_secret: ms,
      client_cert_chain: ccert
    })
  }
}

impl ServerSessionValue {
  pub fn new(cs: &CipherSuite, ms: Vec<u8>, cert_chain: &Option<CertificatePayload>) -> ServerSessionValue {
    ServerSessionValue {
      cipher_suite: cs.clone(),
      master_secret: PayloadU8::new(ms),
      client_cert_chain: cert_chain.clone()
    }
  }
}
