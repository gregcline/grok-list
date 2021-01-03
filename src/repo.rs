use bson::{Bson, oid::ObjectId};
use mongodb::{Client, Database, bson, bson::doc, error::Error as MongoDbError};
use color_eyre::Result;
use thiserror::Error;
use serde::{Serialize, de::DeserializeOwned};
use super::list::List;
use super::store::Store;

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("mongo returned something other than ObjectId for inserted_id")]
    NotObjectId,
    #[error("mongo returned an error: {0:?}")]
    MongoError(#[from] MongoDbError),
    #[error("could not serialize to bson")]
    BsonSer(#[from] bson::ser::Error),
    #[error("could not deserialize from bson")]
    BsonDe(#[from] bson::de::Error),
}

#[derive(Debug)]
enum Collections {
    Lists,
    Stores
}

impl std::fmt::Display for Collections {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Collections::Lists => write!(f, "lists"),
            Collections::Stores => write!(f, "stores"),
        }
    }
}

pub struct Repo {
    data_store: Database,
}

impl Repo {
    pub async fn new(conn_str: &str) -> Result<Self, RepoError> {
        let client = Client::with_uri_str(conn_str).await?.database("grok_list");
        Ok(Repo {
            data_store: client,
        })
    }

    async fn add_document<T: Serialize + DeserializeOwned>(&self, document: &T, collection_name: &Collections) -> Result<Option<T>, RepoError> {
        let collection = self.data_store.collection(&collection_name.to_string());

        let insert_result = collection.insert_one(bson::to_document(&document)?, None).await?;
        let document_id = match insert_result.inserted_id {
            Bson::ObjectId(id) => Ok(id),
            _ => Err(RepoError::NotObjectId),
        }?;
        let inserted_document = self.get_document_by_id::<T>(&document_id, collection_name).await?;
        Ok(inserted_document)
    }

    async fn get_document_by_id<T: Serialize + DeserializeOwned>(&self, id: &ObjectId, collection: &Collections) -> Result<Option<T>, RepoError> {
        let collection = self.data_store.collection(&collection.to_string());
        let document = collection
            .find_one(doc! { "_id": id }, None)
            .await?
            .map(bson::from_document)
            .transpose()?;

        Ok(document)
    }

    async fn delete_document_by_id(&self, id: &ObjectId, collection: &Collections) -> Result<i64, RepoError> {
        let collection = self.data_store.collection(&collection.to_string());
        let delete_result = collection.delete_one( doc! { "_id": id }, None)
            .await?;
        Ok(delete_result.deleted_count)
    }

    pub async fn add_list(&self, list: &List) -> Result<Option<List>, RepoError> {
        self.add_document(list, &Collections::Lists).await
    }

    pub async fn get_list_by_id(&self, id: &ObjectId) -> Result<Option<List>, RepoError> {
        self.get_document_by_id(id, &Collections::Lists).await
    }

    pub async fn delete_list_by_id(&self, id: &ObjectId) -> Result<i64, RepoError> {
        self.delete_document_by_id(id, &Collections::Lists).await
    }

    pub async fn add_store(&self, store: &Store) -> Result<Option<Store>, RepoError> {
        self.add_document(store, &Collections::Stores).await
    }

    pub async fn get_store_by_id(&self, id: &ObjectId) -> Result<Option<Store>, RepoError> {
        self.get_document_by_id(id, &Collections::Stores).await
    }

    pub async fn delete_store_by_id(&self, id: &ObjectId) -> Result<i64, RepoError> {
        self.delete_document_by_id(id, &Collections::Stores).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::list::ListItem;
    use mongodb::bson::oid::ObjectId;

    const MONGO_URI: &str = "mongodb://localhost:27017/";

    #[derive(Error, Debug)]
    enum TestError {
        #[error("got None when reading our writes from mongo")]
        NoneFromMongo,
    }

    #[tokio::test]
    async fn can_insert_and_retrieve_lists_by_id() -> Result<()> {
        let repo = Repo::new(MONGO_URI).await.expect("Couldn't connect to mongo, is it running?");
        let list_item = ListItem::builder("salmon")
            .category("meat")
            .amount("2lb")
            .build();
        let list = List::builder(ObjectId::new())
            .add_item(list_item)
            .build();

        let inserted_list = repo.add_list(&list)
            .await?.ok_or(TestError::NoneFromMongo)?;
        let retrieved = repo.get_list_by_id(&inserted_list._id.clone().expect("Inserted list had no _id"))
            .await
            .expect("Error finding list")
            .unwrap_or_else(|| panic!("List with id: {:?} did not exist", inserted_list._id.clone()));

        assert_eq!(retrieved.user_id, list.user_id);
        assert_eq!(retrieved.items, list.items);

        let items_deleted = repo.delete_list_by_id(&inserted_list._id.expect("Inserted list had no id")).await?;
        assert_eq!(1, items_deleted);

        Ok(())
    }

    #[tokio::test]
    async fn can_insert_and_retrieve_categories_by_id() -> Result<()> {
        let repo = Repo::new(MONGO_URI).await.expect("Couldn't connect to mongo, is it running?");
        let mut store = Store::new("test_store");
        store.add_category("MEAT");
        store.add_category("Produce");

        let inserted_store = repo.add_store(&store)
            .await?.ok_or(TestError::NoneFromMongo)?;
        let retrieved = repo.get_store_by_id(&inserted_store._id.clone().expect("Inserted store had no _id"))
            .await
            .expect("Error finding store")
            .unwrap_or_else(|| panic!("Store with id: {:?} did not exist", inserted_store._id.clone()));

        assert_eq!(retrieved.name, store.name);
        assert_eq!(retrieved.categories, store.categories);

        let stores_deleted = repo.delete_store_by_id(&inserted_store._id.expect("Inserted store had no _id")).await?;
        assert_eq!(1, stores_deleted);

        Ok(())
    }
}
