# App Overview

## Root Directory - Structure

### Main

Main is the entry point of the application. It handles:

- Some auth claims extraction
- Currently implements GraphQL Playground
- env::var inits
- App State, schema and jwt service init
- Creates the HttpServer Object, including app_data, logging, and endpoints
    - Endpoints are public and protected
    - Generally exposed by handlers

### Lib

Handles imports, possibly doing something weird here, but TBA

### Errors

Serves to consolidate error codes from parts of the application and different sources into predicatable and consistent errors:

- AppError Validations
- AppError ErrorCodes

Consolidated types include:

- MongoDb errors
- HttpResponse 
- Validator Errors
- Async Graph errors

### Config

App Config class / object. Generally reads in from env variables or secrets and turns them into universally accessible object.


### App State

Creates repositories and services. Presents them in AppState object. Currently includes:

- User repository
- Quiz repository
- Auth service
- Config

## Auth Directory

Generallly holds the auth types. Namely:

- Claims struct in the claims file
- Jwt struct and functionality in the jwt file
- Middleware services in middleware file
    - Retrieves service from app data
    - Retrieves claims via header, then token
    - Hanldles authenticated users
Manages auth utilities
    - Admin and owner status
    - GraphQL claims

## DB Directory

Manages connection and collections from DB. Possibly redundant at this point or could be moved elsewhere.

## GraphQL Directory

Handles GraphQL bits and pieces. Including creating shemas and query / mutation roots.

## Handlers Directory

### Auth Handler

Manages client points of ingress for authentication related activities. Currently exposes:

- auth/github/callback

### User Handler 

Not totally happy with the split of REST endpoints and GraphQL things here. Will have to think about how to manage. Otherwise manages user related requests and exposes endpoints for that purpose.

### Quiz Handler 

TBA, just sketching it out now.

## Models Directory

Contains the models and DTOs for the app to function

### Domain Models Directory

Contains general domain models for usage. Will probably come back through here soon and update existing domain models to be 'options' to enable draft quiz functionality and processing.

### DTO Directory

Data transfer objects. Happy with thinking about these generally as 'requests' given the graph QL implementation.

## Respository Directory

Holds the repo methods for the different types. Could probably consolidate these into something more re-usable, but will probably keep it as is for readability.

## Services Directory

Actually does the work. 

### User

### Quiz


