use async_trait::async_trait;
use bytes::Bytes;
use log::info;
use pingora::Result;
use pingora::http::{RequestHeader, ResponseHeader};
use pingora::proxy::{ProxyHttp, Session, http_proxy_service};
use pingora::server::Server;
use pingora::server::configuration::Opt;
use pingora::upstreams::peer::HttpPeer;
use std::env;
use std::time::Duration;

#[derive(Clone)]
struct Config {
    listen_addr: String,
    web_upstream: String,
    socketio_upstream: String,
    site_host: String,
    forwarded_proto: String,
}

#[derive(Clone, Copy)]
enum UpstreamKind {
    Web,
    SocketIo,
}

struct RequestCtx {
    upstream: UpstreamKind,
}

struct ErpNextProxy {
    config: Config,
}

impl Config {
    fn from_env() -> Self {
        Self {
            listen_addr: env_or("PINGORA_LISTEN", "127.0.0.1:8088"),
            web_upstream: env_or("PINGORA_WEB_UPSTREAM", "127.0.0.1:8000"),
            socketio_upstream: env_or("PINGORA_SOCKETIO_UPSTREAM", "127.0.0.1:9000"),
            site_host: env_or("PINGORA_SITE_HOST", "erpnext.localhost"),
            forwarded_proto: env_or("PINGORA_FORWARDED_PROTO", "http"),
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn route_for(path: &str) -> UpstreamKind {
    if path.starts_with("/socket.io") {
        UpstreamKind::SocketIo
    } else {
        UpstreamKind::Web
    }
}

#[async_trait]
impl ProxyHttp for ErpNextProxy {
    type CTX = RequestCtx;

    fn new_ctx(&self) -> Self::CTX {
        RequestCtx {
            upstream: UpstreamKind::Web,
        }
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool>
    where
        Self::CTX: Send + Sync,
    {
        let path = session.req_header().uri.path();

        if path == "/_pingora_health" {
            let mut header = ResponseHeader::build(200, Some(2))?;
            header.insert_header("Content-Type", "text/plain; charset=utf-8")?;
            header.insert_header("Cache-Control", "no-store")?;
            header.insert_header("Server", "pingora-erpnext")?;
            session
                .write_response_header(Box::new(header), false)
                .await?;
            session
                .write_response_body(Some(Bytes::from_static(b"ok")), true)
                .await?;
            return Ok(true);
        }

        ctx.upstream = route_for(path);
        Ok(false)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let upstream = match ctx.upstream {
            UpstreamKind::Web => self.config.web_upstream.as_str(),
            UpstreamKind::SocketIo => self.config.socketio_upstream.as_str(),
        };

        let mut peer = HttpPeer::new(upstream, false, self.config.site_host.clone());
        peer.options.connection_timeout = Some(Duration::from_secs(5));
        peer.options.total_connection_timeout = Some(Duration::from_secs(10));
        peer.options.idle_timeout = Some(Duration::from_secs(60));
        peer.options.write_timeout = Some(Duration::from_secs(60));
        peer.options.read_timeout = match ctx.upstream {
            UpstreamKind::Web => Some(Duration::from_secs(120)),
            UpstreamKind::SocketIo => Some(Duration::from_secs(3600)),
        };

        Ok(Box::new(peer))
    }

    async fn upstream_request_filter(
        &self,
        session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()>
    where
        Self::CTX: Send + Sync,
    {
        upstream_request.insert_header("Host", self.config.site_host.as_str())?;
        upstream_request.insert_header("X-Forwarded-Host", self.config.site_host.as_str())?;
        upstream_request
            .insert_header("X-Forwarded-Proto", self.config.forwarded_proto.as_str())?;
        upstream_request.insert_header("X-Frappe-Site-Name", self.config.site_host.as_str())?;

        if let Some(client_ip) = session
            .client_addr()
            .and_then(|client_addr| client_addr.as_inet())
            .map(|addr| addr.ip().to_string())
        {
            upstream_request.insert_header("X-Real-IP", client_ip.as_str())?;
            upstream_request.insert_header("X-Forwarded-For", client_ip.as_str())?;
        }

        Ok(())
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()>
    where
        Self::CTX: Send + Sync,
    {
        upstream_response.insert_header("Server", "pingora-erpnext")?;
        upstream_response.remove_header("alt-svc");
        Ok(())
    }

    async fn logging(
        &self,
        session: &mut Session,
        error: Option<&pingora::Error>,
        ctx: &mut Self::CTX,
    ) {
        let status = session
            .response_written()
            .map_or(0, |response| response.status.as_u16());
        let upstream = match ctx.upstream {
            UpstreamKind::Web => "web",
            UpstreamKind::SocketIo => "socket.io",
        };

        if let Some(error) = error {
            info!(
                "{} -> {status} upstream={upstream} ({error})",
                self.request_summary(session, ctx)
            );
        } else {
            info!(
                "{} -> {status} upstream={upstream}",
                self.request_summary(session, ctx)
            );
        }
    }
}

fn main() {
    env_logger::init();

    let config = Config::from_env();
    let opt = Opt::parse_args();
    let mut server = Server::new(Some(opt)).expect("failed to initialize Pingora server");
    server.bootstrap();

    let mut proxy = http_proxy_service(
        &server.configuration,
        ErpNextProxy {
            config: config.clone(),
        },
    );
    proxy.add_tcp(&config.listen_addr);
    server.add_service(proxy);

    println!(
        "Pingora ERPNext proxy listening on http://{}",
        config.listen_addr
    );
    println!(
        "Routes: /socket.io -> http://{}, everything else -> http://{}",
        config.socketio_upstream, config.web_upstream
    );
    println!("ERPNext site host: {}", config.site_host);

    server.run_forever();
}
