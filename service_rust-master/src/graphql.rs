use std::sync::Arc;

use actix_web::{web, Error, HttpResponse};
use futures::future::Future;

use juniper::http::playground::playground_source;
use juniper::{http::GraphQLRequest, Executor, FieldResult};
use juniper_from_schema::graphql_schema_from_file;

use diesel::prelude::*;


use crate::{DbCon, DbPool};

graphql_schema_from_file!("src/schema.graphql");

pub struct Context {
    db_con: DbCon,
}
impl juniper::Context for Context {}

pub struct Query;
pub struct Mutation;

impl QueryFields for Query {
    fn field_event(
        &self,
        _executor: &Executor<'_, Context>,
        _trail: &QueryTrail<'_, Events, Walked>,
        skip: i32,
        limit: i32,
    ) -> FieldResult<Vec<Events>> {
        use crate::schema::event;

        let result = event::table
            .offset(skip.into())
            .limit(limit.into())
            .load::<crate::models::Events>(&_executor.context().db_con)
            .expect("Error loading posts");

        Ok(result
            .iter()
            .map(|event| Events {
                id: event.id,
                summary: event.summary.to_owned(),
                location: event.location.to_owned(),
                description: event.description.to_owned(),
            })
            .collect())
    }
}

impl MutationFields for Mutation {
    fn field_add_event(
        &self,
        _executor: &Executor<'_, Context>,
        _trail: &QueryTrail<'_, Events, Walked>,
        summary: String,
        location: String,
        description: String,
    ) -> FieldResult<Events> {
        use crate::schema::event;

        let new_event = crate::models::NewEvent { summary: summary, location: location, description: description };

        let event: crate::models::Events = diesel::insert_into(event::table)
            .values(&new_event)
            .get_result(&_executor.context().db_con)
            .expect("Error saving new post");

        Ok(Events {
            id: event.id,
            summary: event.summary.to_owned(),
            location: event.location.to_owned(),
            description: event.description.to_owned(),
        })
    }
}

pub struct Events {
    id: i32,
    summary: String,
    location: String,
    description: String
}

impl EventsFields for Events {
    fn field_id(&self, _: &Executor<'_, Context>) -> FieldResult<juniper::ID> {
        Ok(juniper::ID::new(self.id.to_string()))
    }

    fn field_summary(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.summary)
    }

    fn field_location(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.location)
    }

    fn field_description(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.description)
    }
}

fn playground() -> HttpResponse {
    let html = playground_source("");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

fn graphql(
    schema: web::Data<Arc<Schema>>,
    data: web::Json<GraphQLRequest>,
    db_pool: web::Data<DbPool>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let ctx = Context {
        db_con: db_pool.get().unwrap(),
    };

    web::block(move || {
        let res = data.execute(&schema, &ctx);
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .map_err(Error::from)
    .and_then(|user| {
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(user))
    })
}

pub fn register(config: &mut web::ServiceConfig) {
    let schema = std::sync::Arc::new(Schema::new(Query, Mutation));

    config
        .data(schema)
        .route("/", web::post().to_async(graphql))
        .route("/", web::get().to(playground));
}
