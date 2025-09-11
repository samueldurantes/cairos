use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::Result,
    lsp_types::{
        DidChangeTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
        InitializeParams, InitializeResult, InitializedParams, MessageType, ServerCapabilities,
        ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind,
    },
};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Event {
    uri: String,
    is_write: bool,
    language: Option<String>,
    line_number: Option<i32>,
    cursor_pos: Option<i32>,
}

pub struct CurrentFile {
    uri: String,
    timestamp: time::OffsetDateTime,
}

struct CairosLanguangeServer {
    client: Client,
    http_client: reqwest::Client,
    base_url: String,
    api_token: String,
    current_file: Mutex<CurrentFile>,
}

impl CairosLanguangeServer {
    async fn send(&self, event: Event) {
        let now = time::OffsetDateTime::now_utc();
        let interval = time::Duration::minutes(2);
        let mut current_file = self.current_file.lock().await;

        if event.uri == current_file.uri
            && now - current_file.timestamp < interval
            && !event.is_write
        {
            return;
        }

        if let Err(e) = crate::clients::cairos::send_events(
            &self.http_client,
            &self.base_url,
            &self.api_token,
            crate::clients::cairos::SendEventsParams {
                uri: event.uri.clone(),
                is_write: event.is_write,
                language: event.language,
                line_number: event.line_number,
                cursor_pos: event.cursor_pos,
            },
        )
        .await
        {
            self.client
                .log_message(
                    MessageType::ERROR,
                    format!("Error when trying to send events: {e:?}"),
                )
                .await;
        }

        current_file.uri = event.uri.to_owned();
        current_file.timestamp = now;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for CairosLanguangeServer {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: NAME.to_owned(),
                version: Some(VERSION.to_owned()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Cairos language server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let event = Event {
            uri: params.text_document.uri[url::Position::BeforeUsername..].to_owned(),
            is_write: false,
            language: Some(params.text_document.language_id),
            line_number: None,
            cursor_pos: None,
        };

        self.send(event).await
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let event = Event {
            uri: params.text_document.uri[url::Position::BeforeUsername..].to_owned(),
            is_write: false,
            language: None,
            line_number: params
                .content_changes
                .first()
                .map_or_else(|| None, |c| c.range)
                .map(|c| i32::try_from(c.start.line).unwrap_or(0)),
            cursor_pos: params
                .content_changes
                .first()
                .map_or_else(|| None, |c| c.range)
                .map(|c| i32::try_from(c.start.character).unwrap_or(0)),
        };

        self.send(event).await
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let event = Event {
            uri: params.text_document.uri[url::Position::BeforeUsername..].to_owned(),
            is_write: true,
            language: None,
            line_number: None,
            cursor_pos: None,
        };

        self.send(event).await
    }
}

pub async fn run(http_client: reqwest::Client, base_url: &str, api_token: &str) {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|client| {
        Arc::new(CairosLanguangeServer {
            client,
            http_client,
            base_url: base_url.to_owned(),
            api_token: api_token.to_owned(),
            current_file: Mutex::new(CurrentFile {
                uri: String::new(),
                timestamp: time::OffsetDateTime::now_utc(),
            }),
        })
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
