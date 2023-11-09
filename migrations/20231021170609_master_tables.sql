CREATE TABLE IF NOT EXISTS master_entries
(
    cite_key      TEXT PRIMARY KEY UNIQUE NOT NULL,
    entry_type    VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS book
(
    book_id     TEXT PRIMARY KEY UNIQUE NOT NULL,
    cite_key    TEXT REFERENCES master_entries(cite_key),
    publisher_id TEXT REFERENCES publisher(publisher_id),
    month_year_id TEXT REFERENCES month_year(month_year_id),
    editor      VARCHAR,
    title       VARCHAR,
    pages       VARCHAR,
    volume      VARCHAR,
    edition     VARCHAR,
    series      VARCHAR,
    note        VARCHAR
);

CREATE TABLE IF NOT EXISTS relationship
(
    parent_id   TEXT PRIMARY KEY UNIQUE NOT NULL,
    child_id    INTEGER,
    cite_key    TEXT REFERENCES master_entries(cite_key)
);

CREATE TABLE IF NOT EXISTS author
(
    cite_key    TEXT REFERENCES master_entries(cite_key),
    author_id   TEXT PRIMARY KEY UNIQUE NOT NULL,
    authors     VARCHAR
);

CREATE TABLE IF NOT EXISTS publisher
(
    publisher_id    TEXT PRIMARY KEY UNIQUE NOT NULL,
    publisher       VARCHAR,
    address         VARCHAR
);

CREATE TABLE IF NOT EXISTS organizations
(
    organization_id TEXT PRIMARY KEY UNIQUE NOT NULL,
    organization    VARCHAR,
    address         VARCHAR
);

CREATE TABLE IF NOT EXISTS month_year
(
    month_year_id   TEXT PRIMARY KEY UNIQUE NOT NULL,
    month           VARCHAR,
    year            INTEGER
);

CREATE TABLE IF NOT EXISTS article
(
    cite_key        TEXT REFERENCES master_entries(cite_key),
    article_id      TEXT PRIMARY KEY UNIQUE NOT NULL,
    publisher_id    TEXT REFERENCES publisher(publisher_id),
    month_year_id   TEXT REFERENCES month_year(month_year_id),
    title           VARCHAR,
    journal         VARCHAR,
    volume          VARCHAR,
    pages           VARCHAR,
    note            VARCHAR,
    edition         VARCHAR
);

