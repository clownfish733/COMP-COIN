use std::{ 
    path::PathBuf, sync::{Arc, 
        atomic::{AtomicBool, Ordering}
    }
};

use log::{info, warn};

use tokio::{
    sync::{RwLock, mpsc},
    net::TcpListener,
};

use anyhow::Result;

use crate::node::{
    Node,
    NetworkCommand
};

use axum::{
    Router,
    Json,
    response::Html,
    routing::{get, post},
    extract::State,
};

use super::api_messages::{AddressBook, TransactionRequest, TransactionResponse, UserStatus, NodeStatus};

use tower_http::services::ServeDir;

pub async fn start_ui_server(
    node: Arc<RwLock<Node>>,
    network_tx: mpsc::Sender<NetworkCommand>,
    save: Arc<AtomicBool>,
) -> Result<()>{

    info!("Started UI Server");

    let static_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/ui/static");

    let state = AppState{
        node,
        network_tx,
        save,
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/api/transaction", post(submit_transaction))
        .route("/api/node_status", get(get_node_status))
        .route("/api/user_status", get(get_user_status))
        .route("/api/address_book", get(get_address_book))
        .route("/api/address_book", post(save_address_book))
        .route("/api/save_check", get(check_save_request))
        .nest_service("/static", ServeDir::new(static_dir))
        .with_state(state);

    let addr= "0.0.0.0:3000";

    let listener = TcpListener::bind(addr).await?;

    let url = format!("http://127.0.0.1:3000");
    info!("Web UI running");

    if let Err(e) = webbrowser::open(&url){
        warn!("Failed to open browser: {}",e);
    }

    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
struct AppState{
    node: Arc<RwLock<Node>>,
    network_tx: mpsc::Sender<NetworkCommand>,
    save: Arc<AtomicBool>
}

    
async fn check_save_request(State(state): State<AppState>) -> Json<serde_json::Value>{
    let should_save = state.save.swap(false, Ordering::SeqCst);
    Json(serde_json::json!({"save": should_save }))
}

async fn get_address_book() -> Json<AddressBook>{
    Json(AddressBook::load())
}

async fn save_address_book(
    Json(address_book): Json<AddressBook>
) -> Json<serde_json::Value>{
    address_book.save();
    Json(serde_json::json!({"success": true}))
}


async fn submit_transaction(
    State(state): State<AppState>, 
    Json(req): Json<TransactionRequest>
) -> Json<TransactionResponse>{
    req.log();

    if req.calculate_total_spend() > state.node.read().await.wallet.get_funds(){
        return Json(TransactionResponse {
            success: false,
            message: "Insufficient Funds".to_string()
        })
    }
    
    let Ok(outputs) = req.get_outputs()else{
        return Json(TransactionResponse {
            success: false,
            message: "Contains invalid addressess".to_string()
        })
    };

    let transaction = {
        let node_read = state.node.read().await;
        node_read.wallet.new_transaction(node_read.get_version(), outputs, req.fee)
    };

    if let Err(e) = state.network_tx.send(NetworkCommand::Transaction(transaction)).await{
        warn!("Error Sending network command: {}", e);
    };

    Json(TransactionResponse { 
        success: true, 
        message: "Transaction being broadcasted".to_string()
    })
}

async fn get_node_status(State(state): State<AppState>) -> Json<NodeStatus>{
    Json(state.node.read().await.get_node_status())
}

async fn get_user_status(State(state): State<AppState>) -> Json<UserStatus>{
    Json(state.node.read().await.get_user_status())
}

async fn index() -> Html<&'static str>{
    Html(include_str!("static/index.html"))
}