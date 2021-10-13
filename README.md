# Rust backend project

![full workflow](https://github.com/tappoz/rust-personal-project/actions/workflows/rust.yml/badge.svg)

## Overview

1. HTTP REST API to create/retrieve/search "work to do": `pp_backend_api`.
2. Postgres DB via Docker with a SQL schema: `pp_storage`.
3. `pp_lib` Rust library sharing the source code for the business logic.
4. CLI utility (`cli_01`) to interact with the HTTP API and the DB directly via `pp_lib`.
5. CLI utility (`cli_02`) to open a TCP socket and perform a manual HTTP 
   call to `pp_backend_api`
6. AMQP RabbitMQ queue to publish/subscribe to produce/consume messages.
7. `task_producer` a schedule (CLI) to produce messages as "work demand" via AMQP.
8. A backend schedule to consume the AMQP queue - `task_consumer` 
   that maps a "work demand" to a "work to do" in the DB, then stores
   "events" in the DB table to track the execution of the work to do,
   finally it updates the work row in the DB with the results of the calculations.

All the operations can be performed with a dedicated target in the `Makefile`.

The "work to do" is adding together numbers up to a upper bound threshold.

## API payloads (HTTP REST):

```json
{
  "id": 21,
  "work_code": "api-bjq8euwsEA",
  "add_up_to": 4,
  "done": false,
  "created_on": 1634115736,
  "updated_on": 1634115736
}
```

- Rust structure: `Work`.
- The `work_code` field has a prefix of `api-*`.
- The `done` field is `false` and is supposed to be updated 
  when a hypothetical backend schedule picks up work to do 
  from the database table `works` with a time-range filter 
  (e.g. last 6 hours).
- Both the `work_code` suffix and the `add_up_to` field are
  randomly generated before adding the row to the database.

## Queue messages (AMQP):

```json
{
  "add_up_to": 2,
  "done": false
}
```

- Rust structure: `WorkDemand`.
- This is translated into a Rust structure `Work` by the `task_consumer`.
- The `work_code` for a message pulled from the queue has a prefix of `consumer-*`.
- The `done` field is updated to `true` once the calculations 
  have been performed by the `task_consumer`.

These calculated rows can be searched for from the HTTP API to be retrieved.

## Architecture

![architecture](./doc/architecture.drawio.png "Architecture")

