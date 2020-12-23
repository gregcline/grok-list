use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Store {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    pub name: String,
    pub categories: Vec<String>,
}

impl Store {
    pub fn new(name: &str) -> Self {
        Store {
            _id: None,
            name: name.to_owned(),
            categories: Vec::new(),
        }
    }

    pub fn add_category(&mut self, category: &str) {
        self.categories.push(category.to_lowercase());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn categories_are_made_lowercase_when_added() {
        let mut store = Store::new("test_store");

        store.add_category("Meat");
        store.add_category("PRODUCE");

        assert_eq!(store.categories, vec!["meat", "produce"]);
    }
}
