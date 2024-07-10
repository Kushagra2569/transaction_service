use chrono::prelude::*;
use transaction_service::trnx_service;

#[cfg(test)]
use ::axum_test::TestServer;
#[cfg(test)]
use ::axum_test::TestServerConfig;
use ::serde::Deserialize;
use ::serde::Serialize;

fn test_server() -> TestServer {
    // Build an application with a route.
    let app = trnx_service();

    println!("server started");
    // Run the application for testing.
    let config = TestServerConfig::builder()
        .expect_success_by_default()
        .mock_transport()
        .build();

    TestServer::new_with_config(app, config).unwrap()
}

#[cfg(test)]
mod test_user_registration {
    use super::*;
    use ::serde_json::json;

    #[tokio::test]
    async fn user_registration_check() {
        let server = test_server();
        let response = server
            .post("/register")
            .expect_failure()
            .json(&json!({
                        "email": "testemail123@test.com",
                        "password": "testpassword123",
                        "fullname": "testuser123",
                        "balance": 100.0
            }))
            .await;

        let status = response.status_code();
        assert_eq!(status, 400);
    }
}

#[cfg(test)]
mod test_get_user_balance {
    use super::*;
    use ::serde_json::json;

    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
    struct LoginRequest {
        email: String,
        fullname: String,
        balance: f64,
        token: String,
    }

    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
    struct BalanceRequest {
        email: String,
        balance: f64,
    }

    #[tokio::test]
    async fn user_login_balance() {
        let server = test_server();
        let response = server
            .post("/login")
            .json(&json!({
                        "email": "testemail123@test.com",
                        "password": "testpassword123"
            }))
            .await
            .json::<LoginRequest>();
        println!("{:?}", response);
        let token = response.token;
        let headertoken = format!("Bearer {}", token);
        let header_value_future = axum_test::http::HeaderValue::from_str(headertoken.as_str());
        let header_value = header_value_future.unwrap();

        let balance_response = server
            .get("/balance")
            .json(&json!({
                        "email": "testemail123@test.com",
            }))
            .add_header(axum_test::http::header::AUTHORIZATION, header_value)
            .await
            .json::<BalanceRequest>();
        println!("{:?}", balance_response);
        let balance = balance_response.balance;
        assert_eq!(balance, 100.0)
    }
}

#[cfg(test)]
mod test_get_user_transaction {
    use super::*;
    use ::serde_json::json;

    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
    struct Transaction {
        pub id: String,
        pub from_email: String,
        pub to_email: String,
        pub amount: f64,
        pub trnx_time: DateTime<Utc>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
    struct Transactions {
        pub transactions: Vec<Transaction>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
    struct LoginRequest {
        email: String,
        fullname: String,
        balance: f64,
        token: String,
    }

    #[tokio::test]
    async fn get_user_transactions() {
        let server = test_server();
        let response = server
            .post("/login")
            .json(&json!({
                        "email":"add",
                        "password": "kush123"
            }))
            .await
            .json::<LoginRequest>();
        println!("{:?}", response);
        let token = response.token;
        let headertoken = format!("Bearer {}", token);
        let header_value_future = axum_test::http::HeaderValue::from_str(headertoken.as_str());
        let header_value = header_value_future.unwrap();

        let transaction_response = server
            .get("/transaction")
            .json(&json!({
                        "email": "testemail123@test.com",
            }))
            .add_header(axum_test::http::header::AUTHORIZATION, header_value)
            .await
            .json::<Transactions>();
        println!("{:?}", transaction_response);
        let num_transactions = transaction_response.transactions.len();
        assert_eq!(num_transactions, 2)
    }
}
