use std::collections::HashMap;
use std::path::PathBuf;

use tantivy::schema::{Field, Schema};
use tantivy::Index;

use std::sync::atomic::Ordering;

use crate::config::Config;
use crate::engine::merge_policy::TargetDocCountMergePolicy;
use crate::engine::query::subfield_routes_from_field_map;
use crate::engine::schema::{build_index_settings, build_schema, schema_to_definition};
use crate::engine::searcher::SearchEngine;
use crate::engine::tokenizers::register_custom_tokenizers;
use crate::engine::writer::{WriterConfig, WriterHandle};
use crate::protocol::messages::{
    CreateIndexResponse, DeleteIndexResponse, GetIndexResponse, GetSegmentsResponse, IndexInfo,
    ListIndexesResponse, SchemaDefinition,
};

pub struct ManagedIndex {
    pub name: String,
    pub schema: Schema,
    pub schema_def: SchemaDefinition,
    pub field_map: HashMap<String, Field>,
    pub subfield_routes: HashMap<String, Field>,
    pub index: Index,
    pub writer: WriterHandle,
    pub searcher: SearchEngine,
}

pub struct IndexManager {
    indexes: HashMap<String, ManagedIndex>,
    data_dir: PathBuf,
    config: Config,
}

impl IndexManager {
    pub fn new(config: Config) -> Self {
        let data_dir = PathBuf::from(&config.data_dir);
        Self {
            indexes: HashMap::new(),
            data_dir,
            config,
        }
    }

    /// Load existing indexes from the data directory on startup.
    pub fn load_existing(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.data_dir.exists() {
            std::fs::create_dir_all(&self.data_dir)?;
            return Ok(());
        }

        for entry in std::fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            log::info!("Loading existing index: {}", name);

            match self.open_existing_index(&name, &path) {
                Ok(managed) => {
                    self.indexes.insert(name.clone(), managed);
                    log::info!("Loaded index: {}", name);
                }
                Err(e) => {
                    log::error!("Failed to load index '{}': {}", name, e);
                }
            }
        }

        Ok(())
    }

    fn open_existing_index(
        &self,
        name: &str,
        path: &PathBuf,
    ) -> Result<ManagedIndex, Box<dyn std::error::Error>> {
        let index = Index::open_in_dir(path)?;

        // Register custom tokenizers BEFORE creating writer/reader so the
        // schema's tokenizer references resolve at open time.
        register_custom_tokenizers(&index);

        let schema = index.schema();

        // Reconstruct field map from schema (includes internal __sub__ fields).
        let mut field_map = HashMap::new();
        for (field, field_entry) in schema.fields() {
            field_map.insert(field_entry.name().to_string(), field);
        }

        let sub_routes = subfield_routes_from_field_map(&field_map);

        let index_settings = index.settings().clone();
        let schema_def = schema_to_definition(&schema, Some(&index_settings));

        let writer_config = WriterConfig {
            heap_size: self.config.writer_heap_size,
            num_threads: self.config.num_indexing_threads,
            auto_commit_doc_count: self.config.auto_commit_doc_count,
            auto_commit_interval_secs: self.config.auto_commit_interval_secs,
            max_merge_factor: self.config.max_merge_factor,
            merge_target_docs: self.config.merge_target_docs,
            min_num_segments: self.config.min_num_segments,
            index_threads_pct: self.config.index_threads_pct,
            hard_commit_multiplier: self.config.hard_commit_multiplier,
        };

        let writer = WriterHandle::spawn(
            index.clone(),
            schema.clone(),
            field_map.clone(),
            sub_routes.clone(),
            writer_config,
        );

        let searcher = SearchEngine::new(&index, schema.clone(), field_map.clone(), sub_routes.clone())?;

        Ok(ManagedIndex {
            name: name.to_string(),
            schema,
            schema_def,
            field_map,
            subfield_routes: sub_routes,
            index,
            writer,
            searcher,
        })
    }

    pub fn create_index(
        &mut self,
        name: &str,
        schema_def: SchemaDefinition,
    ) -> Result<CreateIndexResponse, Box<dyn std::error::Error>> {
        if self.indexes.contains_key(name) {
            return Err(format!("Index '{}' already exists", name).into());
        }

        let (schema, field_map) = build_schema(&schema_def)?;
        let index_settings = build_index_settings(&schema_def)?;

        // Create directory for the index
        let index_path = self.data_dir.join(name);
        std::fs::create_dir_all(&index_path)?;

        // Create tantivy index with the requested IndexSettings (compression, block size).
        let index = Index::builder()
            .schema(schema.clone())
            .settings(index_settings.clone())
            .create_in_dir(&index_path)?;

        // Register custom tokenizers BEFORE spawning writer/reader.
        register_custom_tokenizers(&index);

        let sub_routes = subfield_routes_from_field_map(&field_map);

        // Re-materialise the schema definition so the response reports the
        // effective settings (including defaults filled in by tantivy).
        let schema_def = schema_to_definition(&schema, Some(&index_settings));

        // Build field_ids for the response (excludes internal fields from count).
        let field_ids: HashMap<String, u32> = field_map
            .iter()
            .filter(|(name, _)| !name.starts_with("__sub__"))
            .map(|(name, field)| (name.clone(), field.field_id()))
            .collect();

        let writer_config = WriterConfig {
            heap_size: self.config.writer_heap_size,
            num_threads: self.config.num_indexing_threads,
            auto_commit_doc_count: self.config.auto_commit_doc_count,
            auto_commit_interval_secs: self.config.auto_commit_interval_secs,
            max_merge_factor: self.config.max_merge_factor,
            merge_target_docs: self.config.merge_target_docs,
            min_num_segments: self.config.min_num_segments,
            index_threads_pct: self.config.index_threads_pct,
            hard_commit_multiplier: self.config.hard_commit_multiplier,
        };

        let mut writer = WriterHandle::spawn(
            index.clone(),
            schema.clone(),
            field_map.clone(),
            sub_routes.clone(),
            writer_config,
        );

        // Restart with the custom merge policy.
        writer.shutdown();

        let writer = WriterHandle::spawn_with_merge_policy(
            index.clone(),
            schema.clone(),
            field_map.clone(),
            sub_routes.clone(),
            WriterConfig {
                heap_size: self.config.writer_heap_size,
                num_threads: self.config.num_indexing_threads,
                auto_commit_doc_count: self.config.auto_commit_doc_count,
                auto_commit_interval_secs: self.config.auto_commit_interval_secs,
                max_merge_factor: self.config.max_merge_factor,
                merge_target_docs: self.config.merge_target_docs,
                min_num_segments: self.config.min_num_segments,
                index_threads_pct: self.config.index_threads_pct,
                hard_commit_multiplier: self.config.hard_commit_multiplier,
            },
            TargetDocCountMergePolicy {
                target_num_docs: self.config.merge_target_docs,
                max_merge_factor: self.config.max_merge_factor,
                min_num_segments: self.config.min_num_segments.max(2),
            },
        );

        let searcher = SearchEngine::new(&index, schema.clone(), field_map.clone(), sub_routes.clone())?;

        let managed = ManagedIndex {
            name: name.to_string(),
            schema,
            schema_def,
            field_map,
            subfield_routes: sub_routes,
            index,
            writer,
            searcher,
        };

        let field_count = managed.field_map.len();
        let compression = managed.schema_def.compression.clone().unwrap_or_else(|| "lz4".into());
        let block_size = managed.schema_def.block_size.unwrap_or(16384);
        self.indexes.insert(name.to_string(), managed);

        log::info!(
            "Created index '{}' — {} fields, compression={}, block_size={} B",
            name, field_count, compression, block_size
        );

        Ok(CreateIndexResponse {
            success: true,
            field_ids,
        })
    }

    pub fn delete_index(
        &mut self,
        name: &str,
    ) -> Result<DeleteIndexResponse, Box<dyn std::error::Error>> {
        match self.indexes.remove(name) {
            Some(managed) => {
                let total_docs = managed.writer.total_docs_ingested.load(Ordering::Relaxed);
                // Drop ManagedIndex — reader FDs are closed immediately; the
                // writer thread exits on its own when the channel disconnects.
                drop(managed);
                let index_path = self.data_dir.join(name);
                if index_path.exists() {
                    std::fs::remove_dir_all(&index_path)?;
                }
                log::info!("Deleted index '{}' ({} docs ingested)", name, total_docs);
                Ok(DeleteIndexResponse { success: true })
            }
            None => Err(format!("Index '{}' not found", name).into()),
        }
    }

    pub fn list_indexes(&self) -> ListIndexesResponse {
        let indexes = self
            .indexes
            .values()
            .map(|managed| IndexInfo {
                name: managed.name.clone(),
                doc_count: managed.searcher.num_docs(),
                num_segments: managed.searcher.num_segments(),
                pending_docs: managed.writer.pending_docs.load(Ordering::Relaxed),
                search_count: managed.searcher.search_count.load(Ordering::Relaxed),
                search_latency_us: managed.searcher.search_latency_us.load(Ordering::Relaxed),
                total_docs_ingested: managed.writer.total_docs_ingested.load(Ordering::Relaxed),
                raw_bytes_ingested: managed.writer.raw_bytes_ingested.load(Ordering::Relaxed),
            })
            .collect();

        ListIndexesResponse { indexes }
    }

    pub fn get_index(
        &self,
        name: &str,
    ) -> Result<GetIndexResponse, Box<dyn std::error::Error>> {
        match self.indexes.get(name) {
            Some(managed) => Ok(GetIndexResponse {
                name: managed.name.clone(),
                schema: managed.schema_def.clone(),
                doc_count: managed.searcher.num_docs(),
                num_segments: managed.searcher.num_segments(),
                pending_docs: managed.writer.pending_docs.load(Ordering::Relaxed),
                search_count: managed.searcher.search_count.load(Ordering::Relaxed),
                search_latency_us: managed.searcher.search_latency_us.load(Ordering::Relaxed),
                total_docs_ingested: managed.writer.total_docs_ingested.load(Ordering::Relaxed),
                raw_bytes_ingested: managed.writer.raw_bytes_ingested.load(Ordering::Relaxed),
            }),
            None => Err(format!("Index '{}' not found", name).into()),
        }
    }

    pub fn get_segments(
        &self,
        name: &str,
    ) -> Result<GetSegmentsResponse, Box<dyn std::error::Error>> {
        match self.indexes.get(name) {
            Some(managed) => {
                let segments = managed.searcher.segment_info()?;
                Ok(GetSegmentsResponse { segments })
            }
            None => Err(format!("Index '{}' not found", name).into()),
        }
    }

    pub fn get_managed_index(
        &self,
        name: &str,
    ) -> Result<&ManagedIndex, Box<dyn std::error::Error>> {
        self.indexes
            .get(name)
            .ok_or_else(|| format!("Index '{}' not found", name).into())
    }

    #[allow(dead_code)]
    pub fn get_managed_index_mut(
        &mut self,
        name: &str,
    ) -> Result<&mut ManagedIndex, Box<dyn std::error::Error>> {
        self.indexes
            .get_mut(name)
            .ok_or_else(|| format!("Index '{}' not found", name).into())
    }

    /// Drop all indexes: closes all readers (→Fds freed) and disconnects
    /// writer channels (→writer threads exit on their own). Does **not** join
    /// writer threads — that would block if a writer is stuck flushing under
    /// EMFILE. The OS will clean them up on process exit.
    pub fn shutdown(&mut self) {
        for (name, managed) in self.indexes.drain() {
            let total = managed.writer.total_docs_ingested.load(Ordering::Relaxed);
            log::info!("Shut down index '{}' ({} docs ingested)", name, total);
        }
    }
}
