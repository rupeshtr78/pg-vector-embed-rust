use crate::app::config::{EmbedRequest, EmbedResponse, NewArcEmbedRequest, VectorDbConfig};
use crate::app::constants::EMBEDDING_URL;
use crate::vectordb;
use log::{debug, error, info};
use postgres::Client;
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};

use super::vector_embedding;

/// Run the embedding request and load the embeddings into the database
/// Arguments:
/// - rt: &tokio::runtime::Runtime
/// - embed_model: String
/// - input_list: &Vec<String>
/// - vector_table: String
/// - dimension: String
/// - db_config: VectorDbConfig
/// Returns:
/// - Result<JoinHandle<()>, Box<dyn Error>>
pub fn run_embedding_load(
    rt: &tokio::runtime::Runtime,
    embed_model: String,
    input_list: &Vec<String>,
    vector_table: String,
    dimension: String,
    db_config: VectorDbConfig,
) -> Result<JoinHandle<()>, Box<dyn Error>> {
    debug!("Starting Loading Embeddings");
    let url = EMBEDDING_URL;

    // Arc (Atomic Reference Counted) pointer. It is a thread-safe reference-counting pointer.
    let embed_request_arc = NewArcEmbedRequest(&embed_model, input_list);
    let embed_request_arc_clone = Arc::clone(&embed_request_arc);

    // Run embedding request in a separate thread
    let embed_response = rt.block_on(fetch_embedding(&url, &embed_request_arc));

    let dim = dimension.parse::<i32>().unwrap_or_else(|_| {
        error!("Failed to parse dimension");
        0
    });

    // load the embeddings in a separate thread
    let embed_thread = thread::Builder::new().name("embedding_thread".to_owned());

    let run_embed_thread = embed_thread.spawn(move || {
        let mut client = match vectordb::pg_vector::pg_client(&db_config) {
            Ok(client) => client,
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        };
        let embed_data = match embed_request_arc_clone.read() {
            Ok(data) => data,
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        };

        match persist_embedding_data(
            &mut client,
            &vector_table,
            dim,
            &embed_data,
            &embed_response.embeddings,
        ) {
            Ok(_) => {}
            Err(e) => {
                error!("Error: {}", e);
            }
        }

        if let Err(e) = client.close() {
            error!("Error: {}", e);
            return;
        }
    });

    debug!("Finished Loading Embeddings");
    Ok(run_embed_thread?)
}

/// Fetch the embedding from the embedding service
/// Arguments:
/// - url: &str
/// - embed_data: &Arc<RwLock<EmbedRequest>>
/// Returns:
/// - EmbedResponse
pub async fn fetch_embedding(url: &str, embed_data: &Arc<RwLock<EmbedRequest>>) -> EmbedResponse {
    debug!("Running Embedding");
    let embed_data = match embed_data.read() {
        Ok(data) => data,
        Err(e) => {
            error!("Error: {}", e);
            return EmbedResponse {
                model: "".to_string(),
                embeddings: vec![],
            };
        }
    };

    let response = match vector_embedding::create_embed_request(url, &embed_data).await {
        Ok(embed_response) => embed_response,
        Err(e) => {
            error!("Error: {}", e);
            return EmbedResponse {
                model: "".to_string(),
                embeddings: vec![],
            };
        }
    };

    debug!("Finished Running Embedding");
    response
}

/// Persist the embedding data into the database
/// Arguments:
/// - pg_client: &mut Client
/// - table: &String
/// - dimension: i32
/// - embed_request: &EmbedRequest
/// - embeddings: &Vec<Vec<f32>>
/// Returns:
/// - Result<(), Box<dyn Error>>
pub fn persist_embedding_data(
    pg_client: &mut Client,
    table: &String,
    dimension: i32,
    embed_request: &EmbedRequest,
    embeddings: &Vec<Vec<f32>>,
) -> Result<(), Box<dyn Error>> {
    info!("Loading data into table");
    match vectordb::pg_vector::create_table(pg_client, &table, dimension) {
        Ok(_) => {
            debug!("Create table successful");
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    }

    match vectordb::pg_vector::load_vector_data(pg_client, &table, &embed_request, embeddings) {
        Ok(_) => {
            debug!("Load vector data successful");
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    }

    Ok(())
}