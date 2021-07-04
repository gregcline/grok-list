use rocket::{State, http::Status, post, serde::{json::Json}};
use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;
use rocket::error;
use tap::Tap;

use crate::repo::{Repo, RepoError};
use crate::user::User as RepoUser;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct User {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: String,
}

impl User {
    pub fn new(id: Option<ObjectId>, name: String, email: String) -> Self {
        User {
            id: id,
            name,
            email,
        }
    }
}

#[post("/users", data="<user>")]
pub async fn create_user(user: Json<User>, repo: &State<Repo>) -> Result<Json<User>, Status> {
    let new_user = repo.add_user(&RepoUser::new(
        user.name.to_owned(),
        user.email.to_owned()))
        .await
        .map_err(|err| {
            error!("{:?}", err);
            Status::InternalServerError
        })?
        .ok_or_else(|| {
            error!("No new user returned");
            Status::InternalServerError
        })?;

    Ok(Json(User::new(new_user._id, new_user.name, new_user.email)))
}