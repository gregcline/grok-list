use super::list::{List, ListItem};
use super::store::Store;
use super::user::User;
use bson::{oid::ObjectId, Bson};
use color_eyre::Result;
use futures::stream::StreamExt;
use mongodb::{bson, bson::doc, error::Error as MongoDbError, Client, Database};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use std::fmt;
use tap::prelude::*;

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
    #[error("could not find the specified object: {0:?} in the collection: {1}")]
    ObjectNotFound(ObjectId, Collections)
}

#[derive(Debug, Clone)]
enum Collections {
    Lists,
    Stores,
    Users,
}

impl std::fmt::Display for Collections {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Collections::Lists => write!(f, "lists"),
            Collections::Stores => write!(f, "stores"),
            Collections::Users => write!(f, "users"),
        }
    }
}

pub struct Repo {
    data_store: Database,
}

impl Repo {
    pub async fn new(conn_str: &str) -> Result<Self, RepoError> {
        let client = Client::with_uri_str(conn_str).await?.database("grok_list");
        Ok(Repo { data_store: client })
    }

    async fn add_document<T: Serialize + DeserializeOwned + fmt::Debug>(
        &self,
        document: &T,
        collection_name: &Collections,
    ) -> Result<Option<T>, RepoError> {
        let collection = self.data_store.collection(&collection_name.to_string());

        let insert_result = collection
            .insert_one(bson::to_document(&document)?, None)
            .await?;
        let document_id = match insert_result.inserted_id {
            Bson::ObjectId(id) => Ok(id),
            _ => Err(RepoError::NotObjectId),
        }?;
        let inserted_document = self
            .get_document_by_id::<T>(&document_id, collection_name)
            .await?;
        Ok(inserted_document)
    }

    async fn get_document_by_id<T: Serialize + DeserializeOwned + fmt::Debug>(
        &self,
        id: &ObjectId,
        collection: &Collections,
    ) -> Result<Option<T>, RepoError> {
        let collection = self.data_store.collection(&collection.to_string());
        let document = collection
            .find_one(doc! { "_id": id }, None)
            .await?
            .map(bson::from_document)
            .transpose()?;

        Ok(document)
    }

    async fn replace_document_by_id<T: Serialize + DeserializeOwned + fmt::Debug>(
        &self,
        id: &ObjectId,
        document: &T,
        collection: &Collections,
    ) -> Result<Option<T>, RepoError> {
        let db_collection = self.data_store.collection(&collection.to_string());

        let replace_result = db_collection
            .replace_one(doc! { "_id": id }, bson::to_document(document)?, None)
            .await?;
        self.get_document_by_id(&id, collection).await
    }

    async fn delete_document_by_id(
        &self,
        id: &ObjectId,
        collection: &Collections,
    ) -> Result<i64, RepoError> {
        let collection = self.data_store.collection(&collection.to_string());
        let delete_result = collection.delete_one(doc! { "_id": id }, None).await?;
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

    pub async fn add_user(&self, user: &User) -> Result<Option<User>, RepoError> {
        self.add_document(user, &Collections::Users).await
    }

    pub async fn get_user_by_name(&self, name: &str) -> Result<Option<User>, RepoError> {
        let collection = self.data_store.collection(&Collections::Users.to_string());
        let document = collection
            .find_one(doc! { "name": name}, None)
            .await?
            .map(bson::from_document)
            .transpose()?;

        Ok(document)
    }

    pub async fn get_lists_by_user(
        &self,
        user_id: &ObjectId,
    ) -> Result<Vec<Result<List, RepoError>>, RepoError> {
        let collection = self.data_store.collection(&Collections::Lists.to_string());
        let documents = collection
            .find(doc! { "userId": user_id }, None)
            .await?
            .map(|doc_result| {
                doc_result
                    .map_err(RepoError::from)
                    .and_then(|doc| {
                        bson::from_document::<List>(doc).map_err(RepoError::from)
                    })
            })
            .collect::<Vec<Result<List, RepoError>>>()
            .await;

        Ok(documents)
    }

    pub async fn add_list_item(
        &self,
        list_id: &ObjectId,
        item: &ListItem,
    ) -> Result<Option<List>, RepoError> {
        let mut list = self.get_list_by_id(list_id)
                       .await?
                       .ok_or_else(|| RepoError::ObjectNotFound(list_id.clone(), Collections::Lists))?;
        list.add_item(item.clone());
        self.replace_document_by_id(list_id, &list, &Collections::Lists).await
    }
}

#[cfg(test)]
mod test {
    use super::super::list::ListItem;
    use super::*;
    use mongodb::bson::oid::ObjectId;

    const MONGO_URI: &str = "mongodb://localhost:27017/";

    #[derive(Error, Debug)]
    enum TestError {
        #[error("got None when reading our writes from mongo")]
        NoneFromMongo,
    }

    #[tokio::test]
    async fn can_insert_and_retrieve_lists_by_id() -> Result<()> {
        let repo = Repo::new(MONGO_URI)
            .await
            .expect("Couldn't connect to mongo, is it running?");
        let list_item = ListItem::builder("salmon")
            .category("meat")
            .amount("2lb")
            .build();
        let list = List::builder("test_list".to_string(), ObjectId::new())
            .add_item(list_item)
            .build();

        let inserted_list = repo
            .add_list(&list)
            .await?
            .ok_or(TestError::NoneFromMongo)?;
        let retrieved = repo
            .get_list_by_id(&inserted_list._id.clone().expect("Inserted list had no _id"))
            .await
            .expect("Error finding list")
            .unwrap_or_else(|| {
                panic!(
                    "List with id: {:?} did not exist",
                    inserted_list._id.clone()
                )
            });

        assert_eq!(retrieved.user_id, list.user_id);
        assert_eq!(retrieved.items, list.items);

        let items_deleted = repo
            .delete_list_by_id(&inserted_list._id.expect("Inserted list had no id"))
            .await?;
        assert_eq!(1, items_deleted);

        Ok(())
    }

    #[tokio::test]
    async fn can_insert_and_retrieve_categories_by_id() -> Result<()> {
        let repo = Repo::new(MONGO_URI)
            .await
            .expect("Couldn't connect to mongo, is it running?");
        let mut store = Store::new("test_store");
        store.add_category("MEAT");
        store.add_category("Produce");

        let inserted_store = repo
            .add_store(&store)
            .await?
            .ok_or(TestError::NoneFromMongo)?;
        let retrieved = repo
            .get_store_by_id(
                &inserted_store
                    ._id
                    .clone()
                    .expect("Inserted store had no _id"),
            )
            .await
            .expect("Error finding store")
            .unwrap_or_else(|| {
                panic!(
                    "Store with id: {:?} did not exist",
                    inserted_store._id.clone()
                )
            });

        assert_eq!(retrieved.name, store.name);
        assert_eq!(retrieved.categories, store.categories);

        let stores_deleted = repo
            .delete_store_by_id(&inserted_store._id.expect("Inserted store had no _id"))
            .await?;
        assert_eq!(1, stores_deleted);

        Ok(())
    }

    #[tokio::test]
    async fn can_insert_user() -> Result<()> {
        let repo = Repo::new(MONGO_URI)
            .await
            .expect("Couldn't connect to mongo, is it running?");
        let user = User::new("test_user".to_string(), "test@email.com".to_string());

        let inserted_user = repo
            .add_user(&user)
            .await?
            .ok_or(TestError::NoneFromMongo)?;
        let retrieved = repo
            .get_user_by_name(&user.name)
            .await
            .expect("Error finding store")
            .unwrap_or_else(|| panic!("User with name: {:?} did not exist", user.name));

        assert_eq!(retrieved.name, user.name);
        assert_eq!(retrieved.email, user.email);

        Ok(())
    }

    #[tokio::test]
    async fn can_fetch_lists_by_user() -> Result<()> {
        let repo = Repo::new(MONGO_URI)
            .await
            .expect("Couldn't connect to mongo, is it running?");
        let user = User::new("test_user".to_string(), "test@email.com".to_string());

        let inserted_user = repo
            .add_user(&user)
            .await?
            .ok_or(TestError::NoneFromMongo)?;

        let list_item = ListItem::builder("salmon")
            .category("meat")
            .amount("2lb")
            .build();
        let list = List::builder(
            "test_list".to_string(),
            inserted_user._id.clone().expect("Inserted user had no _id"),
        )
        .add_item(list_item)
        .build();

        let inserted_list = repo
            .add_list(&list)
            .await?
            .ok_or(TestError::NoneFromMongo)?;

        let lists: Vec<List> = repo
            .get_lists_by_user(&inserted_user._id.expect("Inserted user had no _id"))
            .await?
            .into_iter()
            .map(Result::unwrap)
            .collect();

        assert_eq!(lists, vec![inserted_list]);

        Ok(())
    }

    #[tokio::test]
    async fn can_add_items_to_existing_list() -> Result<()> {
        let repo = Repo::new(MONGO_URI)
            .await
            .expect("Couldn't connect to mongo, is it running?");

        let list_item = ListItem::builder("salmon")
            .category("meat")
            .amount("2lb")
            .build();
        let list = List::builder(
            "test_list_add_items".to_string(),
            ObjectId::new()
        )
        .add_item(list_item.clone())
        .build();
        let inserted_list = repo.add_list(&list)
            .await?
            .ok_or(TestError::NoneFromMongo)?;

        let new_list_item = ListItem::builder("brocc")
            .category("veg")
            .amount("1")
            .build();

        let updated_list = repo
            .add_list_item(&inserted_list._id.expect("Inserted list had no _id"), &new_list_item)
            .await.unwrap()
            .ok_or(TestError::NoneFromMongo)?;

        assert_eq!(list.name, updated_list.name);
        assert_eq!(vec![list_item, new_list_item], updated_list.items);

        Ok(())
    }
}
