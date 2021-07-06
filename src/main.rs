use rocket::{error, fairing::AdHoc, launch, routes};
use serde::Deserialize;
use crate::{repo::Repo, user_handlers::create_user};
use thiserror::Error;

mod list;
mod repo;
mod store;
mod user;
mod user_handlers;

#[derive(Deserialize, Debug, Clone)]
pub struct DbConfig {
    database_url: String,
    database_name: String,
}

#[derive(Error, Debug)]
enum StartUpError {
    #[error("could fetch db config during start up")]
    ConfigError,
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/api", routes![create_user])
        .attach(AdHoc::config::<DbConfig>())
        .attach(AdHoc::try_on_ignite("Mongo", |rocket| async {
            let db_config = match rocket.state::<DbConfig>() {
                Some(config) => config,
                None => {
                    error!("{:?}", StartUpError::ConfigError);
                    return Err(rocket);
                }
            };
            let repo = match Repo::new(&db_config).await {
                Ok(repo) => repo,
                Err(e) => {
                    error!("Could not connect to Mongo: {:?}", e);
                    return Err(rocket);
                }
            };

            Ok(rocket.manage(repo))
        }))
}

#[cfg(test)]
mod test {
    use std::env;

    use crate::DbConfig;
    use crate::repo::Collections;
    use crate::user_handlers::User;

    use super::rocket;
    use mongodb::bson::doc;
    use rocket::local::blocking::Client;
    use rocket::http::Status;
    use color_eyre::Result;

    fn run_in_test() {
        env::set_var("ROCKET_PROFILE", "test");
    }

    pub async fn clean_up_db(db_config: &DbConfig) -> Result<()> {
        let client = mongodb::Client::with_uri_str(&db_config.database_url)
            .await?.database(&db_config.database_name);
        client.collection(&Collections::Lists.to_string()).delete_many(doc! {}, None).await?;
        client.collection(&Collections::Stores.to_string()).delete_many(doc! {}, None).await?;
        client.collection(&Collections::Users.to_string()).delete_many(doc! {}, None).await?;

        Ok(())
    }

    #[tokio::test]
    async fn can_add_user() -> Result<()> {
        run_in_test();

        let rocket = rocket().ignite().await.unwrap();
        let db_config = rocket.state::<DbConfig>().unwrap().clone();
        let client = Client::tracked(rocket).expect("valid rocket instant");
        let response = client
            .post("/api/users")
            .json(&User::new(None, "foo".to_string(), "foo@bar.com".to_string()))
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        let json = response.into_json::<User>().unwrap();
        assert_eq!(json.name, "foo");
        assert_eq!(json.email, "foo@bar.com");

        clean_up_db(&db_config).await
    }
}