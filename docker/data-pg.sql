--
-- PostgreSQL database dump
--

-- Dumped from database version 12.2
-- Dumped by pg_dump version 12.2

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: ska; Type: SCHEMA; Schema: -; Owner: ska
--

CREATE SCHEMA ska;


ALTER SCHEMA ska OWNER TO ska;

--
-- Name: access_option_option; Type: TYPE; Schema: ska; Owner: ska
--

CREATE TYPE ska.access_option_option AS ENUM (
    'command',
    'from',
    'environment',
    'no-agent-forwarding',
    'no-port-forwarding',
    'no-pty',
    'no-X11-forwarding',
    'no-user-rc'
);


ALTER TYPE ska.access_option_option OWNER TO ska;

--
-- Name: entity_type; Type: TYPE; Schema: ska; Owner: ska
--

CREATE TYPE ska.entity_type AS ENUM (
    'user',
    'server account',
    'group'
);


ALTER TYPE ska.entity_type OWNER TO ska;

--
-- Name: server_account_sync_status; Type: TYPE; Schema: ska; Owner: ska
--

CREATE TYPE ska.server_account_sync_status AS ENUM (
    'not synced yet',
    'sync success',
    'sync failure',
    'sync warning',
    'proposed'
);


ALTER TYPE ska.server_account_sync_status OWNER TO ska;

--
-- Name: server_authorization; Type: TYPE; Schema: ska; Owner: ska
--

CREATE TYPE ska.server_authorization AS ENUM (
    'manual',
    'automatic LDAP',
    'manual LDAP'
);


ALTER TYPE ska.server_authorization OWNER TO ska;

--
-- Name: server_configuration_system; Type: TYPE; Schema: ska; Owner: ska
--

CREATE TYPE ska.server_configuration_system AS ENUM (
    'unknown',
    'cf-sysadmin',
    'puppet-devops',
    'puppet-miniops',
    'puppet-tvstore',
    'none'
);


ALTER TYPE ska.server_configuration_system OWNER TO ska;

--
-- Name: server_custom_keys; Type: TYPE; Schema: ska; Owner: ska
--

CREATE TYPE ska.server_custom_keys AS ENUM (
    'not allowed',
    'allowed'
);


ALTER TYPE ska.server_custom_keys OWNER TO ska;

--
-- Name: server_key_management; Type: TYPE; Schema: ska; Owner: ska
--

CREATE TYPE ska.server_key_management AS ENUM (
    'none',
    'keys',
    'other',
    'decommissioned'
);


ALTER TYPE ska.server_key_management OWNER TO ska;

--
-- Name: server_ldap_access_option_option; Type: TYPE; Schema: ska; Owner: ska
--

CREATE TYPE ska.server_ldap_access_option_option AS ENUM (
    'command',
    'from',
    'environment',
    'no-agent-forwarding',
    'no-port-forwarding',
    'no-pty',
    'no-X11-forwarding',
    'no-user-rc'
);


ALTER TYPE ska.server_ldap_access_option_option OWNER TO ska;

--
-- Name: server_sync_status; Type: TYPE; Schema: ska; Owner: ska
--

CREATE TYPE ska.server_sync_status AS ENUM (
    'not synced yet',
    'sync success',
    'sync failure',
    'sync warning'
);


ALTER TYPE ska.server_sync_status OWNER TO ska;

--
-- Name: server_use_sync_client; Type: TYPE; Schema: ska; Owner: ska
--

CREATE TYPE ska.server_use_sync_client AS ENUM (
    'no',
    'yes'
);


ALTER TYPE ska.server_use_sync_client OWNER TO ska;

--
-- Name: user_auth_realm; Type: TYPE; Schema: ska; Owner: ska
--

CREATE TYPE ska.user_auth_realm AS ENUM (
    'LDAP',
    'local',
    'external'
);


ALTER TYPE ska.user_auth_realm OWNER TO ska;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: access; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska.access (
    id bigint NOT NULL,
    source_entity_id bigint NOT NULL,
    dest_entity_id bigint NOT NULL,
    grant_date timestamp with time zone NOT NULL,
    granted_by bigint NOT NULL
);


ALTER TABLE ska.access OWNER TO ska;

--
-- Name: access_id_seq; Type: SEQUENCE; Schema: ska; Owner: ska
--

CREATE SEQUENCE ska.access_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE ska.access_id_seq OWNER TO ska;

--
-- Name: access_id_seq; Type: SEQUENCE OWNED BY; Schema: ska; Owner: ska
--

ALTER SEQUENCE ska.access_id_seq OWNED BY ska.access.id;


--
-- Name: access_option; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska.access_option (
    id bigint NOT NULL,
    access_id bigint NOT NULL,
    option ska.access_option_option NOT NULL,
    value text
);


ALTER TABLE ska.access_option OWNER TO ska;

--
-- Name: access_option_id_seq; Type: SEQUENCE; Schema: ska; Owner: ska
--

CREATE SEQUENCE ska.access_option_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE ska.access_option_id_seq OWNER TO ska;

--
-- Name: access_option_id_seq; Type: SEQUENCE OWNED BY; Schema: ska; Owner: ska
--

ALTER SEQUENCE ska.access_option_id_seq OWNED BY ska.access_option.id;


--
-- Name: access_request; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska.access_request (
    id bigint NOT NULL,
    source_entity_id bigint NOT NULL,
    dest_entity_id bigint NOT NULL,
    request_date timestamp with time zone NOT NULL,
    requested_by bigint NOT NULL
);


ALTER TABLE ska.access_request OWNER TO ska;

--
-- Name: access_request_id_seq; Type: SEQUENCE; Schema: ska; Owner: ska
--

CREATE SEQUENCE ska.access_request_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE ska.access_request_id_seq OWNER TO ska;

--
-- Name: access_request_id_seq; Type: SEQUENCE OWNED BY; Schema: ska; Owner: ska
--

ALTER SEQUENCE ska.access_request_id_seq OWNED BY ska.access_request.id;


--
-- Name: entity; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska.entity (
    id bigint NOT NULL,
    type ska.entity_type NOT NULL
);


ALTER TABLE ska.entity OWNER TO ska;

--
-- Name: entity_admin; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska.entity_admin (
    entity_id bigint NOT NULL,
    admin bigint NOT NULL
);


ALTER TABLE ska.entity_admin OWNER TO ska;

--
-- Name: entity_event; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska.entity_event (
    id bigint NOT NULL,
    entity_id bigint NOT NULL,
    actor_id bigint,
    date timestamp with time zone NOT NULL,
    details text NOT NULL
);


ALTER TABLE ska.entity_event OWNER TO ska;

--
-- Name: entity_event_id_seq; Type: SEQUENCE; Schema: ska; Owner: ska
--

CREATE SEQUENCE ska.entity_event_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE ska.entity_event_id_seq OWNER TO ska;

--
-- Name: entity_event_id_seq; Type: SEQUENCE OWNED BY; Schema: ska; Owner: ska
--

ALTER SEQUENCE ska.entity_event_id_seq OWNED BY ska.entity_event.id;


--
-- Name: entity_id_seq; Type: SEQUENCE; Schema: ska; Owner: ska
--

CREATE SEQUENCE ska.entity_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE ska.entity_id_seq OWNER TO ska;

--
-- Name: entity_id_seq; Type: SEQUENCE OWNED BY; Schema: ska; Owner: ska
--

ALTER SEQUENCE ska.entity_id_seq OWNED BY ska.entity.id;


--
-- Name: group; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska."group" (
    entity_id bigint NOT NULL,
    name character varying(100) NOT NULL,
    active boolean DEFAULT true NOT NULL,
    system boolean DEFAULT false NOT NULL
);


ALTER TABLE ska."group" OWNER TO ska;

--
-- Name: group_event; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska.group_event (
    id bigint NOT NULL,
    "group" bigint NOT NULL,
    entity_id bigint,
    date timestamp with time zone NOT NULL,
    details text NOT NULL
);


ALTER TABLE ska.group_event OWNER TO ska;

--
-- Name: group_event_id_seq; Type: SEQUENCE; Schema: ska; Owner: ska
--

CREATE SEQUENCE ska.group_event_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE ska.group_event_id_seq OWNER TO ska;

--
-- Name: group_event_id_seq; Type: SEQUENCE OWNED BY; Schema: ska; Owner: ska
--

ALTER SEQUENCE ska.group_event_id_seq OWNED BY ska.group_event.id;


--
-- Name: group_member; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska.group_member (
    id bigint NOT NULL,
    "group" bigint NOT NULL,
    entity_id bigint NOT NULL,
    add_date timestamp with time zone NOT NULL,
    added_by bigint
);


ALTER TABLE ska.group_member OWNER TO ska;

--
-- Name: group_member_id_seq; Type: SEQUENCE; Schema: ska; Owner: ska
--

CREATE SEQUENCE ska.group_member_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE ska.group_member_id_seq OWNER TO ska;

--
-- Name: group_member_id_seq; Type: SEQUENCE OWNED BY; Schema: ska; Owner: ska
--

ALTER SEQUENCE ska.group_member_id_seq OWNED BY ska.group_member.id;


--
-- Name: migration; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska.migration (
    id bigint NOT NULL,
    name text NOT NULL,
    applied timestamp with time zone NOT NULL
);


ALTER TABLE ska.migration OWNER TO ska;

--
-- Name: public_key; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska.public_key (
    id bigint NOT NULL,
    entity_id bigint NOT NULL,
    type character varying(30) NOT NULL,
    keydata text NOT NULL,
    comment text NOT NULL,
    keysize bigint,
    fingerprint_md5 character(47) DEFAULT NULL::bpchar,
    fingerprint_sha256 character varying(50) DEFAULT NULL::character varying,
    randomart_md5 text,
    randomart_sha256 text,
    upload_date timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    active boolean DEFAULT true NOT NULL
);


ALTER TABLE ska.public_key OWNER TO ska;

--
-- Name: public_key_dest_rule; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska.public_key_dest_rule (
    id bigint NOT NULL,
    public_key_id bigint NOT NULL,
    account_name_filter character varying(50) NOT NULL,
    hostname_filter character varying(255) NOT NULL
);


ALTER TABLE ska.public_key_dest_rule OWNER TO ska;

--
-- Name: public_key_dest_rule_id_seq; Type: SEQUENCE; Schema: ska; Owner: ska
--

CREATE SEQUENCE ska.public_key_dest_rule_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE ska.public_key_dest_rule_id_seq OWNER TO ska;

--
-- Name: public_key_dest_rule_id_seq; Type: SEQUENCE OWNED BY; Schema: ska; Owner: ska
--

ALTER SEQUENCE ska.public_key_dest_rule_id_seq OWNED BY ska.public_key_dest_rule.id;


--
-- Name: public_key_id_seq; Type: SEQUENCE; Schema: ska; Owner: ska
--

CREATE SEQUENCE ska.public_key_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE ska.public_key_id_seq OWNER TO ska;

--
-- Name: public_key_id_seq; Type: SEQUENCE OWNED BY; Schema: ska; Owner: ska
--

ALTER SEQUENCE ska.public_key_id_seq OWNED BY ska.public_key.id;


--
-- Name: public_key_signature; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska.public_key_signature (
    id bigint NOT NULL,
    public_key_id bigint NOT NULL,
    signature bytea NOT NULL,
    upload_date timestamp with time zone NOT NULL,
    fingerprint character varying(50) NOT NULL,
    sign_date timestamp with time zone NOT NULL
);


ALTER TABLE ska.public_key_signature OWNER TO ska;

--
-- Name: public_key_signature_id_seq; Type: SEQUENCE; Schema: ska; Owner: ska
--

CREATE SEQUENCE ska.public_key_signature_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE ska.public_key_signature_id_seq OWNER TO ska;

--
-- Name: public_key_signature_id_seq; Type: SEQUENCE OWNED BY; Schema: ska; Owner: ska
--

ALTER SEQUENCE ska.public_key_signature_id_seq OWNED BY ska.public_key_signature.id;


--
-- Name: server; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska.server (
    id bigint NOT NULL,
    uuid character varying(36) DEFAULT NULL::character varying,
    hostname character varying(150) NOT NULL,
    ip_address character varying(64) DEFAULT NULL::character varying,
    deleted boolean DEFAULT false NOT NULL,
    key_management ska.server_key_management DEFAULT 'keys'::ska.server_key_management NOT NULL,
    "authorization" ska.server_authorization DEFAULT 'manual'::ska.server_authorization NOT NULL,
    use_sync_client ska.server_use_sync_client DEFAULT 'no'::ska.server_use_sync_client NOT NULL,
    sync_status ska.server_sync_status DEFAULT 'not synced yet'::ska.server_sync_status NOT NULL,
    configuration_system ska.server_configuration_system DEFAULT 'unknown'::ska.server_configuration_system NOT NULL,
    custom_keys ska.server_custom_keys DEFAULT 'not allowed'::ska.server_custom_keys NOT NULL,
    rsa_key_fingerprint character(32) DEFAULT NULL::bpchar,
    port bigint DEFAULT '22'::bigint NOT NULL
);


ALTER TABLE ska.server OWNER TO ska;

--
-- Name: server_account; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska.server_account (
    entity_id bigint NOT NULL,
    server_id bigint NOT NULL,
    name character varying(50) DEFAULT NULL::character varying,
    sync_status ska.server_account_sync_status DEFAULT 'not synced yet'::ska.server_account_sync_status NOT NULL,
    active boolean DEFAULT true NOT NULL
);


ALTER TABLE ska.server_account OWNER TO ska;

--
-- Name: server_admin; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska.server_admin (
    server_id bigint NOT NULL,
    entity_id bigint NOT NULL
);


ALTER TABLE ska.server_admin OWNER TO ska;

--
-- Name: server_event; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska.server_event (
    id bigint NOT NULL,
    server_id bigint NOT NULL,
    actor_id bigint,
    date timestamp with time zone NOT NULL,
    details text NOT NULL
);


ALTER TABLE ska.server_event OWNER TO ska;

--
-- Name: server_event_id_seq; Type: SEQUENCE; Schema: ska; Owner: ska
--

CREATE SEQUENCE ska.server_event_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE ska.server_event_id_seq OWNER TO ska;

--
-- Name: server_event_id_seq; Type: SEQUENCE OWNED BY; Schema: ska; Owner: ska
--

ALTER SEQUENCE ska.server_event_id_seq OWNED BY ska.server_event.id;


--
-- Name: server_id_seq; Type: SEQUENCE; Schema: ska; Owner: ska
--

CREATE SEQUENCE ska.server_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE ska.server_id_seq OWNER TO ska;

--
-- Name: server_id_seq; Type: SEQUENCE OWNED BY; Schema: ska; Owner: ska
--

ALTER SEQUENCE ska.server_id_seq OWNED BY ska.server.id;


--
-- Name: server_ldap_access_option; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska.server_ldap_access_option (
    id bigint NOT NULL,
    server_id bigint NOT NULL,
    option ska.server_ldap_access_option_option NOT NULL,
    value text
);


ALTER TABLE ska.server_ldap_access_option OWNER TO ska;

--
-- Name: server_ldap_access_option_id_seq; Type: SEQUENCE; Schema: ska; Owner: ska
--

CREATE SEQUENCE ska.server_ldap_access_option_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE ska.server_ldap_access_option_id_seq OWNER TO ska;

--
-- Name: server_ldap_access_option_id_seq; Type: SEQUENCE OWNED BY; Schema: ska; Owner: ska
--

ALTER SEQUENCE ska.server_ldap_access_option_id_seq OWNED BY ska.server_ldap_access_option.id;


--
-- Name: server_note; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska.server_note (
    id bigint NOT NULL,
    server_id bigint NOT NULL,
    entity_id bigint,
    date timestamp with time zone NOT NULL,
    note text NOT NULL
);


ALTER TABLE ska.server_note OWNER TO ska;

--
-- Name: server_note_id_seq; Type: SEQUENCE; Schema: ska; Owner: ska
--

CREATE SEQUENCE ska.server_note_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE ska.server_note_id_seq OWNER TO ska;

--
-- Name: server_note_id_seq; Type: SEQUENCE OWNED BY; Schema: ska; Owner: ska
--

ALTER SEQUENCE ska.server_note_id_seq OWNED BY ska.server_note.id;


--
-- Name: sync_request; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska.sync_request (
    id bigint NOT NULL,
    server_id bigint NOT NULL,
    account_name character varying(50) DEFAULT NULL::character varying,
    processing boolean DEFAULT false NOT NULL
);


ALTER TABLE ska.sync_request OWNER TO ska;

--
-- Name: sync_request_id_seq; Type: SEQUENCE; Schema: ska; Owner: ska
--

CREATE SEQUENCE ska.sync_request_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE ska.sync_request_id_seq OWNER TO ska;

--
-- Name: sync_request_id_seq; Type: SEQUENCE OWNED BY; Schema: ska; Owner: ska
--

ALTER SEQUENCE ska.sync_request_id_seq OWNED BY ska.sync_request.id;


--
-- Name: user; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska."user" (
    entity_id bigint NOT NULL,
    uid character varying(50) NOT NULL,
    name character varying(100) NOT NULL,
    email character varying(100) NOT NULL,
    superior_entity_id bigint,
    auth_realm ska.user_auth_realm DEFAULT 'LDAP'::ska.user_auth_realm NOT NULL,
    active boolean DEFAULT true NOT NULL,
    admin boolean DEFAULT false NOT NULL,
    developer boolean DEFAULT false NOT NULL,
    force_disable boolean DEFAULT false NOT NULL,
    csrf_token bytea DEFAULT '\x4e554c4c'::bytea
);


ALTER TABLE ska."user" OWNER TO ska;

--
-- Name: user_alert; Type: TABLE; Schema: ska; Owner: ska
--

CREATE TABLE ska.user_alert (
    id bigint NOT NULL,
    entity_id bigint NOT NULL,
    class character varying(15) NOT NULL,
    content text NOT NULL,
    escaping bigint DEFAULT '1'::bigint NOT NULL
);


ALTER TABLE ska.user_alert OWNER TO ska;

--
-- Name: user_alert_id_seq; Type: SEQUENCE; Schema: ska; Owner: ska
--

CREATE SEQUENCE ska.user_alert_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE ska.user_alert_id_seq OWNER TO ska;

--
-- Name: user_alert_id_seq; Type: SEQUENCE OWNED BY; Schema: ska; Owner: ska
--

ALTER SEQUENCE ska.user_alert_id_seq OWNED BY ska.user_alert.id;


--
-- Name: access id; Type: DEFAULT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.access ALTER COLUMN id SET DEFAULT nextval('ska.access_id_seq'::regclass);


--
-- Name: access_option id; Type: DEFAULT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.access_option ALTER COLUMN id SET DEFAULT nextval('ska.access_option_id_seq'::regclass);


--
-- Name: access_request id; Type: DEFAULT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.access_request ALTER COLUMN id SET DEFAULT nextval('ska.access_request_id_seq'::regclass);


--
-- Name: entity id; Type: DEFAULT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.entity ALTER COLUMN id SET DEFAULT nextval('ska.entity_id_seq'::regclass);


--
-- Name: entity_event id; Type: DEFAULT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.entity_event ALTER COLUMN id SET DEFAULT nextval('ska.entity_event_id_seq'::regclass);


--
-- Name: group_event id; Type: DEFAULT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.group_event ALTER COLUMN id SET DEFAULT nextval('ska.group_event_id_seq'::regclass);


--
-- Name: group_member id; Type: DEFAULT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.group_member ALTER COLUMN id SET DEFAULT nextval('ska.group_member_id_seq'::regclass);


--
-- Name: public_key id; Type: DEFAULT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.public_key ALTER COLUMN id SET DEFAULT nextval('ska.public_key_id_seq'::regclass);


--
-- Name: public_key_dest_rule id; Type: DEFAULT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.public_key_dest_rule ALTER COLUMN id SET DEFAULT nextval('ska.public_key_dest_rule_id_seq'::regclass);


--
-- Name: public_key_signature id; Type: DEFAULT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.public_key_signature ALTER COLUMN id SET DEFAULT nextval('ska.public_key_signature_id_seq'::regclass);


--
-- Name: server id; Type: DEFAULT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.server ALTER COLUMN id SET DEFAULT nextval('ska.server_id_seq'::regclass);


--
-- Name: server_event id; Type: DEFAULT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.server_event ALTER COLUMN id SET DEFAULT nextval('ska.server_event_id_seq'::regclass);


--
-- Name: server_ldap_access_option id; Type: DEFAULT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.server_ldap_access_option ALTER COLUMN id SET DEFAULT nextval('ska.server_ldap_access_option_id_seq'::regclass);


--
-- Name: server_note id; Type: DEFAULT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.server_note ALTER COLUMN id SET DEFAULT nextval('ska.server_note_id_seq'::regclass);


--
-- Name: sync_request id; Type: DEFAULT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.sync_request ALTER COLUMN id SET DEFAULT nextval('ska.sync_request_id_seq'::regclass);


--
-- Name: user_alert id; Type: DEFAULT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.user_alert ALTER COLUMN id SET DEFAULT nextval('ska.user_alert_id_seq'::regclass);


--
-- Data for Name: access; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska.access (id, source_entity_id, dest_entity_id, grant_date, granted_by) FROM stdin;
1	4	6	2019-04-22 13:22:40+00	2
\.


--
-- Data for Name: access_option; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska.access_option (id, access_id, option, value) FROM stdin;
\.


--
-- Data for Name: access_request; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska.access_request (id, source_entity_id, dest_entity_id, request_date, requested_by) FROM stdin;
\.


--
-- Data for Name: entity; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska.entity (id, type) FROM stdin;
1	user
2	user
3	user
4	group
5	server account
6	group
\.


--
-- Data for Name: entity_admin; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska.entity_admin (entity_id, admin) FROM stdin;
4	2
\.


--
-- Data for Name: entity_event; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska.entity_event (id, entity_id, actor_id, date, details) FROM stdin;
1	2	1	2019-04-22 13:20:58+00	{"action":"User add"}
2	3	1	2019-04-22 13:21:04+00	{"action":"User add"}
3	4	1	2019-04-22 13:21:13+00	{"action":"Group add"}
4	4	1	2019-04-22 13:21:13+00	{"action":"Administrator add","value":"user:rainbow"}
5	2	2	2019-04-22 13:21:57+00	{"action":"Pubkey add","value":"6e:ef:f4:2d:1a:60:b5:fa:13:92:bc:93:fd:98:e1:00"}
6	6	2	2019-04-22 13:22:14+00	{"action":"Group add"}
7	6	2	2019-04-22 13:22:14+00	{"action":"Member add","value":"account:root@test.example.com"}
8	5	1	2019-04-22 13:22:16+00	{"action":"Setting update","value":"sync success","oldvalue":"not synced yet","field":"Sync status"}
9	6	2	2019-04-22 13:22:40+00	{"action":"Access add","value":"group:admin"}
10	4	2	2019-04-22 13:23:25+00	{"action":"Member add","value":"user:rainbow"}
\.


--
-- Data for Name: group; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska."group" (entity_id, name, active, system) FROM stdin;
4	admin	t	f
6	accounts-root	t	t
\.


--
-- Data for Name: group_event; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska.group_event (id, "group", entity_id, date, details) FROM stdin;
\.


--
-- Data for Name: group_member; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska.group_member (id, "group", entity_id, add_date, added_by) FROM stdin;
1	6	5	2019-04-22 13:22:14+00	2
2	4	2	2019-04-22 13:23:25+00	2
\.


--
-- Data for Name: migration; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska.migration (id, name, applied) FROM stdin;
1	Add migration support	2019-04-22 13:19:58+00
2	Initial setup, converted to migration	2019-04-22 13:20:01+00
3	Add port number field	2019-04-22 13:20:01+00
4	Add local usermanagment	2019-04-22 13:20:02+00
5	Add key deprication to public key	2019-04-22 13:20:02+00
6	Add additional ssh key access options	2019-04-22 13:20:02+00
\.


--
-- Data for Name: public_key; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska.public_key (id, entity_id, type, keydata, comment, keysize, fingerprint_md5, fingerprint_sha256, randomart_md5, randomart_sha256, upload_date, active) FROM stdin;
1	2	ssh-rsa	AAAAB3NzaC1yc2EAAAADAQABAAACAQC3/aRJAhxgH8zkw5yj/U8wOViPmn+yplS+I5laZ59357tFe+jcvGazr9EsSb4HeTkcC1ykSvFexqAz6D9i6bUug6Tqhrk+VoJsLQm6zCt8bSdMBPwHvUUzToVWVhdYD2MPPo6BnaWHCJhkh/FgQ3ymEvt06Vr81knN7spXhKe7/lc2MjDaVgZQ8ubRcpKO+lMVyU12Q09lHIgEJzInCLQSGwxVsVlVzHpueaNrkOom0+3h7GuLU98NoebC1Q6vtW+YhskT5nEmjDEt6F3KIw+hrb4VBcWbVkkXpmr2huzUlTBZqw5y8IGHGml02iOTr3udi12ru9sWkJF7D7p21z272mT05rJBzQcSbEwwoBr9tNyVNDdrCfLX/yZESPeX8JcOCOOefSzbFGWXkhs4EO4q6fT8xtVS2mDz/fm0Nd9h5wipMQOE2YARx8pEqNjbvY0NGDbUnqD5qj/fEdErp0DzhLpKyuX5HHil14dxBpjsxlo1CKGtr1j3QWsKlFM7snpdRoWsPqpTjqwRIUW/znjEaAkbuZs35gIqOr42clzfe9C20CjyvZZ1RvedcSCmzZsiyJzzWdOQ+KYJwsQXrHI1D1WII2oDa/DOI4RFZbPPpmQdGR1MgNPuJNo6+DE0GVZJb7R3f7xKDEOv+ScRnVfYiMpokeSG1nRUgCPpJy10Bw==	test@ska-demo.itmettke.de	4096	6e:ef:f4:2d:1a:60:b5:fa:13:92:bc:93:fd:98:e1:00	dhS/m1bOqLMF3Icg8qeS7Xz2b4XwshhDY+/1OpP9X1A	+---[RSA 4096]----+\n|                 |\n|                 |\n|          .      |\n|         . .     |\n|      E.S..      |\n|       ++o.      |\n|        ==+.     |\n|       .+*o*..   |\n|         oO+o..  |\n+------[MD5]------+	+---[RSA 4096]----+\n|          .      |\n|           o     |\n|      . . o .   E|\n|       o ++o.o . |\n|        So=o+o+. |\n|       + +o.oO+..|\n|      o o  =*+o=.|\n|       +  =+o = +|\n|        ooo+.oo+=|\n+----[SHA256]-----+	2019-04-22 13:21:57+00	t
\.


--
-- Data for Name: public_key_dest_rule; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska.public_key_dest_rule (id, public_key_id, account_name_filter, hostname_filter) FROM stdin;
\.


--
-- Data for Name: public_key_signature; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska.public_key_signature (id, public_key_id, signature, upload_date, fingerprint, sign_date) FROM stdin;
\.


--
-- Data for Name: server; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska.server (id, uuid, hostname, ip_address, deleted, key_management, "authorization", use_sync_client, sync_status, configuration_system, custom_keys, rsa_key_fingerprint, port) FROM stdin;
1	\N	test.example.com	\N	f	keys	manual	no	sync success	unknown	not allowed	\N	22
\.


--
-- Data for Name: server_account; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska.server_account (entity_id, server_id, name, sync_status, active) FROM stdin;
5	1	root	sync success	t
\.


--
-- Data for Name: server_admin; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska.server_admin (server_id, entity_id) FROM stdin;
1	4
\.


--
-- Data for Name: server_event; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska.server_event (id, server_id, actor_id, date, details) FROM stdin;
1	1	2	2019-04-22 13:22:14+00	{"action":"Server add"}
2	1	2	2019-04-22 13:22:14+00	{"action":"Account add","value":"root"}
3	1	2	2019-04-22 13:22:14+00	{"action":"Administrator add","value":"group:admin"}
4	1	1	2019-04-22 13:22:15+00	{"action":"Setting update","value":"172.29.0.2","oldvalue":null,"field":"Ip address"}
5	1	1	2019-04-22 13:22:15+00	{"action":"Setting update","value":"F85CFE3722AFBB4622E6D2738659B04C","oldvalue":null,"field":"Rsa key fingerprint"}
6	1	1	2019-04-22 13:22:16+00	{"action":"Sync status change","value":"Synced successfully"}
7	1	1	2019-04-22 13:22:16+00	{"action":"Setting update","value":"sync success","oldvalue":"not synced yet","field":"Sync status"}
\.


--
-- Data for Name: server_ldap_access_option; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska.server_ldap_access_option (id, server_id, option, value) FROM stdin;
\.


--
-- Data for Name: server_note; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska.server_note (id, server_id, entity_id, date, note) FROM stdin;
\.


--
-- Data for Name: sync_request; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska.sync_request (id, server_id, account_name, processing) FROM stdin;
\.


--
-- Data for Name: user; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska."user" (entity_id, uid, name, email, superior_entity_id, auth_realm, active, admin, developer, force_disable, csrf_token) FROM stdin;
1	keys-sync	Synchronization script		\N	local	t	t	f	f	\\x3036326238343736383335363233353736326161326664373634346635333763323633373635373664653664643239383162313438313037373831646638326538633032323637643132343639323036613964383165616561646532633065326633336633326165383437653432373331303663613836393238656633343431
2	rainbow	Rain Bow	rainbow@localhost	\N	local	t	t	f	f	\\x3231653134353832303661653737333637613363626439336364356637373264363864356462343631396239666666653732626135343234663131623139363462333862363732656334373362656530313962643464323337616165333265323031616665306236653033363437636238656433666238383063373934316132
3	proceme	Proce Me	proceme@localhost	\N	local	t	f	f	f	\N
\.


--
-- Data for Name: user_alert; Type: TABLE DATA; Schema: ska; Owner: ska
--

COPY ska.user_alert (id, entity_id, class, content, escaping) FROM stdin;
\.


--
-- Name: access_id_seq; Type: SEQUENCE SET; Schema: ska; Owner: ska
--

SELECT pg_catalog.setval('ska.access_id_seq', 1, true);


--
-- Name: access_option_id_seq; Type: SEQUENCE SET; Schema: ska; Owner: ska
--

SELECT pg_catalog.setval('ska.access_option_id_seq', 1, true);


--
-- Name: access_request_id_seq; Type: SEQUENCE SET; Schema: ska; Owner: ska
--

SELECT pg_catalog.setval('ska.access_request_id_seq', 1, true);


--
-- Name: entity_event_id_seq; Type: SEQUENCE SET; Schema: ska; Owner: ska
--

SELECT pg_catalog.setval('ska.entity_event_id_seq', 10, true);


--
-- Name: entity_id_seq; Type: SEQUENCE SET; Schema: ska; Owner: ska
--

SELECT pg_catalog.setval('ska.entity_id_seq', 6, true);


--
-- Name: group_event_id_seq; Type: SEQUENCE SET; Schema: ska; Owner: ska
--

SELECT pg_catalog.setval('ska.group_event_id_seq', 1, true);


--
-- Name: group_member_id_seq; Type: SEQUENCE SET; Schema: ska; Owner: ska
--

SELECT pg_catalog.setval('ska.group_member_id_seq', 2, true);


--
-- Name: public_key_dest_rule_id_seq; Type: SEQUENCE SET; Schema: ska; Owner: ska
--

SELECT pg_catalog.setval('ska.public_key_dest_rule_id_seq', 1, true);


--
-- Name: public_key_id_seq; Type: SEQUENCE SET; Schema: ska; Owner: ska
--

SELECT pg_catalog.setval('ska.public_key_id_seq', 1, true);


--
-- Name: public_key_signature_id_seq; Type: SEQUENCE SET; Schema: ska; Owner: ska
--

SELECT pg_catalog.setval('ska.public_key_signature_id_seq', 1, true);


--
-- Name: server_event_id_seq; Type: SEQUENCE SET; Schema: ska; Owner: ska
--

SELECT pg_catalog.setval('ska.server_event_id_seq', 7, true);


--
-- Name: server_id_seq; Type: SEQUENCE SET; Schema: ska; Owner: ska
--

SELECT pg_catalog.setval('ska.server_id_seq', 1, true);


--
-- Name: server_ldap_access_option_id_seq; Type: SEQUENCE SET; Schema: ska; Owner: ska
--

SELECT pg_catalog.setval('ska.server_ldap_access_option_id_seq', 1, true);


--
-- Name: server_note_id_seq; Type: SEQUENCE SET; Schema: ska; Owner: ska
--

SELECT pg_catalog.setval('ska.server_note_id_seq', 1, true);


--
-- Name: sync_request_id_seq; Type: SEQUENCE SET; Schema: ska; Owner: ska
--

SELECT pg_catalog.setval('ska.sync_request_id_seq', 1, true);


--
-- Name: user_alert_id_seq; Type: SEQUENCE SET; Schema: ska; Owner: ska
--

SELECT pg_catalog.setval('ska.user_alert_id_seq', 1, true);


--
-- Name: access idx_24697_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.access
    ADD CONSTRAINT idx_24697_primary PRIMARY KEY (id);


--
-- Name: access_option idx_24703_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.access_option
    ADD CONSTRAINT idx_24703_primary PRIMARY KEY (id);


--
-- Name: access_request idx_24712_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.access_request
    ADD CONSTRAINT idx_24712_primary PRIMARY KEY (id);


--
-- Name: entity idx_24718_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.entity
    ADD CONSTRAINT idx_24718_primary PRIMARY KEY (id);


--
-- Name: entity_admin idx_24722_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.entity_admin
    ADD CONSTRAINT idx_24722_primary PRIMARY KEY (entity_id, admin);


--
-- Name: entity_event idx_24727_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.entity_event
    ADD CONSTRAINT idx_24727_primary PRIMARY KEY (id);


--
-- Name: group idx_24734_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska."group"
    ADD CONSTRAINT idx_24734_primary PRIMARY KEY (entity_id);


--
-- Name: group_event idx_24741_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.group_event
    ADD CONSTRAINT idx_24741_primary PRIMARY KEY (id);


--
-- Name: group_member idx_24750_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.group_member
    ADD CONSTRAINT idx_24750_primary PRIMARY KEY (id);


--
-- Name: public_key idx_24762_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.public_key
    ADD CONSTRAINT idx_24762_primary PRIMARY KEY (id);


--
-- Name: public_key_dest_rule idx_24775_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.public_key_dest_rule
    ADD CONSTRAINT idx_24775_primary PRIMARY KEY (id);


--
-- Name: public_key_signature idx_24781_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.public_key_signature
    ADD CONSTRAINT idx_24781_primary PRIMARY KEY (id);


--
-- Name: server idx_24790_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.server
    ADD CONSTRAINT idx_24790_primary PRIMARY KEY (id);


--
-- Name: server_account idx_24805_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.server_account
    ADD CONSTRAINT idx_24805_primary PRIMARY KEY (entity_id);


--
-- Name: server_admin idx_24811_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.server_admin
    ADD CONSTRAINT idx_24811_primary PRIMARY KEY (server_id, entity_id);


--
-- Name: server_event idx_24816_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.server_event
    ADD CONSTRAINT idx_24816_primary PRIMARY KEY (id);


--
-- Name: server_ldap_access_option idx_24825_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.server_ldap_access_option
    ADD CONSTRAINT idx_24825_primary PRIMARY KEY (id);


--
-- Name: server_note idx_24834_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.server_note
    ADD CONSTRAINT idx_24834_primary PRIMARY KEY (id);


--
-- Name: sync_request idx_24843_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.sync_request
    ADD CONSTRAINT idx_24843_primary PRIMARY KEY (id);


--
-- Name: user idx_24849_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska."user"
    ADD CONSTRAINT idx_24849_primary PRIMARY KEY (entity_id);


--
-- Name: user_alert idx_24863_primary; Type: CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.user_alert
    ADD CONSTRAINT idx_24863_primary PRIMARY KEY (id);


--
-- Name: idx_24697_fk_access_entity_2; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24697_fk_access_entity_2 ON ska.access USING btree (dest_entity_id);


--
-- Name: idx_24697_fk_access_entity_3; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24697_fk_access_entity_3 ON ska.access USING btree (granted_by);


--
-- Name: idx_24697_source_entity_id_dest_entity_id; Type: INDEX; Schema: ska; Owner: ska
--

CREATE UNIQUE INDEX idx_24697_source_entity_id_dest_entity_id ON ska.access USING btree (source_entity_id, dest_entity_id);


--
-- Name: idx_24703_access_id_option; Type: INDEX; Schema: ska; Owner: ska
--

CREATE UNIQUE INDEX idx_24703_access_id_option ON ska.access_option USING btree (access_id, option);


--
-- Name: idx_24712_fk_access_request_entity_2; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24712_fk_access_request_entity_2 ON ska.access_request USING btree (dest_entity_id);


--
-- Name: idx_24712_fk_access_request_entity_3; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24712_fk_access_request_entity_3 ON ska.access_request USING btree (requested_by);


--
-- Name: idx_24712_source_entity_id_dest_entity_id; Type: INDEX; Schema: ska; Owner: ska
--

CREATE UNIQUE INDEX idx_24712_source_entity_id_dest_entity_id ON ska.access_request USING btree (source_entity_id, dest_entity_id);


--
-- Name: idx_24722_fk_entity_admin_entity_2; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24722_fk_entity_admin_entity_2 ON ska.entity_admin USING btree (admin);


--
-- Name: idx_24727_fk_entity_event_actor_id; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24727_fk_entity_event_actor_id ON ska.entity_event USING btree (actor_id);


--
-- Name: idx_24727_fk_entity_event_entity_id; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24727_fk_entity_event_entity_id ON ska.entity_event USING btree (entity_id);


--
-- Name: idx_24734_name; Type: INDEX; Schema: ska; Owner: ska
--

CREATE UNIQUE INDEX idx_24734_name ON ska."group" USING btree (name);


--
-- Name: idx_24741_fk_group_event_entity; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24741_fk_group_event_entity ON ska.group_event USING btree (entity_id);


--
-- Name: idx_24741_fk_group_event_group; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24741_fk_group_event_group ON ska.group_event USING btree ("group");


--
-- Name: idx_24750_fk_group_member_entity; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24750_fk_group_member_entity ON ska.group_member USING btree (entity_id);


--
-- Name: idx_24750_fk_group_member_entity_2; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24750_fk_group_member_entity_2 ON ska.group_member USING btree (added_by);


--
-- Name: idx_24750_group_entity_id; Type: INDEX; Schema: ska; Owner: ska
--

CREATE UNIQUE INDEX idx_24750_group_entity_id ON ska.group_member USING btree ("group", entity_id);


--
-- Name: idx_24762_fk_public_key_entity; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24762_fk_public_key_entity ON ska.public_key USING btree (entity_id);


--
-- Name: idx_24762_public_key_fingerprint; Type: INDEX; Schema: ska; Owner: ska
--

CREATE UNIQUE INDEX idx_24762_public_key_fingerprint ON ska.public_key USING btree (fingerprint_sha256);


--
-- Name: idx_24775_fk_public_key_dest_rule_public_key; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24775_fk_public_key_dest_rule_public_key ON ska.public_key_dest_rule USING btree (public_key_id);


--
-- Name: idx_24781_fk_public_key_signature_public_key; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24781_fk_public_key_signature_public_key ON ska.public_key_signature USING btree (public_key_id);


--
-- Name: idx_24790_hostname; Type: INDEX; Schema: ska; Owner: ska
--

CREATE UNIQUE INDEX idx_24790_hostname ON ska.server USING btree (hostname);


--
-- Name: idx_24790_ip_address; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24790_ip_address ON ska.server USING btree (ip_address);


--
-- Name: idx_24805_fk_server_account_server; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24805_fk_server_account_server ON ska.server_account USING btree (server_id);


--
-- Name: idx_24805_server_id_name; Type: INDEX; Schema: ska; Owner: ska
--

CREATE UNIQUE INDEX idx_24805_server_id_name ON ska.server_account USING btree (server_id, name);


--
-- Name: idx_24811_fk_server_admin_entity; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24811_fk_server_admin_entity ON ska.server_admin USING btree (entity_id);


--
-- Name: idx_24816_fk_server_event_actor_id; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24816_fk_server_event_actor_id ON ska.server_event USING btree (actor_id);


--
-- Name: idx_24816_fk_server_log_server; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24816_fk_server_log_server ON ska.server_event USING btree (server_id);


--
-- Name: idx_24825_server_id_option; Type: INDEX; Schema: ska; Owner: ska
--

CREATE UNIQUE INDEX idx_24825_server_id_option ON ska.server_ldap_access_option USING btree (server_id, option);


--
-- Name: idx_24834_fk_server_note_server; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24834_fk_server_note_server ON ska.server_note USING btree (server_id);


--
-- Name: idx_24834_fk_server_note_user; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24834_fk_server_note_user ON ska.server_note USING btree (entity_id);


--
-- Name: idx_24843_server_id_account_name; Type: INDEX; Schema: ska; Owner: ska
--

CREATE UNIQUE INDEX idx_24843_server_id_account_name ON ska.sync_request USING btree (server_id, account_name);


--
-- Name: idx_24849_uid; Type: INDEX; Schema: ska; Owner: ska
--

CREATE UNIQUE INDEX idx_24849_uid ON ska."user" USING btree (uid);


--
-- Name: idx_24863_fk_user_alert_entity; Type: INDEX; Schema: ska; Owner: ska
--

CREATE INDEX idx_24863_fk_user_alert_entity ON ska.user_alert USING btree (entity_id);


--
-- Name: access fk_access_entity; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.access
    ADD CONSTRAINT fk_access_entity FOREIGN KEY (source_entity_id) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: access fk_access_entity_2; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.access
    ADD CONSTRAINT fk_access_entity_2 FOREIGN KEY (dest_entity_id) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: access fk_access_entity_3; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.access
    ADD CONSTRAINT fk_access_entity_3 FOREIGN KEY (granted_by) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: access_option fk_access_option_access; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.access_option
    ADD CONSTRAINT fk_access_option_access FOREIGN KEY (access_id) REFERENCES ska.access(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: access_request fk_access_request_entity; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.access_request
    ADD CONSTRAINT fk_access_request_entity FOREIGN KEY (source_entity_id) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: access_request fk_access_request_entity_2; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.access_request
    ADD CONSTRAINT fk_access_request_entity_2 FOREIGN KEY (dest_entity_id) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: access_request fk_access_request_entity_3; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.access_request
    ADD CONSTRAINT fk_access_request_entity_3 FOREIGN KEY (requested_by) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: entity_admin fk_entity_admin_entity; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.entity_admin
    ADD CONSTRAINT fk_entity_admin_entity FOREIGN KEY (entity_id) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: entity_admin fk_entity_admin_entity_2; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.entity_admin
    ADD CONSTRAINT fk_entity_admin_entity_2 FOREIGN KEY (admin) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: entity_event fk_entity_event_actor_id; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.entity_event
    ADD CONSTRAINT fk_entity_event_actor_id FOREIGN KEY (actor_id) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE SET NULL;


--
-- Name: entity_event fk_entity_event_entity_id; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.entity_event
    ADD CONSTRAINT fk_entity_event_entity_id FOREIGN KEY (entity_id) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: group fk_group_entity; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska."group"
    ADD CONSTRAINT fk_group_entity FOREIGN KEY (entity_id) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: group_event fk_group_event_entity; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.group_event
    ADD CONSTRAINT fk_group_event_entity FOREIGN KEY (entity_id) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE SET NULL;


--
-- Name: group_event fk_group_event_group; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.group_event
    ADD CONSTRAINT fk_group_event_group FOREIGN KEY ("group") REFERENCES ska."group"(entity_id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: group_member fk_group_member_entity; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.group_member
    ADD CONSTRAINT fk_group_member_entity FOREIGN KEY (entity_id) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: group_member fk_group_member_entity_2; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.group_member
    ADD CONSTRAINT fk_group_member_entity_2 FOREIGN KEY (added_by) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE SET NULL;


--
-- Name: group_member fk_group_member_group; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.group_member
    ADD CONSTRAINT fk_group_member_group FOREIGN KEY ("group") REFERENCES ska."group"(entity_id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: public_key_dest_rule fk_public_key_dest_rule_public_key; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.public_key_dest_rule
    ADD CONSTRAINT fk_public_key_dest_rule_public_key FOREIGN KEY (public_key_id) REFERENCES ska.public_key(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: public_key fk_public_key_entity; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.public_key
    ADD CONSTRAINT fk_public_key_entity FOREIGN KEY (entity_id) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: public_key_signature fk_public_key_signature_public_key; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.public_key_signature
    ADD CONSTRAINT fk_public_key_signature_public_key FOREIGN KEY (public_key_id) REFERENCES ska.public_key(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: server_account fk_server_account_entity; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.server_account
    ADD CONSTRAINT fk_server_account_entity FOREIGN KEY (entity_id) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: server_account fk_server_account_server; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.server_account
    ADD CONSTRAINT fk_server_account_server FOREIGN KEY (server_id) REFERENCES ska.server(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: server_admin fk_server_admin_entity; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.server_admin
    ADD CONSTRAINT fk_server_admin_entity FOREIGN KEY (entity_id) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: server_admin fk_server_admin_server; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.server_admin
    ADD CONSTRAINT fk_server_admin_server FOREIGN KEY (server_id) REFERENCES ska.server(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: server_event fk_server_event_actor_id; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.server_event
    ADD CONSTRAINT fk_server_event_actor_id FOREIGN KEY (actor_id) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE SET NULL;


--
-- Name: server_ldap_access_option fk_server_ldap_access_option_server; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.server_ldap_access_option
    ADD CONSTRAINT fk_server_ldap_access_option_server FOREIGN KEY (server_id) REFERENCES ska.server(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: server_event fk_server_log_server; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.server_event
    ADD CONSTRAINT fk_server_log_server FOREIGN KEY (server_id) REFERENCES ska.server(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: server_note fk_server_note_entity; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.server_note
    ADD CONSTRAINT fk_server_note_entity FOREIGN KEY (entity_id) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE SET NULL;


--
-- Name: server_note fk_server_note_server; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.server_note
    ADD CONSTRAINT fk_server_note_server FOREIGN KEY (server_id) REFERENCES ska.server(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: sync_request fk_sync_request_server; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.sync_request
    ADD CONSTRAINT fk_sync_request_server FOREIGN KEY (server_id) REFERENCES ska.server(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: user_alert fk_user_alert_entity; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska.user_alert
    ADD CONSTRAINT fk_user_alert_entity FOREIGN KEY (entity_id) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- Name: user fk_user_entity; Type: FK CONSTRAINT; Schema: ska; Owner: ska
--

ALTER TABLE ONLY ska."user"
    ADD CONSTRAINT fk_user_entity FOREIGN KEY (entity_id) REFERENCES ska.entity(id) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

