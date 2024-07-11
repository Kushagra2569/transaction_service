# **Transaction Service**

Transaction Service is a robust backend server developed using Rust and the Axum web framework. Designed to provide secure and efficient handling of user data and transactions.

## **Key Features**

### **User Registration and Login:**

  Secure endpoints for user registration and login and error handling.
  Password hashing and storage to ensure user credentials are securely stored.
### **JWT Authentication:**

  Implementation of JSON Web Token (JWT) for secure user authentication.
  Token-based authorization for accessing protected routes.
  Easy integration with front-end applications for seamless authentication.
### **Transaction Management:**

  Endpoints for creating and listing user transactions.
  Detailed transaction logging for audit and tracking purposes.
### **Data Storage:**

  Utilizes PostgreSQL as the primary database for storing user information and transaction records.

## **Pre-Requisites**
Require Rust Compiler setup or dev environment to run.

Require Posgresql database setup with the following table

users table with id , full_name, role,email,and balance

userlogin table with id ,full_name,email,password,created_at and updated_at

authorise table with id,email,token columns

transactions table with id,from_email,to_email,amount and created_at

all the column are of varchar(255) types except for the following

created_at ,updated_at are of TIMESTAMPTZ

role is an enum type

balance is float8

and tables have users id and email as foreign key relations

## **Setup Instructions**
Create a .env file in the root folder and add values for POSTGRES_URL and JWT_KEY
run the command "Cargo run" in the root folder.
Server runs on localhost on port 3042

## **EndPoints**
### **POST /register**
endpoint for registering a new user and setting initial balance
example Json request:
```json
{
    "fullname":"user",
    "password":"user123",
    "email":"user@test.com",
    "balance": 1000
}
```
example Response:
```json
{
    "email": "user@test.com",
    "fullname": "user"
}
```

### **POST /login**
endpoint for logging in as  a existing user
example Json request:
```json
{
    "password":"user123",
    "email":"user@test.com"
}
```
example Response:
```json
{
    "balance": 1000.0,
    "email": "user@test.com",
    "fullname": "user",
    "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE3MjA3NDYxNTYsImlhdCI6MTcyMDY1OTc1NiwiZW1haWwiOiJ1c2VyQHRlc3QuY29tIn0.VB04u5lSOISntkwpxdRcMdSnX4b-l8zUE0_AUWo1pqA"
}
```
 **NOTE: the JWT Auth token is provided here in the token field**

### **PUT /user**
endpoint for modifying user details, currently supports modifying the fullname for the user
Requires the auth token to be set in the bearer header field
example Json request:
```json
{
    "old_name":"user",
    "new_name": "user123"
}
```
example Response:
```json
{
    "email": "user@test.com",
    "fullname": "user123"
}
```

### **POST /authorise**
endpoint for checking if the current user is authorised
Requires the auth token to be set in the bearer header field
example Json request:
```json
{
    "email": "user@test.com"
}
```
example Response:
Returns a 401 unauthorized response if the token is not valid

### **GET /balance**
endpoint for checking the users current balance
Requires the auth token to be set in the bearer header field
example Json request:
```json
{
    "email":"user@test.com"
}
```
example Response:
```json
{
    "balance": 1000.0,
    "email": "user@test.com"
}
```
### **POST /transaction**
endpoint for sending an amount to another user and creating a transaction
Requires the auth token to be set in the bearer header field
example Json request:
```json
{
    "from_email":"user@test.com",
    "to_email": "add",
    "amount": 600
}
```
example Response:
```json
{
    "amount": 600.0,
    "from_email": "user@test.com",
    "to_email": "add"
}
```

### **Get /transaction**
endpoint for listing all the credit and debit transactions
Requires the auth token to be set in the bearer header field
example Json request:
```json
{
}
```
example Response:
```json
{
    "transactions": [
        {
            "amount": 600.0,
            "from_email": "user@test.com",
            "id": "7875cf9202c44ebb96f365f8d4c87d64",
            "to_email": "add",
            "trnx_time": "2024-07-11T01:16:02.117002Z"
        }
    ]
}
```


  
  
