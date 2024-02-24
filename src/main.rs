use actix_web::{web, App, HttpResponse, HttpServer,Responder,get, HttpRequest};
use tokio::time::{sleep, Duration};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
mod reverse;


async fn my_handler() -> impl Responder {
    sleep(Duration::from_secs(5)).await; // <-- Ok. Worker thread will handle other requests here
    HttpResponse::Ok()
    .content_type("text/plain; charset=utf-8")
    .body("Traitement terminé après un délai de ")
    }

#[get("/")]
async fn index_port_8081(_req: HttpRequest) -> impl Responder{
    HttpResponse::Ok().body("Welcome to port 8081")
}

#[get("/")]
async fn index_port_8082(_req: HttpRequest) -> impl Responder{
    HttpResponse::Ok().body("Welcome to port 8082")
}
#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder= SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file("./nopass.pem", SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file("./cert.pem").unwrap();

    let proxy_task = tokio::spawn(async {
        reverse::start_reverse_proxy().await
    });


        let server_8081 = tokio::spawn(async {
            HttpServer::new(|| {
            App::new()
           .service(index_port_8081)
           .service(web::resource("/long").route(web::get().to(my_handler)))
        })
        .bind(("127.0.0.1", 8081))?
        .run();
        .await
        });

        let server_8082 = 
        tokio::spawn(async {
            HttpServer::new(|| {
         App::new()
        .service(index_port_8082)
        .service(web::resource("/long").route(web::get().to(my_handler)))
        })
        .bind(("127.0.0.1", 8082))? 
        .run();
        .await
    });
        tokio::try_join!(proxy_task, server_8081, server_8082)?;
        Ok(())
}
