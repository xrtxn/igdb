pub mod models;
mod utils;

use reqwest::{ header::HeaderMap, header::HeaderValue, blocking::Client };
use serde::{ de::DeserializeOwned };
use utils::EndpointUtils;
use utils::response_handler::Result;

const BASE_URL: &str = "https://api.igdb.com";
const VERSION:  &str = "v4";

pub struct APIWrapper {
    http_client: Client,
}

impl APIWrapper {
    pub fn new(
        access_token: &str,
        client_id: &str
    ) -> Result<APIWrapper> {
        let mut headers: HeaderMap = HeaderMap::new();
        
        headers.insert("Authorization", format!("Bearer {}", access_token).parse().unwrap());
        headers.insert("Client-ID", HeaderValue::from_str(client_id).unwrap());
    
        let http_client: Client = Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        let wrapper = APIWrapper { http_client };

        Ok(wrapper)
    }

    fn post<D>(
      &self,
      body: String,
      request_endpoint: &str
    ) -> Result<D>
    where
      D: DeserializeOwned,
    {
      Ok(
        self
          .http_client
          .post(format!("{}/{}/{}", BASE_URL, VERSION, request_endpoint))
          .body(body)
          .send()
          .unwrap()
          .json()
          .unwrap()
      )
    }

    fn post_json_response(
      &self,
      body: String,
      request_endpoint: &str
    ) -> Result<Vec<serde_json::Value>>
    {
        let response = self
          .http_client
          .post(format!("{}/{}/{}", BASE_URL, VERSION, request_endpoint))
          .body(body)
          .send()
          .unwrap()
          .text()
          .unwrap();

        Ok(serde_json::from_str(&response).unwrap())
    }

    #[cfg(feature = "game")]
    pub fn games<'a>(&'a self) -> EndpointUtils<'a>{
      EndpointUtils { wrapper: self, query_string: Vec::new(), endpoint: "games"}
    }

    #[cfg(feature = "character")]
    pub fn characters<'a>(&'a self) -> EndpointUtils<'a>{
      EndpointUtils { wrapper: self, query_string: Vec::new(), endpoint: "characters"}
    }

    #[cfg(feature = "genre")]
    pub fn genres<'a>(&'a self) -> EndpointUtils<'a>{
      EndpointUtils { wrapper: self, query_string: Vec::new(), endpoint: "genres"}
    }
}

#[cfg(test)]
mod tests {
  use super::*;
  use models::*;
  use std::env;

  // Testing FIELDS apicalypse query
  #[test]
  fn fields_test() {
    let access_token = env::var("TWITCH_ACCESS_TOKEN").unwrap();
    let client_id = env::var("TWITCH_CLIENT_ID").unwrap();
    let api_wrapper = APIWrapper::new(&access_token, &client_id).unwrap();

    let games: Vec<Game> = api_wrapper.games()
      .fields("name")
      .limit("2")
      .request()
      .unwrap();

    let expected_result: Vec<Game> = vec![
      Game { id: 176032, name: String::from("Nick Quest") },
      Game { id: 246925, name: String::from("Stickman and the Sword of Legends") }
    ];

    assert_eq!(&expected_result, &games)
  }

  // Testing EXCLUDE apicalypse query
  #[test]
  fn exclude_test() {
    let access_token = env::var("TWITCH_ACCESS_TOKEN").unwrap();
    let client_id = env::var("TWITCH_CLIENT_ID").unwrap();
    let api_wrapper = APIWrapper::new(&access_token, &client_id).unwrap();

    let genres_without_slug_field: Vec<Genre> = api_wrapper.genres()
      .fields("name, slug")
      .exclude("slug")
      .limit("3")
      .request()
      .unwrap();

    let expected_result: Vec<Genre> = vec![
      Genre { id: 4, name: String::from("Fighting"), slug: None },
      Genre { id: 5, name: String::from("Shooter"), slug: None },
      Genre { id: 7, name: String::from("Music"), slug: None }
    ];

    assert_eq!(&expected_result, &genres_without_slug_field)
  }

  // Testing WHERE apicalypse query
  #[test]
  fn where_test() {
    let access_token = env::var("TWITCH_ACCESS_TOKEN").unwrap();
    let client_id = env::var("TWITCH_CLIENT_ID").unwrap();
    let api_wrapper = APIWrapper::new(&access_token, &client_id).unwrap();

    let test_characters: Vec<Character> = api_wrapper.characters()
      .fields("*")
      .where_like("gender != null")
      .where_like("species != null")
      .request()
      .unwrap();

    let expected_character_result = Character {
      id: 4445,
      akas: None,
      country_name: None,
      description: None,
      created_at: Some(1431216000),
      games: Some(vec![380, 1219, 1221, 2993]),
      gender: Some(0),
      mug_shot: Some(3620),
      name: Some(String::from("Beast")),
      slug: Some(String::from("beast")),
      species: Some(5),
      updated_at: Some(1472601600),
      url: Some(String::from("https://www.igdb.com/characters/beast")),
      checksum: Some(String::from("eb661aaf-a1e1-4acf-b48a-14b8aaa26a52"))
    };

    assert_eq!(&test_characters[0], &expected_character_result);
    assert_eq!(&test_characters[0].gender(), "Male");
    assert_eq!(&test_characters[0].species(), "Unknown");
  }

  // Testing SEARCH apicalypse query
  #[test]
  fn search_test() {
    let access_token = env::var("TWITCH_ACCESS_TOKEN").unwrap();
    let client_id = env::var("TWITCH_CLIENT_ID").unwrap();
    let api_wrapper = APIWrapper::new(&access_token, &client_id).unwrap();

    let games: Vec<Game> = api_wrapper.games()
      .fields("name")
      .search("zelda")
      .limit("2")
      .request()
      .unwrap();

    let expected_result: Vec<Game> = vec![
      Game { id: 1025, name: String::from("Zelda II: The Adventure of Link") },
      Game { id: 1022, name: String::from("The Legend of Zelda") }
    ];

    assert_eq!(&expected_result, &games)
  }

  // Testing LIMIT, OFFSET, SORT ASC, SORT DESC apicalypse queries
  #[test]
  fn sorting_test() {
    let access_token = env::var("TWITCH_ACCESS_TOKEN").unwrap();
    let client_id = env::var("TWITCH_CLIENT_ID").unwrap();
    let api_wrapper = APIWrapper::new(&access_token, &client_id).unwrap();

    let games_limited_by_5_asc: Vec<Game> = api_wrapper.games()
      .fields("name")
      .limit("5")
      .sort_asc("id")
      .request()
      .unwrap();

    let first_expected_result: Vec<Game> = vec![
      Game { id: 1, name: String::from("Thief II: The Metal Age") },
      Game { id: 2, name: String::from("Thief: The Dark Project") },
      Game { id: 3, name: String::from("Thief: Deadly Shadows") },
      Game { id: 4, name: String::from("Thief") },
      Game { id: 5, name: String::from("Baldur's Gate") }
    ];

    assert_eq!(&first_expected_result, &games_limited_by_5_asc);

    let games_with_offset_desc: Vec<Game> = api_wrapper.games()
      .fields("name")
      .limit("2")
      .offset("3")
      .sort_desc("id")
      .request()
      .unwrap();

    let second_expected_result: Vec<Game> = vec![
      Game { id: 255523, name: String::from("Doki-doki Surprise") },
      Game { id: 255522, name: String::from("Boreal Tenebrae: Deluxe Special Edition") },
    ];

    assert_eq!(&second_expected_result, &games_with_offset_desc);
  }

  #[test]
  fn json_response_test() {
    let access_token = env::var("TWITCH_ACCESS_TOKEN").unwrap();
    let client_id = env::var("TWITCH_CLIENT_ID").unwrap();
    let api_wrapper = APIWrapper::new(&access_token, &client_id).unwrap();

    let test_characters: Vec<serde_json::Value> = api_wrapper.characters()
      .fields("name, gender, country_name")
      .where_like("gender != null")
      .limit("4")
      .request_json()
      .unwrap();

    let expected_result = vec![
      serde_json::json!({
        "gender": 0,
        "id": 4445,
        "name": "Beast"
      }),
      serde_json::json!({
        "gender": 0,
        "id": 8988,
        "name": "Mr. Wong"
      }),
      serde_json::json!({
        "gender": 1,
        "id": 1143,
        "name": "Annie"
      }), 
      serde_json::json!({
        "gender": 0,
        "id": 6032,
        "name": "Richtofen"
      })
    ];

    assert_eq!(&test_characters, &expected_result);
  }
}
