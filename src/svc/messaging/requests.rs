use rand::{self, distributions::Alphanumeric, Rng};
use sozu_command_lib::{
    config::ListenerBuilder,
    proto::command::{request::RequestType, AddBackend, Cluster, Request, RequestHttpFrontend},
};

use super::GeneratorError;

pub fn generate_requests_for_a_random_cluster() -> Result<Vec<Request>, GeneratorError> {
    let mut requests: Vec<Request> = Vec::new();

    let random_cluster_id = random_id_of_7_chars();
    let random_hostname = format!("{}.com", random_cluster_id);
    let random_address = random_socket_address();
    // let address = "127.0.0.1:8080".to_string();

    let add_cluster = RequestType::AddCluster(Cluster {
        cluster_id: random_cluster_id.clone(),
        ..Default::default()
    })
    .into();
    requests.push(add_cluster);

    let listener = ListenerBuilder::new_http(&random_address)
        .to_http(None)
        .map_err(|to_http_error| GeneratorError::ListenerCreation(to_http_error.to_string()))?;

    let add_listener = RequestType::AddHttpListener(listener).into();
    requests.push(add_listener);

    let add_frontend = RequestType::AddHttpFrontend(RequestHttpFrontend {
        cluster_id: Some(random_cluster_id.clone()),
        address: random_address,
        hostname: random_hostname,
        ..Default::default()
    })
    .into();
    requests.push(add_frontend);

    let add_backend_1 = RequestType::AddBackend(AddBackend {
        cluster_id: random_cluster_id.clone(),
        backend_id: format!("{}_backend_1", random_cluster_id),
        address: random_socket_address(),
        ..Default::default()
    })
    .into();
    requests.push(add_backend_1);

    let add_backend_2 = RequestType::AddBackend(AddBackend {
        cluster_id: random_cluster_id.clone(),
        backend_id: format!("{}_backend_2", random_cluster_id),
        address: random_socket_address(),
        ..Default::default()
    })
    .into();
    requests.push(add_backend_2);

    Ok(requests)
}

fn random_id_of_7_chars() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect()
}

fn random_socket_address() -> String {
    let mut rng = rand::thread_rng();

    let ip_address = format!(
        "{}.{}.{}.{}",
        rng.gen_range(0..256),
        rng.gen_range(0..256),
        rng.gen_range(0..256),
        rng.gen_range(0..256)
    );

    let port = rng.gen_range(1025..65536);

    let socket_address = format!("{}:{}", ip_address, port);

    socket_address
}
