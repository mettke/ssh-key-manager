-- Your SQL goes here
-- remove foreign keys
create procedure remove_foreign_keys() BEGIN
IF EXISTS (
    SELECT NULL 
    FROM information_schema.TABLE_CONSTRAINTS
    WHERE
        CONSTRAINT_SCHEMA = DATABASE() AND
        CONSTRAINT_TYPE = "FOREIGN KEY"
)
THEN
    ALTER TABLE `access`
        DROP FOREIGN KEY `FK_access_entity`,
        DROP FOREIGN KEY `FK_access_entity_2`,
        DROP FOREIGN KEY `FK_access_entity_3`;
    ALTER TABLE `access_option`
        DROP FOREIGN KEY `FK_access_option_access`;
    ALTER TABLE `access_request`
        DROP FOREIGN KEY `FK_access_request_entity`,
        DROP FOREIGN KEY `FK_access_request_entity_2`,
        DROP FOREIGN KEY `FK_access_request_entity_3`;
    ALTER TABLE `entity_admin`
        DROP FOREIGN KEY `FK_entity_admin_entity`,
        DROP FOREIGN KEY `FK_entity_admin_entity_2`;
    ALTER TABLE `entity_event`
        DROP FOREIGN KEY `FK_entity_event_actor_id`,
        DROP FOREIGN KEY `FK_entity_event_entity_id`;
    ALTER TABLE `group`
        DROP FOREIGN KEY `FK_group_entity`;
    ALTER TABLE `group_event`
        DROP FOREIGN KEY `FK_group_event_entity`,
        DROP FOREIGN KEY `FK_group_event_group`;
    ALTER TABLE `group_member`
        DROP FOREIGN KEY `FK_group_member_entity`,
        DROP FOREIGN KEY `FK_group_member_entity_2`,
        DROP FOREIGN KEY `FK_group_member_group`;
    ALTER TABLE `public_key`
        DROP FOREIGN KEY `FK_public_key_entity`;
    ALTER TABLE `public_key_dest_rule`
        DROP FOREIGN KEY `FK_public_key_dest_rule_public_key`;
    ALTER TABLE `public_key_signature`
        DROP FOREIGN KEY `FK_public_key_signature_public_key`;
    ALTER TABLE `server_account`
        DROP FOREIGN KEY `FK_server_account_entity`,
        DROP FOREIGN KEY `FK_server_account_server`;
    ALTER TABLE `server_admin`
        DROP FOREIGN KEY `FK_server_admin_entity`,
        DROP FOREIGN KEY `FK_server_admin_server`;
    ALTER TABLE `server_event`
        DROP FOREIGN KEY `FK_server_event_actor_id`,
        DROP FOREIGN KEY `FK_server_log_server`;
    ALTER TABLE `server_ldap_access_option`
        DROP FOREIGN KEY `FK_server_ldap_access_option_server`;
    ALTER TABLE `server_note`
        DROP FOREIGN KEY `FK_server_note_entity`,
        DROP FOREIGN KEY `FK_server_note_server`;
    ALTER TABLE `sync_request`
        DROP FOREIGN KEY `FK_sync_request_server`;
    ALTER TABLE `user`
        DROP FOREIGN KEY `FK_user_entity`;
    ALTER TABLE `user_alert`
        DROP FOREIGN KEY `FK_user_alert_entity`;
END IF;
END;
CALL remove_foreign_keys();
DROP PROCEDURE remove_foreign_keys;

DROP FUNCTION IF EXISTS GEN_UUID;
create function GEN_UUID()
RETURNS Binary(16)
BEGIN
    RETURN UUID_TO_BIN(UUID(), 1);
END;

-- create or migrate entity table
CREATE TABLE `entity_v3` (
    `id` Binary(16) NOT NULL,
    `type` enum('user','server account', 'group') NOT NULL,
    `migration_id` int(10) DEFAULT NULL,
    PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TRIGGER `entity_before_insert` 
BEFORE INSERT ON `entity_v3` FOR EACH ROW 
BEGIN
    IF new.id IS NULL THEN
        SET new.id = GEN_UUID();
    END IF;
END;

create procedure migrate_table() 
BEGIN
    IF EXISTS (
        SELECT NULL 
        FROM information_schema.TABLES
        WHERE
            TABLE_SCHEMA = DATABASE() AND
            TABLE_NAME   = 'entity'
    )
    THEN
        INSERT INTO entity_v3 SELECT null, `type`, `id` FROM entity;
        DROP TABLE `entity`;
    END IF;
END;
CALL migrate_table();
DROP PROCEDURE migrate_table;

RENAME TABLE `entity_v3` TO `entity`;

-- create or migrate users table
CREATE TABLE `users_v3` (
    `entity_id` Binary(16) NOT NULL,
    `uid` varchar(50) NOT NULL,
    `name` varchar(100) DEFAULT NULL,
    `email` varchar(100) DEFAULT NULL,
    `password` varchar(250) DEFAULT NULL,
    `active` tinyint(1) unsigned NOT NULL DEFAULT '1',
    `type` enum('user', 'admin', 'superuser') NOT NULL DEFAULT 'user',
    PRIMARY KEY (`entity_id`),
    UNIQUE KEY `KEY_user_uid` (`uid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

create procedure migrate_table() 
BEGIN
    IF EXISTS (
        SELECT NULL 
        FROM information_schema.TABLES
        WHERE
            TABLE_SCHEMA = DATABASE() AND
            TABLE_NAME   = 'user'
    )
    THEN
        INSERT INTO users_v3 (`entity_id`,`uid`,`name`,`email`,`active`,`type`)
            SELECT e.id, u.uid, u.name, u.email, u.active - u.force_disable, 'user' FROM user u 
            INNER JOIN entity e ON u.entity_id=e.migration_id
            WHERE u.admin = '0';
        INSERT INTO users_v3 (`entity_id`,`uid`,`name`,`email`,`active`,`type`)
            SELECT e.id, u.uid, u.name, u.email, u.active - u.force_disable, 'admin' FROM user u 
            INNER JOIN entity e ON u.entity_id=e.migration_id
            WHERE u.admin = '1';
        UPDATE users_v3 SET type = 'superuser'
            WHERE uid = 'keys-sync'; 
        DROP TABLE `user`;
    ELSE
        SET @uuid = GEN_UUID();
        INSERT INTO entity SET id = @uuid, type = 'user';
        INSERT INTO users_v3 SET entity_id = @uuid, uid = 'root', name = 'Default Administrator', type = 'superuser';
    END IF;
END;
CALL migrate_table();
DROP PROCEDURE migrate_table;

RENAME TABLE `users_v3` TO `users`;
ALTER TABLE `users`
    ADD CONSTRAINT `FK_users_entity_id` FOREIGN KEY (`entity_id`) REFERENCES `entity` (`id`) ON DELETE CASCADE;

-- create or migrate groups table
CREATE TABLE `groups_v3` (
    `entity_id` Binary(16) NOT NULL,
    `name` varchar(150) NOT NULL,
    `active` tinyint(1) unsigned NOT NULL DEFAULT '1',
    `system`  BOOLEAN NOT NULL DEFAULT FALSE,
    `oauth_scope` varchar(150) DEFAULT NULL,
    `ldap_group` varchar(150) DEFAULT NULL,
    PRIMARY KEY (`entity_id`),
    UNIQUE KEY `KEY_groups_name` (`name`),
    KEY `KEY_groups_oauth_scope` (`oauth_scope`),
    KEY `KEY_groups_ldap_group` (`ldap_group`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

create procedure migrate_table() 
BEGIN
    IF EXISTS (
        SELECT NULL 
        FROM information_schema.TABLES
        WHERE
            TABLE_SCHEMA = DATABASE() AND
            TABLE_NAME   = 'group'
    )
    THEN
        SET @db = DATABASE();
        SET @q = CONCAT('RENAME TABLE ', @db, '.group TO `group_v2`');
        PREPARE stmt FROM @q;
        EXECUTE stmt;
        DEALLOCATE PREPARE stmt;

        INSERT INTO groups_v3 (`entity_id`, `name`, `active`, `system`) 
            SELECT e.id, g.name, g.active, g.system FROM group_v2 g 
            INNER JOIN entity e ON g.entity_id=e.migration_id;

        DROP TABLE `group_v2`;
    END IF;
END;
CALL migrate_table();
DROP PROCEDURE migrate_table;

RENAME TABLE `groups_v3` TO `groups`;
ALTER TABLE `groups`
    ADD CONSTRAINT `FK_groups_entity_id` FOREIGN KEY (`entity_id`) REFERENCES `entity` (`id`) ON DELETE CASCADE;

-- create or migrate server_admin table
CREATE TABLE `group_admin` (
    `group_id` Binary(16) NOT NULL,
    `admin_id` Binary(16) NOT NULL,
    PRIMARY KEY (`group_id`,`admin_id`),
    KEY `FK_group_admin_entity` (`group_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

create procedure migrate_table() 
BEGIN
    IF EXISTS (
        SELECT NULL 
        FROM information_schema.TABLES
        WHERE
            TABLE_SCHEMA = DATABASE() AND
            TABLE_NAME   = 'entity_admin'
    )
    THEN
        INSERT INTO group_admin
            SELECT e1.id, e2.id
            FROM entity_admin sa
            INNER JOIN entity e1 ON sa.entity_id=e1.migration_id
            LEFT JOIN entity e2 ON sa.admin=e2.migration_id
            WHERE e1.type = 'group';
    END IF;
END;
CALL migrate_table();
DROP PROCEDURE migrate_table;

ALTER TABLE `group_admin`
    ADD CONSTRAINT `FK_group_admin_server_id` FOREIGN KEY (`group_id`) REFERENCES `entity` (`id`) ON DELETE CASCADE,
    ADD CONSTRAINT `FK_group_admin_entity_id` FOREIGN KEY (`admin_id`) REFERENCES `entity` (`id`) ON DELETE CASCADE;

-- create or migrate group_member table
CREATE TABLE `group_member_v3` (
    `group_id` Binary(16) NOT NULL,
    `member_id` Binary(16) NOT NULL,
    `add_date` datetime NOT NULL,
    `added_by` Binary(16) DEFAULT NULL,
    PRIMARY KEY (`group_id`, `member_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 ROW_FORMAT=COMPACT;

create procedure migrate_table() 
BEGIN
    IF EXISTS (
        SELECT NULL 
        FROM information_schema.TABLES
        WHERE
            TABLE_SCHEMA = DATABASE() AND
            TABLE_NAME   = 'group_member'
    )
    THEN
        INSERT INTO group_member_v3 SELECT e1.id, e2.id, gm.add_date, e3.id
            FROM group_member gm 
            INNER JOIN entity e1 ON gm.group=e1.migration_id
            INNER JOIN entity e2 ON gm.entity_id=e2.migration_id
            LEFT JOIN entity e3 ON gm.added_by=e3.migration_id;

        DROP TABLE `group_member`;
    END IF;
END;
CALL migrate_table();
DROP PROCEDURE migrate_table;

RENAME TABLE `group_member_v3` TO `group_member`;
ALTER TABLE `group_member`
    ADD CONSTRAINT `FK_group_member_group_id` FOREIGN KEY (`group_id`) REFERENCES `entity` (`id`) ON DELETE CASCADE,
    ADD CONSTRAINT `FK_group_member_member_id` FOREIGN KEY (`member_id`) REFERENCES `entity` (`id`) ON DELETE CASCADE;

-- create or migrate server table
CREATE TABLE `server_v3` (
    `id` Binary(16) NOT NULL,
    `hostname` varchar(150) NOT NULL,
    `ip_address` varchar(64) DEFAULT NULL, -- TODO: 128 bits for ipv6 and 32 bits for ipv4
    `name` varchar(100) DEFAULT NULL,
    `deleted` tinyint(1) unsigned NOT NULL DEFAULT '0',
    `key_management` enum('none', 'keys', 'other') NOT NULL DEFAULT 'keys',
    `authorization` enum('manual', 'automatic') NOT NULL DEFAULT 'manual',
    `sync_status` enum('not synced yet', 'sync success', 'sync failure', 'sync warning') NOT NULL DEFAULT 'not synced yet',
    `rsa_key_fingerprint` char(32) DEFAULT NULL, -- TODO ?
    `port` int(10) unsigned NOT NULL DEFAULT 22,
    `migration_id` int(10) DEFAULT NULL,
    PRIMARY KEY (`id`),
    KEY `KEY_server_hostname` (`hostname`),
    KEY `KEY_server_ip_address` (`ip_address`),
    KEY `KEY_server_rsa_key_fingerprint` (`rsa_key_fingerprint`),
    KEY `KEY_server_port` (`port`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TRIGGER `server_before_insert` 
BEFORE INSERT ON `server_v3` FOR EACH ROW 
BEGIN
    IF new.id IS NULL THEN
        SET new.id = GEN_UUID();
    END IF;
END;

create procedure migrate_table() 
BEGIN
    IF EXISTS (
        SELECT NULL 
        FROM information_schema.TABLES
        WHERE
            TABLE_SCHEMA = DATABASE() AND
            TABLE_NAME   = 'server'
    )
    THEN
        INSERT INTO server_v3
            SELECT null, s.hostname, s.ip_address, null, s.deleted, s.key_management, s.authorization, s.sync_status, s.rsa_key_fingerprint, s.port, s.id 
            FROM server s
            WHERE s.key_management!="decommissioned" AND s.authorization!="automatic LDAP" AND s.authorization!="manual LDAP";

        INSERT INTO server_v3
            SELECT null, s.hostname, s.ip_address, null, 1, 'none', s.authorization, s.sync_status, s.rsa_key_fingerprint, s.port, s.id 
            FROM server s
            WHERE s.key_management="decommissioned" AND s.authorization!="automatic LDAP" AND s.authorization!="manual LDAP";

        INSERT INTO server_v3
            SELECT null, s.hostname, s.ip_address, null, s.deleted, s.key_management, 'automatic', s.sync_status, s.rsa_key_fingerprint, s.port, s.id 
            FROM server s
            WHERE s.key_management!="decommissioned" AND (s.authorization="automatic LDAP" OR s.authorization="manual LDAP");

        INSERT INTO server_v3
            SELECT null, s.hostname, s.ip_address, null, 1, 'none', 'automatic', s.sync_status, s.rsa_key_fingerprint, s.port, s.id 
            FROM server s
            WHERE s.key_management="decommissioned" AND (s.authorization="automatic LDAP" OR s.authorization="manual LDAP");

        DROP TABLE `server`;
    END IF;
END;
CALL migrate_table();
DROP PROCEDURE migrate_table;

RENAME TABLE `server_v3` TO `server`;

-- create or migrate server_account table
CREATE TABLE `server_account_v3` (
    `entity_id` Binary(16) NOT NULL,
    `server_id` Binary(16) NOT NULL,
    `name` varchar(100) DEFAULT NULL,
    `sync_status` enum('not synced yet', 'sync success', 'sync failure', 'sync warning') NOT NULL DEFAULT 'not synced yet',
    `active` tinyint(1) unsigned NOT NULL DEFAULT '1',
    PRIMARY KEY (`entity_id`),
    UNIQUE KEY `KEY_server_account_combination` (`server_id`, `name`),
    KEY `KEY_server_account_server_id` (`server_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

create procedure migrate_table() 
BEGIN
    IF EXISTS (
        SELECT NULL 
        FROM information_schema.TABLES
        WHERE
            TABLE_SCHEMA = DATABASE() AND
            TABLE_NAME   = 'server_account'
    )
    THEN
        INSERT INTO server_account_v3
            SELECT e.id, s.id, sa.name, sa.sync_status, sa.active
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

        DROP TABLE `server_account`;
    END IF;
END;
CALL migrate_table();
DROP PROCEDURE migrate_table;

RENAME TABLE `server_account_v3` TO `server_account`;
ALTER TABLE `server_account`
    ADD CONSTRAINT `FK_server_account_entity_id` FOREIGN KEY (`entity_id`) REFERENCES `entity` (`id`) ON DELETE CASCADE,
    ADD CONSTRAINT `FK_server_account_server_id` FOREIGN KEY (`server_id`) REFERENCES `server` (`id`) ON DELETE CASCADE;

-- create or migrate sync_request table
CREATE TABLE `sync_request_v3` (
    `id` Binary(16) NOT NULL,
    `server_id` Binary(16) NOT NULL,
    `account_id` Binary(16) DEFAULT NULL,
    `processing` tinyint(1) unsigned NOT NULL DEFAULT '0',
    PRIMARY KEY (`id`),
    UNIQUE KEY `KEY_sync_request_combination` (`server_id`,`account_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TRIGGER `sync_request_before_insert` 
BEFORE INSERT ON `sync_request_v3` FOR EACH ROW 
BEGIN
    IF new.id IS NULL THEN
        SET new.id = GEN_UUID();
    END IF;
END;

create procedure migrate_table() 
BEGIN
    IF EXISTS (
        SELECT NULL 
        FROM information_schema.TABLES
        WHERE
            TABLE_SCHEMA = DATABASE() AND
            TABLE_NAME   = 'sync_request'
    )
    THEN
        INSERT INTO sync_request_v3
            SELECT null, s.id, sa.entity_id, sr.processing
            FROM sync_request sr
            INNER JOIN server s ON sr.server_id=s.migration_id
            LEFT JOIN server_account sa ON sr.account_name=sa.name;

        DROP TABLE `sync_request`;
    END IF;
END;
CALL migrate_table();
DROP PROCEDURE migrate_table;

RENAME TABLE `sync_request_v3` TO `sync_request`;
ALTER TABLE `sync_request`
    ADD CONSTRAINT `FK_sync_request_server_id` FOREIGN KEY (`server_id`) REFERENCES `server` (`id`) ON DELETE CASCADE,
    ADD CONSTRAINT `FK_sync_request_account_id` FOREIGN KEY (`account_id`) REFERENCES `entity` (`id`) ON DELETE CASCADE;

-- create or migrate server_note table
CREATE TABLE `server_note_v3` (
    `id` Binary(16) NOT NULL,
    `server_id` Binary(16) NOT NULL,
    `entity_id` Binary(16) DEFAULT NULL,
    `date` datetime NOT NULL,
    `note` mediumtext NOT NULL,
    PRIMARY KEY (`id`),
    KEY `KEY_server_note_server_id` (`server_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TRIGGER `server_note_before_insert` 
BEFORE INSERT ON `server_note_v3` FOR EACH ROW 
BEGIN
    IF new.id IS NULL THEN
        SET new.id = GEN_UUID();
    END IF;
END;

create procedure migrate_table() 
BEGIN
    IF EXISTS (
        SELECT NULL 
        FROM information_schema.TABLES
        WHERE
            TABLE_SCHEMA = DATABASE() AND
            TABLE_NAME   = 'server_note'
    )
    THEN
        INSERT INTO server_note_v3
            SELECT null, s.id, e.id, sn.date, sn.note
            FROM server_note sn
            INNER JOIN server s ON sn.server_id=s.migration_id
            LEFT JOIN entity e ON sn.entity_id=e.migration_id;

        DROP TABLE `server_note`;
    END IF;
END;
CALL migrate_table();
DROP PROCEDURE migrate_table;

RENAME TABLE `server_note_v3` TO `server_note`;
ALTER TABLE `server_note`
    ADD CONSTRAINT `FK_server_note_server_id` FOREIGN KEY (`server_id`) REFERENCES `server` (`id`) ON DELETE CASCADE;

-- create or migrate event table
CREATE TABLE `event_v3` (
    `id` Binary(16) NOT NULL,
    `actor_id` Binary(16),
    `date` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `details` mediumtext NOT NULL,
    `type` enum('server', 'entity') NOT NULL,
    `object_id` Binary(16),
    PRIMARY KEY (`id`),
    KEY `KEY_event_object_id` (`object_id`),
    KEY `KEY_event_actor_id` (`actor_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TRIGGER `event_before_insert` 
BEFORE INSERT ON `event_v3` FOR EACH ROW 
BEGIN
    IF new.id IS NULL THEN
        SET new.id = GEN_UUID();
    END IF;
END;

CREATE TABLE `event_migration` (
    `id` Binary(16) NOT NULL,
    `ref_id` int(10) NOT NULL,
    PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

create procedure migrate_table() 
BEGIN
    IF EXISTS (
        SELECT NULL 
        FROM information_schema.TABLES
        WHERE
            TABLE_SCHEMA = DATABASE() AND
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
    
        INSERT INTO event_v3
            SELECT null, em2.id, null, ee.date, ee.details, 'entity', em1.id, null
            FROM entity_event ee
            LEFT JOIN event_migration em1 ON ee.entity_id=em1.ref_id
            LEFT JOIN event_migration em2 ON ee.actor_id=em2.ref_id;

        DROP TABLE `entity_event`;
    END IF;
END;
CALL migrate_table();
DROP PROCEDURE migrate_table;

create procedure migrate_table() 
BEGIN
    IF EXISTS (
        SELECT NULL 
        FROM information_schema.TABLES
        WHERE
            TABLE_SCHEMA = DATABASE() AND
            TABLE_NAME   = 'server_event'
    )
    THEN
        CREATE TABLE `server_migration` (
            `id` Binary(16) NOT NULL,
            `ref_id` int(10) NOT NULL,
            PRIMARY KEY (`id`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

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

        INSERT INTO event_v3
            SELECT null, e.id, null, se.date, se.details, 'server', s.id, null
            FROM server_event se
            LEFT JOIN server_migration s ON se.server_id=s.ref_id
            LEFT JOIN event_migration e ON se.actor_id=e.ref_id;

        DROP TABLE `server_migration`;
        DROP TABLE `server_event`;
    END IF;
END;
CALL migrate_table();
DROP PROCEDURE migrate_table;

create procedure migrate_table() 
BEGIN
    IF EXISTS (
        SELECT NULL 
        FROM information_schema.TABLES
        WHERE
            TABLE_SCHEMA = DATABASE() AND
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

        INSERT INTO event_v3
            SELECT null, em2.id, null, ge.date, ge.details, 'entity', em1.id, null
            FROM group_event ge
            LEFT JOIN event_migration em1 ON ge.group=em1.ref_id
            LEFT JOIN event_migration em2 ON ge.entity_id=em2.ref_id;

        DROP TABLE `group_event`;
    END IF;
END;
CALL migrate_table();
DROP PROCEDURE migrate_table;

DROP TABLE `event_migration`;
RENAME TABLE `event_v3` TO `event`;

-- create or migrate server_admin table
CREATE TABLE `server_admin_v3` (
    `server_id` Binary(16) NOT NULL,
    `entity_id` Binary(16) NOT NULL,
    PRIMARY KEY (`server_id`,`entity_id`),
    KEY `FK_server_admin_entity` (`entity_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

create procedure migrate_table() 
BEGIN
    IF EXISTS (
        SELECT NULL 
        FROM information_schema.TABLES
        WHERE
            TABLE_SCHEMA = DATABASE() AND
            TABLE_NAME   = 'server_admin'
    )
    THEN
        INSERT INTO server_admin_v3
            SELECT s.id, e.id
            FROM server_admin sa
            INNER JOIN server s ON sa.server_id=s.migration_id
            LEFT JOIN entity e ON sa.entity_id=e.migration_id;

        DROP TABLE `server_admin`;
    END IF;
END;
CALL migrate_table();
DROP PROCEDURE migrate_table;

RENAME TABLE `server_admin_v3` TO `server_admin`;
ALTER TABLE `server_admin`
    ADD CONSTRAINT `FK_server_admin_server_id` FOREIGN KEY (`server_id`) REFERENCES `server` (`id`) ON DELETE CASCADE,
    ADD CONSTRAINT `FK_server_admin_entity_id` FOREIGN KEY (`entity_id`) REFERENCES `entity` (`id`) ON DELETE CASCADE;

-- create or migrate access table
CREATE TABLE `access_v3` (
    `id` Binary(16) NOT NULL,
    `source_id` Binary(16) NOT NULL,
    `dest_id` Binary(16) NOT NULL,
    `grant_date` datetime NOT NULL,
    `granted_by` Binary(16),
    `migration_id` int(10) DEFAULT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `source_entity_id_dest_entity_id` (`source_id`, `dest_id`),
    KEY `FK_access_entity_2` (`dest_id`),
    KEY `FK_access_entity_3` (`granted_by`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 ROW_FORMAT=COMPACT;

CREATE TRIGGER `access_before_insert` 
BEFORE INSERT ON `access_v3` FOR EACH ROW 
BEGIN
    IF new.id IS NULL THEN
        SET new.id = GEN_UUID();
    END IF;
END;

create procedure migrate_table() 
BEGIN
    IF EXISTS (
        SELECT NULL 
        FROM information_schema.TABLES
        WHERE
            TABLE_SCHEMA = DATABASE() AND
            TABLE_NAME   = 'access'
    )
    THEN
        INSERT INTO access_v3
            SELECT null, e1.id, e2.id, a.grant_date, e3.id, a.id
            FROM access a
            INNER JOIN entity e1 ON a.source_entity_id=e1.migration_id
            INNER JOIN entity e2 ON a.dest_entity_id=e2.migration_id
            LEFT JOIN entity e3 ON a.granted_by=e3.migration_id;

        DROP TABLE `access`;
    END IF;
END;
CALL migrate_table();
DROP PROCEDURE migrate_table;

RENAME TABLE `access_v3` TO `access`;
ALTER TABLE `access`
    ADD CONSTRAINT `FK_access_source_id` FOREIGN KEY (`source_id`) REFERENCES `entity` (`id`) ON DELETE CASCADE,
    ADD CONSTRAINT `FK_access_dest_id` FOREIGN KEY (`dest_id`) REFERENCES `entity` (`id`) ON DELETE CASCADE;

-- create or migrate access_option table
CREATE TABLE `access_option_v3` (
    `id` Binary(16) NOT NULL,
    `access_id` Binary(16) NOT NULL,
    `option` enum('command', 'from', 'environment', 'no-agent-forwarding', 'no-port-forwarding', 'no-pty', 'no-X11-forwarding', 'no-user-rc') NOT NULL,
    `value` text,
    PRIMARY KEY (`id`),
    UNIQUE KEY `access_id_option` (`access_id`, `option`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TRIGGER `access_option_before_insert` 
BEFORE INSERT ON `access_option_v3` FOR EACH ROW 
BEGIN
    IF new.id IS NULL THEN
        SET new.id = GEN_UUID();
    END IF;
END;

create procedure migrate_table() 
BEGIN
    IF EXISTS (
        SELECT NULL 
        FROM information_schema.TABLES
        WHERE
            TABLE_SCHEMA = DATABASE() AND
            TABLE_NAME   = 'access_option'
    )
    THEN
        INSERT INTO access_option_v3
            SELECT null, a.id, ao.option, ao.value
            FROM access_option ao
            INNER JOIN access a ON ao.access_id=a.migration_id;

        DROP TABLE `access_option`;
    END IF;
END;
CALL migrate_table();
DROP PROCEDURE migrate_table;

RENAME TABLE `access_option_v3` TO `access_option`;
ALTER TABLE `access_option`
    ADD CONSTRAINT `FK_access_option_access_id` FOREIGN KEY (`access_id`) REFERENCES `access` (`id`) ON DELETE CASCADE;

-- create or migrate public_key table
CREATE TABLE `public_key_v3` (
    `id` Binary(16) NOT NULL,
    `entity_id` Binary(16) NOT NULL,
    `type` varchar(30) NOT NULL,
    `keydata` mediumtext NOT NULL,
    `comment` mediumtext DEFAULT NULL,
    `keysize` smallint DEFAULT NULL,
    `fingerprint_md5` Binary(16) DEFAULT NULL,
    `fingerprint_sha256` Binary(32) DEFAULT NULL,
    `randomart_md5` varchar(220),
    `randomart_sha256` varchar(220),
    `upload_date` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `active` BOOLEAN NOT NULL DEFAULT TRUE,
    PRIMARY KEY (`id`),
    KEY `FK_public_key_entity` (`entity_id`),
    UNIQUE KEY `public_key_fingerprint` (`fingerprint_sha256`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TRIGGER `public_key_before_insert` 
BEFORE INSERT ON `public_key_v3` FOR EACH ROW 
BEGIN
    IF new.id IS NULL THEN
        SET new.id = GEN_UUID();
    END IF;
END;

create procedure migrate_table() 
BEGIN
    IF EXISTS (
        SELECT NULL 
        FROM information_schema.TABLES
        WHERE
            TABLE_SCHEMA = DATABASE() AND
            TABLE_NAME   = 'public_key'
    )
    THEN       
        INSERT INTO public_key_v3
            SELECT null, e.id, pk.type, pk.keydata, pk.comment, pk.keysize, 
                (IF(pk.fingerprint_md5 IS NULL,NULL,unhex(replace(pk.fingerprint_md5, ':', '')))),
                (IF(pk.fingerprint_sha256 IS NULL,NULL,from_base64(CONCAT(pk.fingerprint_sha256, "=")))),
                pk.randomart_md5, pk.randomart_sha256, pk.upload_date, pk.active
            FROM public_key pk
            INNER JOIN entity e ON pk.entity_id=e.migration_id;

        DROP TABLE `public_key`;
    END IF;
END;
CALL migrate_table();
DROP PROCEDURE migrate_table;

RENAME TABLE `public_key_v3` TO `public_key`;

-- remove deprecated tables
DROP TABLE IF EXISTS `access_request`;
DROP TABLE IF EXISTS `entity_admin`;
DROP TABLE IF EXISTS `migration`;
DROP TABLE IF EXISTS `public_key_dest_rule`;
DROP TABLE IF EXISTS `public_key_signature`;
DROP TABLE IF EXISTS `server_ldap_access_option`;
DROP TABLE IF EXISTS `user_alert`;

-- remove deleted
DELETE from `users` where active=0;
ALTER TABLE `users`
    DROP `active`;

DELETE from `groups` where active=0;
ALTER TABLE `groups`
    DROP `active`;

DELETE from `server` where deleted=1;
ALTER TABLE `server`
    DROP `deleted`;

DELETE from `server_account` where active=0;
ALTER TABLE `server_account`
    DROP `active`;

-- remove migration columns
ALTER TABLE `entity`
    DROP `migration_id`;
ALTER TABLE `server`
    DROP `migration_id`;
ALTER TABLE `access`
    DROP `migration_id`;
