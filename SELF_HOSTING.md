# Self-Hosting RipTide

This guide provides instructions on how to self-host the RipTide API using Docker. This is the recommended way to run RipTide in production.

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/)
- [Docker Compose](https://docs.docker.com/compose/install/)

## 1. Clone the Repository

First, clone the RipTide repository to your local machine:

```bash
git clone https://github.com/your-org/riptide-api.git
cd riptide-api
```

## 2. Configure Your Environment

RipTide is configured using environment variables. You can set these variables in a `.env` file.

Create a `.env` file by copying the example file:

```bash
cp .env.example .env
```

Now, open the `.env` file and edit the variables as needed. The most important variable to set is `SERPER_API_KEY`, which is required for the search functionality.

```
# .env

# Serper.dev API key for search functionality
SERPER_API_KEY=your_serper_api_key_here

# Redis connection
REDIS_URL=redis://localhost:6379/0

# Headless service URL
HEADLESS_URL=http://localhost:9123

# Logging level
RUST_LOG=info
```

## 3. Start the Application

Once you have configured your `.env` file, you can start the application using Docker Compose:

```bash
docker-compose up -d
```

This command will build the RipTide image and start the following services:

- `riptide-api`: The main RipTide API service.
- `redis`: The Redis database for caching.
- `swagger-ui`: The Swagger UI for API documentation.

## 4. Accessing the Services

Once the services are running, you can access them at the following URLs:

- **RipTide API**: `http://localhost:8080`
- **Swagger UI**: `http://localhost:8081`

## 5. Stopping the Application

To stop the application, run the following command:

```bash
docker-compose down
```
