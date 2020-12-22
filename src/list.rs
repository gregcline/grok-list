use serde::{Serialize, Deserialize};
use mongodb::bson;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct List {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<bson::oid::ObjectId>,
    #[serde(rename(serialize = "userId", deserialize = "userId"))]
    pub user_id: bson::oid::ObjectId,
    pub items: Vec<ListItem>,
}

impl List {
    pub fn builder(user_id: bson::oid::ObjectId) -> ListBuilder {
        ListBuilder::new(user_id)
    }

    pub fn add_item(&mut self, item: &ListItem) {
        self.items.push(item.clone());
    }
}

#[derive(Debug, Clone)]
pub struct ListBuilder {
    pub _id: Option<bson::oid::ObjectId>,
    pub user_id: bson::oid::ObjectId,
    pub items: Vec<ListItem>,
}

impl ListBuilder {
    pub fn new(user_id: bson::oid::ObjectId) -> Self {
        ListBuilder {
            _id: None,
            user_id: user_id,
            items: Vec::new(),
        }
    }

    pub fn build(&self) -> List {
        List {
            _id: self._id.clone(),
            user_id: self.user_id.clone(),
            items: self.items.clone(),
        }
    }

    pub fn add_item<'a>(&'a mut self, item: &ListItem) -> &'a mut Self {
        self.items.push(item.clone());
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ListItem {
    name: String,
    category: Option<String>,
    amount: Option<String>,
}

impl ListItem {
    pub fn builder(name: &str) -> ListItemBuilder {
        ListItemBuilder::new(name)
    }
}

#[derive(Debug, Clone)]
pub struct ListItemBuilder {
    name: String,
    category: Option<String>,
    amount: Option<String>,
}

impl ListItemBuilder {
    pub fn new(name: &str) -> Self {
        ListItemBuilder {
            name: name.to_owned(),
            category: None,
            amount: None,
        }
    }

    pub fn category<'a>(&'a mut self, category: &str) -> &'a mut Self {
        self.category = Some(category.to_lowercase());
        self
    }

    pub fn amount<'a>(&'a mut self, amount: &str) -> &'a mut Self {
        self.amount = Some(amount.to_owned());
        self
    }

    pub fn build(&self) -> ListItem {
        ListItem {
            name: self.name.clone(),
            category: self.category.clone(),
            amount: self.amount.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn category_and_amount_none_by_default() {
        let item = ListItem::builder("broccoli")
            .build();

        assert_eq!(item.name, "broccoli".to_owned());
        assert_eq!(item.category, None);
        assert_eq!(item.amount, None);
    }

    #[test]
    fn category_set_to_lowercase() {
        let item = ListItem::builder("salmon")
            .category("Meat")
            .amount("1")
            .build();
        assert_eq!(item.category, Some("meat".to_owned()));
    }

    #[test]
    fn list_builder_requires_a_user() {
        let user_id = bson::oid::ObjectId::new();
        let list = List::builder(user_id.clone())
            .build();

        assert_eq!(list.user_id, user_id);
    }

    #[test]
    fn list_builder_can_add_items() {
        let user_id = bson::oid::ObjectId::new();
        let item_1 = ListItem::builder("salmon")
            .category("Meat")
            .amount("1")
            .build();
        let item_2 = ListItem::builder("broccoli")
            .category("produce")
            .amount("2")
            .build();
        let item_3 = ListItem::builder("la croix")
            .category("water")
            .amount("3")
            .build();
        let list = List::builder(user_id.clone())
            .add_item(&item_1)
            .add_item(&item_2)
            .add_item(&item_3)
            .build();

        assert_eq!(list.items, vec![item_1, item_2, item_3]);
    }

    #[test]
    fn list_can_add_items() {
        let user_id = bson::oid::ObjectId::new();
        let mut list = List::builder(user_id.clone())
            .build();

        let item_1 = ListItem::builder("salmon")
            .category("Meat")
            .amount("1")
            .build();
        let item_2 = ListItem::builder("broccoli")
            .category("produce")
            .amount("2")
            .build();
        let item_3 = ListItem::builder("la croix")
            .category("water")
            .amount("3")
            .build();

        list.add_item(&item_1);
        list.add_item(&item_2);
        list.add_item(&item_3);

        assert_eq!(list.items, vec![item_1, item_2, item_3]);
    }
}