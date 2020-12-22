use bson::{oid::ObjectId, to_document};
use bson::Bson;
use mongodb::{Client, Database, bson, bson::doc};
use color_eyre::Result;
use thiserror::Error;
use super::list::{List, ListItem};

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("mongo returned something other than ObjectId for inserted_id")]
    NotObjectId,
}

pub struct Repo {
    data_store: Database,
}

impl Repo {
    pub async fn new(conn_str: &str) -> Result<Self> {
        let client = Client::with_uri_str(conn_str).await?.database("grok_list");
        Ok(Repo {
            data_store: client,
        })
    }

    pub async fn add_list(&self, list: &List) -> Result<Option<List>> {
        let collection = self.data_store.collection("lists");

        let insert_result = collection.insert_one(bson::to_document(&list)?, None).await?;
        let list_id = match insert_result.inserted_id {
            Bson::ObjectId(id) => Ok(id),
            _ => Err(RepoError::NotObjectId)
        }?;
        let inserted_list = self.get_list_by_id(&list_id).await?;
        Ok(inserted_list)
    }

    pub async fn get_list_by_id(&self, id: &ObjectId) -> Result<Option<List>> {
        let collection = self.data_store.collection("lists");
        let list = collection
            .find_one(doc! { "_id": id }, None)
            .await?
            .map(bson::from_document)
            .map(Result::ok)
            .flatten();

        Ok(list)
    }

    pub async fn delete_list_by_id(&self, id: &ObjectId) -> Result<i64> {
        let collection = self.data_store.collection("lists");
        let delete_result = collection.delete_one( doc! { "_id": id }, None)
            .await?;
        Ok(delete_result.deleted_count)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio;
    use mongodb::bson::oid::ObjectId;

    #[tokio::test]
    async fn can_insert_and_retrieve_lists_by_id() -> Result<()> {
        let repo = Repo::new("mongodb://localhost:27017/").await.expect("Couldn't connect to mongo, is it running?");
        let list_item = ListItem::builder("salmon")
            .category("meat")
            .amount("2lb")
            .build();
        let list = List::builder(ObjectId::new())
            .add_item(&list_item)
            .build();

        let inserted_list = repo.add_list(&list)
            .await?.ok_or(RepoError::NotObjectId)?;
        let retrieved = repo.get_list_by_id(&inserted_list._id.clone().expect("Inserted list had no _id"))
            .await
            .expect("Error finding list")
            .expect("Couldn't find the list");

        assert_eq!(retrieved.user_id, list.user_id);
        assert_eq!(retrieved.items, list.items);

        let items_deleted = repo.delete_list_by_id(&inserted_list._id.expect("Inserted list had no id")).await?;
        assert_eq!(1, items_deleted);

        Ok(())
    }
}
