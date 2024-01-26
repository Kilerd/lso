-- Add migration script here

CREATE TABLE "analyzer_results" (
    "id" integer NOT NULL,
    "explain_id" varchar NOT NULL,
    "name" varchar NOT NULL,
    "pass" bool NOT NULL,
    "msg" text, PRIMARY KEY (id)
);
