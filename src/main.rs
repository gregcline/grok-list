use rocket::{Config, State, error, fairing::AdHoc, launch, routes};
use serde::Deserialize;
use crate::{repo::Repo, user_handlers::create_user};

mod list;
mod repo;
mod store;
mod user;
mod user_handlers;

const MONGO_URI: &str = "mongodb://localhost:27017/";
const DB_NAME: &str = "grok_list";

#[derive(Deserialize, Debug)]
struct DbConfig {
    database_url: String,
    database_name: String,
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/api", routes![create_user])
        .attach(AdHoc::config::<DbConfig>())
        .attach(AdHoc::try_on_ignite("Mongo", |rocket| async {
            let figment = rocket.state::<DbConfig>();
            println!("figment {:#?}", figment);
            let repo = match Repo::new(MONGO_URI, DB_NAME).await {
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

    use crate::user_handlers::User;

    use super::rocket;
    use rocket::local::blocking::Client;
    use rocket::http::Status;

    #[test]
    fn can_add_user() {
        env::set_var("ROCKET_PROFILE", "test");
        let client = Client::tracked(rocket()).expect("valid rocket instant");
        let mut response = client
            .post("/api/users")
            .json(&User::new(None, "foo".to_string(), "foo@bar.com".to_string()))
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        let json = response.into_json::<User>().unwrap();
        assert_eq!(json.name, "foo");
        assert_eq!(json.email, "foo@bar.com");
    }
}