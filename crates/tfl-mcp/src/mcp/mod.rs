//! MCP server for TfL.
//!
//! Two tools, following the pattern of its sibling projects:
//! - `tfl_schema` — the SDL and the rules for using it
//! - `tfl` — executes a query
//!
//! The schema is several thousand tokens, so it stays behind a tool call rather
//! than riding in the always-loaded tool descriptions: a session that never
//! mentions a train should pay close to nothing for having this connected.

pub mod graphql;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, PoisonError},
};

use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, ContentBlock, Implementation, ProtocolVersion, ServerCapabilities,
        ServerInfo,
    },
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use tfl_api_client::{Client, Config};

type ToolResult = std::result::Result<CallToolResult, McpError>;

/// Header carrying the caller's TfL `app_key` in HTTP transport mode.
///
/// A trusted upstream — the hosted gateway, after authenticating the user —
/// sets this before proxying. Over stdio it is absent and the configured key is
/// used instead.
///
/// Unlike its sibling servers, a missing key is not an error: TfL answers
/// anonymous callers at 50 requests/minute. A key raises that to 500.
pub const APP_KEY_HEADER: &str = "x-tfl-app-key";

#[derive(Clone)]
pub struct TflMcp {
    schema: Arc<graphql::TflSchema>,
    /// Clients keyed by app key. A client owns a connection pool, so building
    /// one per request would throw away every keep-alive connection.
    clients: Arc<Mutex<HashMap<Option<String>, Arc<Client>>>>,
    /// Key used when a request carries no [`APP_KEY_HEADER`]. Always set over
    /// stdio if the user configured one; `None` in a hosted deployment, where
    /// each request brings its own.
    default_app_key: Option<String>,
    #[allow(dead_code)] // referenced by #[tool_handler] macro expansion
    tool_router: ToolRouter<Self>,
}

/// A GraphQL request from the model.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GraphqlRequest {
    /// The GraphQL query.
    pub query: String,
    /// Variables, as a JSON object encoded in a string.
    pub variables: Option<String>,
}

impl TflMcp {
    pub fn new(default_app_key: Option<String>) -> Self {
        Self {
            schema: Arc::new(graphql::schema()),
            clients: Arc::new(Mutex::new(HashMap::new())),
            default_app_key: default_app_key
                .map(|k| k.trim().to_string())
                .filter(|k| !k.is_empty()),
            tool_router: Self::tool_router(),
        }
    }

    fn headers<'a>(&self, ctx: &'a RequestContext<RoleServer>) -> Option<&'a http::HeaderMap> {
        ctx.extensions
            .get::<http::request::Parts>()
            .map(|p| &p.headers)
    }

    /// The key for this request: the header if there is one, else the
    /// configured default, else none at all.
    fn app_key(&self, ctx: &RequestContext<RoleServer>) -> Option<String> {
        self.headers(ctx)
            .and_then(|h| h.get(APP_KEY_HEADER))
            .and_then(|v| v.to_str().ok())
            .map(str::trim)
            .filter(|k| !k.is_empty())
            .map(str::to_string)
            .or_else(|| self.default_app_key.clone())
    }

    fn client_for(&self, app_key: Option<String>) -> Arc<Client> {
        if let Some(existing) = self
            .clients
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&app_key)
        {
            return existing.clone();
        }
        // Construction does no I/O, so building one under the lock on a race is
        // harmless; `or_insert` keeps whichever arrived first.
        let client = Arc::new(
            Client::new(Config {
                app_key: app_key.clone(),
                cache: crate::cache_from_env(),
                ..Default::default()
            })
            .expect("client construction cannot fail with a default config"),
        );
        self.clients
            .lock()
            .unwrap()
            .entry(app_key)
            .or_insert(client)
            .clone()
    }

    fn text_result(text: impl Into<String>) -> ToolResult {
        Ok(CallToolResult::success(vec![ContentBlock::text(
            text.into(),
        )]))
    }

    fn error_result(msg: impl Into<String>) -> ToolResult {
        Ok(CallToolResult::error(vec![ContentBlock::text(msg.into())]))
    }
}

#[tool_router]
impl TflMcp {
    #[tool(
        name = "tfl_schema",
        title = "TfL Schema",
        description = "The London transport API's GraphQL schema. Call once before the first `tfl` query."
    )]
    async fn tfl_schema(&self) -> ToolResult {
        Self::text_result(graphql::sdl())
    }

    #[tool(
        name = "tfl",
        title = "TfL",
        description = "Execute a GraphQL query against London transport (TfL). Use for anything about getting around London or conditions in it: live arrivals and departures; tube/bus/DLR/Overground/Elizabeth line status and disruptions; stations and stops; journey planning between any two places with times, fares and step-free routing; Santander Cycles docking stations; road traffic and closures on the A406, A2 and other major roads; air quality and pollution forecasts; licensed minicab and taxi operators; electric-vehicle charge points; car parks; and road collision history. Get the schema from `tfl_schema` first. Variables are a JSON string."
    )]
    async fn tfl(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(req): Parameters<GraphqlRequest>,
    ) -> ToolResult {
        let client = self.client_for(self.app_key(&ctx));
        let mut request = graphql::request(&req.query, client);

        if let Some(ref vars) = req.variables {
            match serde_json::from_str::<serde_json::Value>(vars) {
                Ok(serde_json::Value::Object(map)) => {
                    request = request.variables(async_graphql::Variables::from_json(
                        serde_json::Value::Object(map),
                    ));
                }
                Ok(_) => return Self::error_result("Variables must be a JSON object"),
                Err(e) => return Self::error_result(format!("Invalid variables JSON: {e}")),
            }
        }

        let response = self.schema.execute(request).await;
        let json = serde_json::to_string_pretty(&response)
            .unwrap_or_else(|e| format!("{{\"error\": \"Serialization failed: {e}\"}}"));
        Self::text_result(json)
    }
}

#[tool_handler]
impl ServerHandler for TflMcp {
    fn get_info(&self) -> ServerInfo {
        let server_info = Implementation::new("tfl", env!("CARGO_PKG_VERSION"))
            .with_title("TfL MCP Server")
            .with_website_url("https://github.com/radiosilence/tfl-mcp");

        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::LATEST)
            .with_server_info(server_info)
            // Terse on purpose: this is loaded into every session, most of
            // which never mention a train. The schema, and the reason batching
            // matters, are a tool call away.
            .with_instructions(
                "London transport, as a GraphQL API. Read the schema once with \
                 `tfl_schema`, then query with `tfl`. Variables go as a JSON string.\n\n\
                 Covers live arrivals, line status and disruptions, stations and the \
                 stops on a line in travel order, journey planning with times, fares \
                 and step-free routing, Santander Cycles, road traffic and closures, \
                 air quality, minicab operators, EV charge points, car parks and \
                 road collision history. Reach for it for anything about getting \
                 around London or conditions in it — \"how do I get to\", \"is the \
                 tube working\", \"where can I get a bike\", \"how is the traffic on \
                 the A406\", \"is the air bad today\", \"get me a cab\".\n\n\
                 `now` gives the time in London and costs no request. TfL's own \
                 timestamps carry no timezone, so read it before deciding whether a \
                 departure is soon.\n\n\
                 The graph is joined on TfL's own foreign keys and everything below a \
                 list is batched, so ask for what you need in ONE query rather than \
                 looping: selecting `line { name }` across sixty arrivals costs one \
                 extra request, not sixty. Field descriptions say what each one costs.\n\n\
                 Arrivals are live and go stale within a minute or two — re-query \
                 rather than reusing an earlier answer, and give times as \
                 \"in N minutes\" rather than a clock time unless asked.\n\n\
                 Journeys accept place names, but TfL calls most of them ambiguous \
                 and answers with candidates instead of routes — \"Kings Cross\" does. \
                 Select `isAmbiguous` and `fromOptions`/`toOptions` alongside \
                 `journeys` so one query tells you which happened, then re-query with \
                 an option's `value`. Stations are listed first and are usually what \
                 was meant, even when a shop of the same name scores higher.",
            )
    }
}

/// Which surfaces [`run_http_server`] should expose.
#[derive(Debug, Clone, Copy)]
pub struct HttpSurfaces {
    pub mcp: bool,
    pub graphql: bool,
    pub graphiql: bool,
    pub browser: bool,
}

/// Runs the MCP server over stdio.
pub async fn run_server(app_key: Option<String>) -> anyhow::Result<()> {
    use rmcp::{ServiceExt, transport::stdio};

    let service = TflMcp::new(app_key);
    let server = service
        .serve(stdio())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start MCP server: {e}"))?;
    server.waiting().await?;
    Ok(())
}

/// Runs the MCP server, and optionally GraphQL and GraphiQL, over HTTP.
///
/// A request's own `X-Tfl-App-Key` header always wins; the configured key is
/// the fallback. Do **not** expose this to the internet without a trusted layer
/// in front: the header is trusted unconditionally.
pub async fn run_http_server(
    addr: &str,
    app_key: Option<String>,
    surfaces: HttpSurfaces,
) -> anyhow::Result<()> {
    use rmcp::transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    };

    let mcp = TflMcp::new(app_key);
    let mut router = axum::Router::new();

    if surfaces.mcp {
        // Disable rmcp's DNS-rebinding Host allowlist: this transport runs
        // behind a trusted reverse proxy, which forwards an internal Host that
        // the default allowlist would reject with 403. Rebinding protection
        // guards browsers hitting a localhost MCP directly, which is not this.
        let config = StreamableHttpServerConfig::default().disable_allowed_hosts();
        let service = StreamableHttpService::new(
            {
                let template = mcp.clone();
                move || Ok(template.clone())
            },
            Arc::new(LocalSessionManager::default()),
            config,
        );
        router = router.nest_service("/mcp", service);
        tracing::info!("MCP streamable-HTTP listening on http://{addr}/mcp");
    }

    if surfaces.graphql || surfaces.graphiql {
        router = router.route("/graphql", axum::routing::post(graphql_endpoint));
        tracing::info!("GraphQL endpoint on http://{addr}/graphql");
    }

    if surfaces.graphiql {
        let ide = askama::Template::render(&GraphiqlPage {
            title: "TfL GraphQL",
            endpoint: "/graphql",
        })?;
        router = router.route(
            "/",
            axum::routing::get(move || {
                let ide = ide.clone();
                async move { axum::response::Html(ide) }
            }),
        );
        tracing::info!("GraphiQL IDE on http://{addr}/");
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Only once the listener is bound, so the browser cannot beat us to it.
    if surfaces.browser {
        let url = format!("http://{addr}/");
        if let Err(e) = open::that_detached(&url) {
            tracing::warn!("Could not open a browser at {url}: {e}");
        }
    }

    axum::serve(listener, router.with_state(mcp))
        .await
        .map_err(|e| anyhow::anyhow!("HTTP server error: {e}"))?;
    Ok(())
}

#[derive(askama::Template)]
#[template(path = "graphiql.html")]
struct GraphiqlPage {
    title: &'static str,
    endpoint: &'static str,
}

async fn graphql_endpoint(
    axum::extract::State(mcp): axum::extract::State<TflMcp>,
    headers: http::HeaderMap,
    axum::Json(req): axum::Json<async_graphql::Request>,
) -> axum::Json<async_graphql::Response> {
    let app_key = headers
        .get(APP_KEY_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .map(str::to_string)
        .or_else(|| mcp.default_app_key.clone());

    let client = mcp.client_for(app_key);
    let request = req
        .data(graphql::loaders::Loaders::new(client.clone()))
        .data(client);
    axum::Json(mcp.schema.execute(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_configured_keys_are_ignored() {
        // An unset environment variable arrives as an empty string, and sending
        // TfL an empty key is worse than sending none.
        for key in [Some("".to_string()), Some("   ".to_string()), None] {
            assert!(TflMcp::new(key).default_app_key.is_none());
        }
        assert_eq!(
            TflMcp::new(Some(" abc ".to_string())).default_app_key,
            Some("abc".to_string())
        );
    }

    #[test]
    fn clients_are_reused_per_key() {
        let mcp = TflMcp::new(None);
        let anonymous = mcp.client_for(None);
        assert!(Arc::ptr_eq(&anonymous, &mcp.client_for(None)));
        assert!(!Arc::ptr_eq(&anonymous, &mcp.client_for(Some("k".into()))));
    }
}
