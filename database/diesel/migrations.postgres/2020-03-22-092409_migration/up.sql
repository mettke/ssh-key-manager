-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- remove foreign keys
do $$
BEGIN
IF EXISTS (
    SELECT NULL 
    FROM information_schema.TABLE_CONSTRAINTS
    WHERE
        table_schema = current_database() AND
        constraint_type = 'FOREIGN KEY'
)
THEN
    ALTER TABLE access
        DROP CONSTRAINT FK_access_entity,
        DROP CONSTRAINT FK_access_entity_2,
        DROP CONSTRAINT FK_access_entity_3;
    ALTER TABLE access_option
        DROP CONSTRAINT FK_access_option_access;
    ALTER TABLE access_request
        DROP CONSTRAINT FK_access_request_entity,
        DROP CONSTRAINT FK_access_request_entity_2,
        DROP CONSTRAINT FK_access_request_entity_3;
    ALTER TABLE entity_admin
        DROP CONSTRAINT FK_entity_admin_entity,
        DROP CONSTRAINT FK_entity_admin_entity_2;
    ALTER TABLE entity_event
        DROP CONSTRAINT FK_entity_event_actor_id,
        DROP CONSTRAINT FK_entity_event_entity_id;
    ALTER TABLE "group"
        DROP CONSTRAINT FK_group_entity;
    ALTER TABLE group_event
        DROP CONSTRAINT FK_group_event_entity,
        DROP CONSTRAINT FK_group_event_group;
    ALTER TABLE group_member
        DROP CONSTRAINT FK_group_member_entity,
        DROP CONSTRAINT FK_group_member_entity_2,
        DROP CONSTRAINT FK_group_member_group;
    ALTER TABLE public_key
        DROP CONSTRAINT FK_public_key_entity;
    ALTER TABLE public_key_dest_rule
        DROP CONSTRAINT FK_public_key_dest_rule_public_key;
    ALTER TABLE public_key_signature
        DROP CONSTRAINT FK_public_key_signature_public_key;
    ALTER TABLE server_account
        DROP CONSTRAINT FK_server_account_entity,
        DROP CONSTRAINT FK_server_account_server;
    ALTER TABLE server_admin
        DROP CONSTRAINT FK_server_admin_entity,
        DROP CONSTRAINT FK_server_admin_server;
    ALTER TABLE server_event
        DROP CONSTRAINT FK_server_event_actor_id,
        DROP CONSTRAINT FK_server_log_server;
    ALTER TABLE server_ldap_access_option
        DROP CONSTRAINT FK_server_ldap_access_option_server;
    ALTER TABLE server_note
        DROP CONSTRAINT FK_server_note_entity,
        DROP CONSTRAINT FK_server_note_server;
    ALTER TABLE sync_request
        DROP CONSTRAINT FK_sync_request_server;
    ALTER TABLE "user"
        DROP CONSTRAINT FK_user_entity;
    ALTER TABLE user_alert
        DROP CONSTRAINT FK_user_alert_entity;
END IF;
END
$$;

CREATE OR REPLACE FUNCTION GEN_UUID()
RETURNS bytea AS $$
DECLARE
    uuid text := uuid_generate_v1() as text;
    uuid_tmp text := uuid;
BEGIN
    uuid := overlay(uuid placing (SUBSTR(uuid_tmp, 15, 4)) from 1 for 4);
    uuid := overlay(uuid placing (SUBSTR(uuid_tmp, 10, 4)) from 5 for 4);
    uuid := overlay(uuid placing (SUBSTR(uuid_tmp, 1, 4)) from 10 for 4);
    uuid := overlay(uuid placing (SUBSTR(uuid_tmp, 5, 4)) from 15 for 4);
    RETURN decode(replace(uuid, '-', ''), 'hex');
END;
$$ LANGUAGE plpgsql;

-- create or migrate entity table
CREATE TYPE entity_type_v3 AS ENUM ('user','server account', 'group');

CREATE TABLE entity_v3 (
    id bytea NOT NULL DEFAULT GEN_UUID(),
    type entity_type_v3 NOT NULL,
    migration_id integer DEFAULT NULL,
    PRIMARY KEY (id)
);

do $$
BEGIN
IF EXISTS (
    SELECT NULL 
    FROM information_schema.TABLES
    WHERE
        TABLE_SCHEMA = current_database() AND
        TABLE_NAME   = 'entity'
)
THEN
    INSERT INTO entity_v3("type", "migration_id") 
        SELECT type::text::entity_type_v3, "id" 
        FROM entity;
    DROP TABLE entity;
    DROP TYPE IF EXISTS entity_type;
END IF;
END
$$;

ALTER TABLE entity_v3
    RENAME TO entity;
ALTER TYPE entity_type_v3 
    RENAME TO entity_type;

-- create or migrate users table
CREATE TYPE user_type AS ENUM ('user', 'admin', 'superuser');

CREATE TABLE users_v3 (
    entity_id bytea NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    uid varchar(50) NOT NULL UNIQUE,
    name varchar(100) DEFAULT NULL,
    email varchar(100) DEFAULT NULL,
    password varchar(250) DEFAULT NULL,
    active boolean NOT NULL DEFAULT true,
    type user_type NOT NULL DEFAULT 'user',
    PRIMARY KEY (entity_id)
);

do $$
BEGIN
IF EXISTS (
    SELECT NULL 
    FROM information_schema.TABLES
    WHERE
        TABLE_SCHEMA = current_database() AND
        TABLE_NAME   = 'user'
)
THEN
    INSERT INTO users_v3 ("entity_id","uid","name","email","active","type")
        SELECT e.id, u.uid, u.name, u.email, u.active and not u.force_disable, 'user' FROM "user" u 
        INNER JOIN entity e ON u.entity_id=e.migration_id
        WHERE u.admin = '0';
    INSERT INTO users_v3 ("entity_id","uid","name","email","active","type")
        SELECT e.id, u.uid, u.name, u.email, u.active and not u.force_disable, 'admin' FROM "user" u 
        INNER JOIN entity e ON u.entity_id=e.migration_id
        WHERE u.admin = '1';
    UPDATE users_v3 SET type = 'superuser'
        WHERE uid = 'keys-sync'; 
    DROP TABLE "user";
    DROP TYPE IF EXISTS user_auth_realm;
ELSE
    WITH var as (INSERT INTO entity("type") VALUES('user') RETURNING ID)
    INSERT INTO users_v3("entity_id", "uid", "name", "type") 
        VALUES((select * from var), 'root', 'Default Administrator', 'superuser');
END IF;
END
$$;

ALTER TABLE users_v3
    RENAME TO "users";

-- create or migrate group table
CREATE TABLE "groups_v3" (
    "entity_id" bytea NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    "name" varchar(150) NOT NULL UNIQUE,
    "active" boolean NOT NULL DEFAULT true,
    "system" boolean NOT NULL DEFAULT false,
    "oauth_scope" varchar(150) DEFAULT NULL,
    "ldap_group" varchar(150) DEFAULT NULL,
    PRIMARY KEY ("entity_id")
);
CREATE INDEX "KEY_groups_oauth_scope" ON "groups_v3" ("oauth_scope");
CREATE INDEX "KEY_groups_ldap_group" ON "groups_v3" ("ldap_group");

do $$
BEGIN
IF EXISTS (
    SELECT NULL 
    FROM information_schema.TABLES
    WHERE
        TABLE_SCHEMA = current_database() AND
        TABLE_NAME   = 'group'
)
THEN
    INSERT INTO groups_v3 ("entity_id", "name", "active", "system") 
        SELECT e.id, g.name, g.active, g.system FROM "group" g 
        INNER JOIN entity e ON g.entity_id=e.migration_id;

    DROP TABLE "group";
END IF;
END
$$;

ALTER TABLE groups_v3
    RENAME TO "groups";

-- create or migrate server_admin table
CREATE TABLE "group_admin" (
    "group_id" bytea NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    "admin_id" bytea NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    PRIMARY KEY ("group_id","admin_id")
);
CREATE INDEX "FK_group_admin_entity" ON "group_admin" ("group_id");

do $$
BEGIN
IF EXISTS (
    SELECT NULL 
    FROM information_schema.TABLES
    WHERE
        TABLE_SCHEMA = current_database() AND
        TABLE_NAME   = 'entity_admin'
)
THEN
    INSERT INTO group_admin
        SELECT e1.id, e2.id
        FROM entity_admin sa
        INNER JOIN entity e1 ON sa.entity_id=e1.migration_id
        LEFT JOIN entity e2 ON sa.admin=e2.migration_id
        WHERE e1.type = 'group'::entity_type;
END IF;
END
$$;

-- create or migrate group_member table
CREATE TABLE "group_member_v3" (
    "group_id" bytea NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    "member_id" bytea NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    "add_date" timestamp with time zone NOT NULL,
    "added_by" bytea DEFAULT NULL,
    PRIMARY KEY ("group_id", "member_id")
);

do $$
BEGIN
IF EXISTS (
    SELECT NULL 
    FROM information_schema.TABLES
    WHERE
        TABLE_SCHEMA = current_database() AND
        TABLE_NAME   = 'group_member'
)
THEN
    INSERT INTO group_member_v3 SELECT e1.id, e2.id, gm.add_date, e3.id
        FROM group_member gm 
        INNER JOIN entity e1 ON gm.group=e1.migration_id
        INNER JOIN entity e2 ON gm.entity_id=e2.migration_id
        LEFT JOIN entity e3 ON gm.added_by=e3.migration_id;

    DROP TABLE "group_member";
END IF;
END
$$;

ALTER TABLE group_member_v3
    RENAME TO "group_member";

-- create or migrate server table
CREATE TYPE server_key_management_v3 AS ENUM ('none', 'keys', 'other');
CREATE TYPE server_authorization_v3 AS ENUM ('manual', 'automatic');
CREATE TYPE server_sync_status_v3 AS ENUM ('not synced yet', 'sync success', 'sync failure', 'sync warning');

CREATE TABLE "server_v3" (
    "id" bytea NOT NULL DEFAULT GEN_UUID(),
    "hostname" varchar(150) NOT NULL,
    "ip_address" varchar(64) DEFAULT NULL, -- TODO: 128 bits for ipv6 and 32 bits for ipv4
    "name" varchar(100) DEFAULT NULL,
    "deleted" boolean NOT NULL DEFAULT false,
    "key_management" server_key_management_v3 NOT NULL DEFAULT 'keys',
    "authorization" server_authorization_v3 NOT NULL DEFAULT 'manual',
    "sync_status" server_sync_status_v3 NOT NULL DEFAULT 'not synced yet',
    "rsa_key_fingerprint" char(32) DEFAULT NULL, -- TODO ?
    "port" integer NOT NULL DEFAULT 22,
    "migration_id" integer DEFAULT NULL,
    PRIMARY KEY ("id")
);
CREATE INDEX "KEY_server_hostname" ON "server_v3" ("hostname");
CREATE INDEX "KEY_server_ip_address" ON "server_v3" ("ip_address");
CREATE INDEX "KEY_server_rsa_key_fingerprint" ON "server_v3" ("rsa_key_fingerprint");
CREATE INDEX "KEY_server_port" ON "server_v3" ("port");

do $$
BEGIN
IF EXISTS (
    SELECT NULL 
    FROM information_schema.TABLES
    WHERE
        TABLE_SCHEMA = current_database() AND
        TABLE_NAME   = 'server'
)
THEN
    INSERT INTO server_v3 ("hostname", "ip_address", "deleted", "key_management", "authorization", "sync_status", "rsa_key_fingerprint", "port", "migration_id")
        SELECT s.hostname, s.ip_address, s.deleted, s.key_management::text::server_key_management_v3, s.authorization::text::server_authorization_v3, s.sync_status::text::server_sync_status_v3, s.rsa_key_fingerprint, s.port, s.id 
        FROM server s
        WHERE s.key_management!='decommissioned' AND s.authorization!='automatic LDAP' AND s.authorization!='manual LDAP';

    INSERT INTO server_v3 ("hostname", "ip_address", "deleted", "key_management", "authorization", "sync_status", "rsa_key_fingerprint", "port", "migration_id")
        SELECT s.hostname, s.ip_address, true, 'none', s.authorization::text::server_authorization_v3, s.sync_status::text::server_sync_status_v3, s.rsa_key_fingerprint, s.port, s.id 
        FROM server s
        WHERE s.key_management='decommissioned' AND s.authorization!='automatic LDAP' AND s.authorization!='manual LDAP';

    INSERT INTO server_v3 ("hostname", "ip_address", "deleted", "key_management", "authorization", "sync_status", "rsa_key_fingerprint", "port", "migration_id")
        SELECT s.hostname, s.ip_address, s.deleted, s.key_management::text::server_key_management_v3, 'automatic', s.sync_status::text::server_sync_status_v3, s.rsa_key_fingerprint, s.port, s.id 
        FROM server s
        WHERE s.key_management!='decommissioned' AND (s.authorization='automatic LDAP' OR s.authorization='manual LDAP');

    INSERT INTO server_v3 ("hostname", "ip_address", "deleted", "key_management", "authorization", "sync_status", "rsa_key_fingerprint", "port", "migration_id")
        SELECT s.hostname, s.ip_address, true, 'none', 'automatic', s.sync_status::text::server_sync_status_v3, s.rsa_key_fingerprint, s.port, s.id 
        FROM server s
        WHERE s.key_management='decommissioned' AND (s.authorization='automatic LDAP' OR s.authorization='manual LDAP');

    DROP TABLE "server";
    DROP TYPE IF EXISTS server_key_management;
    DROP TYPE IF EXISTS server_authorization;
    DROP TYPE IF EXISTS server_sync_status;
    DROP TYPE IF EXISTS server_configuration_system;
    DROP TYPE IF EXISTS server_custom_keys;
    DROP TYPE IF EXISTS server_use_sync_client;
END IF;
END
$$;

ALTER TABLE server_v3
    RENAME TO server;
ALTER TYPE server_key_management_v3 
    RENAME TO server_key_management;
ALTER TYPE server_authorization_v3 
    RENAME TO server_authorization;
ALTER TYPE server_sync_status_v3 
    RENAME TO server_sync_status;

-- create or migrate server_account table
CREATE TABLE "server_account_v3" (
    "entity_id" bytea NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    "server_id" bytea NOT NULL REFERENCES server(id) ON DELETE CASCADE,
    "name" varchar(100) DEFAULT NULL,
    "sync_status" server_sync_status NOT NULL DEFAULT 'not synced yet',
    "active" boolean NOT NULL DEFAULT true,
    PRIMARY KEY ("entity_id"),
    CONSTRAINT "KEY_server_account_combination" UNIQUE("server_id", "name")
);
CREATE INDEX "KEY_server_account_server_id" ON "server_account_v3" ("server_id");

do $$
BEGIN
IF EXISTS (
    SELECT NULL 
    FROM information_schema.TABLES
    WHERE
        TABLE_SCHEMA = current_database() AND
        TABLE_NAME   = 'server_account'
)
THEN
    INSERT INTO server_account_v3
        SELECT e.id, s.id, sa.name, sa.sync_status::text::server_sync_status, sa.active
        FROM server_account sa
        INNER JOIN entity e ON sa.entity_id=e.migration_id
        INNER JOIN server s ON sa.server_id=s.migration_id
        WHERE sa.sync_status != 'proposed';

    INSERT INTO server_account_v3
        SELECT e.id, s.id, sa.name, 'not synced yet', sa.active
        FROM server_account sa
        INNER JOIN entity e ON sa.entity_id=e.migration_id
        INNER JOIN server s ON sa.server_id=s.migration_id
        WHERE sa.sync_status = 'proposed';

    DROP TABLE "server_account";
    DROP TYPE IF EXISTS server_account_sync_status;
END IF;
END
$$;

ALTER TABLE server_account_v3
    RENAME TO server_account;

-- create or migrate sync_request table
CREATE TABLE "sync_request_v3" (
    "id" bytea NOT NULL DEFAULT GEN_UUID(),
    "server_id" bytea NOT NULL REFERENCES server(id) ON DELETE CASCADE,
    "account_id" bytea NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    "processing" boolean NOT NULL DEFAULT false,
    PRIMARY KEY ("id"),
    CONSTRAINT "KEY_sync_request_combination" UNIQUE("server_id", "account_id")
);

do $$
BEGIN
IF EXISTS (
    SELECT NULL 
    FROM information_schema.TABLES
    WHERE
        TABLE_SCHEMA = current_database() AND
        TABLE_NAME   = 'sync_request'
)
THEN
    INSERT INTO sync_request_v3("server_id", "account_id", "processing")
        SELECT s.id, sa.entity_id, sr.processing
        FROM sync_request sr
        INNER JOIN server s ON sr.server_id=s.migration_id
        LEFT JOIN server_account sa ON sr.account_name=sa.name;

    DROP TABLE "sync_request";
END IF;
END
$$;

ALTER TABLE sync_request_v3
    RENAME TO sync_request;

-- create or migrate server_note table
CREATE TABLE "server_note_v3" (
    "id" bytea NOT NULL DEFAULT GEN_UUID(),
    "server_id" bytea NOT NULL REFERENCES server(id) ON DELETE CASCADE,
    "entity_id" bytea DEFAULT NULL,
    "date" timestamp with time zone NOT NULL,
    "note" text NOT NULL,
    PRIMARY KEY ("id")
);
CREATE INDEX "KEY_server_note_server_id" ON "server_note_v3" ("server_id");

do $$
BEGIN
IF EXISTS (
    SELECT NULL 
    FROM information_schema.TABLES
    WHERE
        TABLE_SCHEMA = current_database() AND
        TABLE_NAME   = 'server_note'
)
THEN
    INSERT INTO server_note_v3 ("server_id", "entity_id", "date", "note")
        SELECT s.id, e.id, sn.date, sn.note
        FROM server_note sn
        INNER JOIN server s ON sn.server_id=s.migration_id
        LEFT JOIN entity e ON sn.entity_id=e.migration_id;

    DROP TABLE "server_note";
END IF;
END
$$;

ALTER TABLE server_note_v3
    RENAME TO server_note;

-- create or migrate event table
CREATE TYPE event_type_v3 AS ENUM ('server', 'entity');

CREATE TABLE "event_v3" (
    "id" bytea NOT NULL DEFAULT GEN_UUID(),
    "actor_id" bytea,
    "date" timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "details" text NOT NULL,
    "type" event_type_v3 NOT NULL,
    "object_id" bytea,
    PRIMARY KEY ("id")
);
CREATE INDEX "KEY_event_object_id" ON "event_v3" ("object_id");
CREATE INDEX "KEY_event_actor_id" ON "event_v3" ("actor_id");

CREATE TABLE "event_migration" (
    "id" bytea NOT NULL,
    "ref_id" integer NOT NULL,
    PRIMARY KEY ("id")
);

do $$
BEGIN
IF EXISTS (
    SELECT NULL 
    FROM information_schema.TABLES
    WHERE
        TABLE_SCHEMA = current_database() AND
        TABLE_NAME   = 'entity_event'
)
THEN
    INSERT INTO event_migration 
        SELECT COALESCE(e.id, GEN_UUID()), sub.id
        FROM (SELECT entity_id AS id FROM entity_event 
            union 
            SELECT actor_id AS id FROM entity_event) sub
        LEFT JOIN entity e ON sub.id=e.migration_id
        WHERE sub.id NOT IN (SELECT ref_id FROM event_migration)
        GROUP BY sub.id, e.id;

    INSERT INTO event_v3 ("actor_id", "date", "details", "type", "object_id")
        SELECT em2.id, ee.date, ee.details, 'entity', em1.id
        FROM entity_event ee
        LEFT JOIN event_migration em1 ON ee.entity_id=em1.ref_id
        LEFT JOIN event_migration em2 ON ee.actor_id=em2.ref_id;        

    DROP TABLE "entity_event";
END IF;
END
$$;

do $$
BEGIN
IF EXISTS (
    SELECT NULL 
    FROM information_schema.TABLES
    WHERE
        TABLE_SCHEMA = current_database() AND
        TABLE_NAME   = 'server_event'
)
THEN
    CREATE TABLE "server_migration" (
        "id" bytea NOT NULL,
        "ref_id" integer NOT NULL,
        PRIMARY KEY ("id")
    );

    INSERT INTO server_migration 
        SELECT COALESCE(s.id, GEN_UUID()), sub.id
        FROM (SELECT server_id AS id FROM server_event) sub
        LEFT JOIN server s ON sub.id=s.migration_id
        GROUP BY sub.id, s.id;

    INSERT INTO event_migration 
        SELECT COALESCE(e.id, GEN_UUID()), sub.id
        FROM (SELECT actor_id AS id FROM server_event) sub
        LEFT JOIN entity e ON sub.id=e.migration_id
        WHERE sub.id NOT IN (SELECT ref_id FROM event_migration)
        GROUP BY sub.id, e.id;

    INSERT INTO event_v3 ("actor_id", "date", "details", "type", "object_id")
        SELECT e.id, se.date, se.details, 'server', s.id
        FROM server_event se
        LEFT JOIN server_migration s ON se.server_id=s.ref_id
        LEFT JOIN event_migration e ON se.actor_id=e.ref_id;

    DROP TABLE "server_migration";
    DROP TABLE "server_event";
END IF;
END
$$;

do $$
BEGIN
IF EXISTS (
    SELECT NULL 
    FROM information_schema.TABLES
    WHERE
        TABLE_SCHEMA = current_database() AND
        TABLE_NAME   = 'group_event'
)
THEN
    INSERT INTO event_migration 
        SELECT COALESCE(e.id, GEN_UUID()), sub.id
        FROM (SELECT entity_id AS id FROM group_event 
            union 
            SELECT ee.group AS id FROM group_event ee) sub
        LEFT JOIN entity e ON sub.id=e.migration_id
        WHERE sub.id NOT IN (SELECT ref_id FROM event_migration)
        GROUP BY sub.id, e.id;

    INSERT INTO event_v3 ("actor_id", "date", "details", "type", "object_id")
        SELECT em2.id, ge.date, ge.details, 'entity', em1.id
        FROM group_event ge
        LEFT JOIN event_migration em1 ON ge.group=em1.ref_id
        LEFT JOIN event_migration em2 ON ge.entity_id=em2.ref_id;

    DROP TABLE "group_event";
END IF;
END
$$;

DROP TABLE "event_migration";
ALTER TABLE event_v3
    RENAME TO event;
ALTER TYPE event_type_v3 
    RENAME TO event_type;

-- create or migrate server_admin table
CREATE TABLE "server_admin_v3" (
    "server_id" bytea NOT NULL REFERENCES server(id) ON DELETE CASCADE,
    "entity_id" bytea NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    PRIMARY KEY ("server_id","entity_id")
);
CREATE INDEX "FK_server_admin_entity" ON "server_admin_v3" ("entity_id");

do $$
BEGIN
IF EXISTS (
    SELECT NULL 
    FROM information_schema.TABLES
    WHERE
        TABLE_SCHEMA = current_database() AND
        TABLE_NAME   = 'server_admin'
)
THEN
    INSERT INTO server_admin_v3
        SELECT s.id, e.id
        FROM server_admin sa
        INNER JOIN server s ON sa.server_id=s.migration_id
        LEFT JOIN entity e ON sa.entity_id=e.migration_id;

    DROP TABLE "server_admin";
END IF;
END
$$;

ALTER TABLE server_admin_v3
    RENAME TO server_admin;

-- create or migrate access table
CREATE TABLE "access_v3" (
    "id" bytea NOT NULL DEFAULT GEN_UUID(),
    "source_id" bytea NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    "dest_id" bytea NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    "grant_date" timestamp with time zone NOT NULL,
    "granted_by" bytea,
    "migration_id" integer DEFAULT NULL,
    PRIMARY KEY ("id"),
    CONSTRAINT "source_entity_id_dest_entity_id" UNIQUE("source_id", "dest_id")
);
CREATE INDEX "FK_access_entity_2" ON "access_v3" ("dest_id");
CREATE INDEX "FK_access_entity_3" ON "access_v3" ("granted_by");

do $$
BEGIN
IF EXISTS (
    SELECT NULL 
    FROM information_schema.TABLES
    WHERE
        TABLE_SCHEMA = current_database() AND
        TABLE_NAME   = 'access'
)
THEN
    INSERT INTO access_v3 ("source_id", "dest_id", "grant_date", "granted_by", "migration_id")
        SELECT e1.id, e2.id, a.grant_date, e3.id, a.id
        FROM access a
        INNER JOIN entity e1 ON a.source_entity_id=e1.migration_id
        INNER JOIN entity e2 ON a.dest_entity_id=e2.migration_id
        LEFT JOIN entity e3 ON a.granted_by=e3.migration_id;

    DROP TABLE "access";
END IF;
END
$$;

ALTER TABLE access_v3
    RENAME TO access;

-- create or migrate access_option table
CREATE TYPE access_option_type_v3 AS ENUM ('command', 'from', 'environment', 'no-agent-forwarding', 'no-port-forwarding', 'no-pty', 'no-X11-forwarding', 'no-user-rc');

CREATE TABLE "access_option_v3" (
    "id" bytea NOT NULL DEFAULT GEN_UUID(),
    "access_id" bytea NOT NULL REFERENCES access(id) ON DELETE CASCADE,
    "option" access_option_type_v3 NOT NULL,
    "value" text,
    PRIMARY KEY ("id"),
    CONSTRAINT "access_id_option" UNIQUE("access_id", "option")
);

do $$
BEGIN
IF EXISTS (
    SELECT NULL 
    FROM information_schema.TABLES
    WHERE
        TABLE_SCHEMA = current_database() AND
        TABLE_NAME   = 'access_option'
)
THEN
    INSERT INTO access_option_v3
        SELECT null, a.id, ao.option::text::access_option_type_v3, ao.value
        FROM access_option ao
        INNER JOIN access a ON ao.access_id=a.migration_id;

    DROP TABLE "access_option";
    DROP TYPE IF EXISTS access_option_option;
END IF;
END
$$;

ALTER TABLE access_option_v3
    RENAME TO access_option;
ALTER TYPE access_option_type_v3 
    RENAME TO access_option_type;

-- create or migrate public_key table
CREATE TABLE "public_key_v3" (
    "id" bytea NOT NULL DEFAULT GEN_UUID(),
    "entity_id" bytea NOT NULL,
    "type" varchar(30) NOT NULL,
    "keydata" text NOT NULL,
    "comment" text DEFAULT NULL,
    "keysize" integer DEFAULT NULL,
    "fingerprint_md5" bytea DEFAULT NULL,
    "fingerprint_sha256" bytea DEFAULT NULL UNIQUE,
    "randomart_md5" varchar(220),
    "randomart_sha256" varchar(220),
    "upload_date" timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "active" BOOLEAN NOT NULL DEFAULT TRUE,
    PRIMARY KEY ("id")
);
CREATE INDEX "FK_public_key_entity" ON "public_key_v3" ("entity_id");

do $$
BEGIN
IF EXISTS (
    SELECT NULL 
    FROM information_schema.TABLES
    WHERE
        TABLE_SCHEMA = current_database() AND
        TABLE_NAME   = 'public_key'
)
THEN
    INSERT INTO public_key_v3 ("entity_id", "type", "keydata", "comment", "keysize", "fingerprint_md5", "fingerprint_sha256", "randomart_md5", "randomart_sha256", "upload_date", "active")
        SELECT e.id, pk.type, pk.keydata, pk.comment, pk.keysize, 
            (CASE WHEN pk.fingerprint_md5 IS NULL THEN NULL ELSE decode(replace(pk.fingerprint_md5, ':', ''), 'hex') END),
            (CASE WHEN pk.fingerprint_sha256 IS NULL THEN NULL ELSE decode(CONCAT(pk.fingerprint_sha256, '='), 'base64') END),
            pk.randomart_md5, pk.randomart_sha256, pk.upload_date, pk.active
        FROM public_key pk
        INNER JOIN entity e ON pk.entity_id=e.migration_id;

    DROP TABLE "public_key";
END IF;
END
$$;

ALTER TABLE public_key_v3
    RENAME TO public_key;

-- remove deprecated tables
DROP TABLE IF EXISTS "access_request";
DROP TABLE IF EXISTS "entity_admin";
DROP TABLE IF EXISTS "migration";
DROP TABLE IF EXISTS "public_key_dest_rule";
DROP TABLE IF EXISTS "public_key_signature";
DROP TABLE IF EXISTS "server_ldap_access_option";
DROP TYPE IF EXISTS server_ldap_access_option_option;
DROP TABLE IF EXISTS "user_alert";

-- remove deleted
DELETE from "users" where active=false;
ALTER TABLE "users"
    DROP "active";

DELETE from "groups" where active=false;
ALTER TABLE "groups"
    DROP "active";

DELETE from "server" where deleted=true;
ALTER TABLE server
    DROP "deleted";

DELETE from "server_account" where active=false;
ALTER TABLE server_account
    DROP "active";

-- remove migration columns
ALTER TABLE entity
    DROP "migration_id";
ALTER TABLE server
    DROP "migration_id";
ALTER TABLE access
    DROP "migration_id";
