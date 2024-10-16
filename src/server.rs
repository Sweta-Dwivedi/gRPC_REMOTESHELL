use tonic::{Request, Response, Status};
use tonic::transport::Server;
use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloResponse, HelloRequest};
use mongodb::{Client, Collection};
use serde::{Deserialize, Serialize};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[derive(Debug, Serialize, Deserialize)]
struct HelloResponseMongo {
    message: String,
}

#[derive(Debug)]
pub struct MyGreeter {
    collection: Collection<HelloResponseMongo>,
}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(&self, request: Request<HelloRequest>) -> Result<Response<HelloResponse>, Status> {
        let message = format!("Hello {} from Server Side", request.into_inner().name);
        let reply = HelloResponse { message: message.clone() };

        // Store the response in MongoDB
        let response_doc = HelloResponseMongo { message: message.clone() };
        self.collection.insert_one(response_doc, None).await.map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to MongoDB
    let client = Client::with_uri_str("mongodb://localhost:27017").await?;
    let database = client.database("my_database1");
    let collection = database.collection::<HelloResponseMongo>("responses");

    // Set up the greeter service
    let greeter_service = MyGreeter { collection };
    
    let address = "[::1]:50051".parse()?;
    println!("Listening...!");
    Server::builder()
        .add_service(GreeterServer::new(greeter_service))
        .serve(address)
        .await?;

    Ok(())
}
