use serde::{Serialize, Deserialize};
use mongodb::bson;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct List {
    _id: Option<bson::oid::ObjectId>,
    #[serde(alias = "userId")]
    user_id: bson::oid::ObjectId,
    items: Vec<ListItem>,
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
    _id: Option<bson::oid::ObjectId>,
    user_id: bson::oid::ObjectId,
    items: Vec<ListItem>,
}

impl ListBuilder {
    pub fn new(user_id: bson::oid::ObjectId) -> Self {
        ListBuilder {
            _id: None,
            user_id: user_id,
            items: Vec::new(),
        }
    }

    pub fn build(self) -> List {
        List {
            _id: self._id,
            user_id: self.user_id,
            items: self.items,
        }
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

    pub fn category(self, category: &str) -> Self {
        ListItemBuilder {
            category: Some(category.to_lowercase()),
            ..self
        }
    }

    pub fn amount(self, amount: &str) -> Self {
        ListItemBuilder {
            amount: Some(amount.to_owned()),
            ..self
        }
    }

    pub fn build(self) -> ListItem {
        ListItem {
            name: self.name,
            category: self.category,
            amount: self.amount,
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
    fn list_can_add_multiple_items() {
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