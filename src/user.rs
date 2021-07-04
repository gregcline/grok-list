use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct User {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    pub name: String,
    pub email: String,
}

impl User {
    pub fn new(name: String, email: String) -> Self {
        User {
            _id: None,
            name,
            email,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn name_and_email_are_required() {
        let name = "foo";
        let email = "foo@bar.com";
        let user = User::new(name.to_string(), email.to_string());

        assert_eq!(user.name, name);
        assert_eq!(user.email, email);
        assert_eq!(user._id, None);
    }
}
