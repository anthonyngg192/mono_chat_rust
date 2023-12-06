use async_std::stream::StreamExt;
use bson::{doc, from_document, Document};

// use super::super::MongoDb;
use crate::{
    models::stats::{Index, Stats},
    traits::AbstractStats,
    Error, Result,
};
use std::collections::HashMap;

#[async_trait]
impl AbstractStats for super::super::MongoDb {
    async fn generate_stats(&self) -> Result<Stats> {
        let mut indices = HashMap::new();
        let mut coll_stats = HashMap::new();

        let collection_names =
            self.db()
                .list_collection_names(None)
                .await
                .map_err(|_| Error::DatabaseError {
                    operation: "list_collection_names",
                    with: "database",
                })?;

        for collection in collection_names {
            indices.insert(
                collection.to_string(),
                self.col::<Document>(&collection)
                    .aggregate(
                        vec![doc! {
                           "$indexStats": { }
                        }],
                        None,
                    )
                    .await
                    .map_err(|_| Error::DatabaseError {
                        operation: "aggregate",
                        with: "col",
                    })?
                    .filter_map(|s| s.ok())
                    .collect::<Vec<Document>>()
                    .await
                    .into_iter()
                    .filter_map(|doc| from_document(doc).ok())
                    .collect::<Vec<Index>>(),
            );

            coll_stats.insert(
                collection.to_string(),
                self.col::<Document>(&collection)
                    .aggregate(
                        vec![doc! {
                            "$collStats": {
                                "latencyStats": {
                                    "histograms": true
                                },
                                "storageStats": {},
                                "count": {},
                                "queryExecStats": {}
                            }
                        }],
                        None,
                    )
                    .await
                    .map_err(|_| Error::DatabaseError {
                        operation: "aggregate",
                        with: "database",
                    })?
                    .filter_map(|s| s.ok())
                    .collect::<Vec<Document>>()
                    .await
                    .into_iter()
                    .filter_map(|doc| from_document(doc).ok())
                    .next()
                    .ok_or(Error::DatabaseError {
                        operation: "next aggregate",
                        with: "col",
                    })?,
            );
        }

        Ok(Stats {
            indices,
            coll_stats,
        })
    }
}
