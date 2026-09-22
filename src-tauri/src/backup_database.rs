//! Bounded COPY backup of the authenticated 0.1.1 app and native ledger.
//! This is not a provider-wide dump or permission to overwrite a database.
use crate::{
    error::{Error, Result},
    model::Installation,
    release::VerifiedRelease,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use postgres::{Client, IsolationLevel, Transaction};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Read, Write},
};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

pub const SOURCE: &str = "37072f3ef4efc387b57f6edcccdfd0e71b09fc61abcaee4649f9aba76fa332ac";
pub const DESTINATION: &str = "46da117bce94a5af20f6680a1188a804866aec4ca302f9862686c2e869e5c78a";
const LOCK: i64 = 0x56494c4c4f57;
const MAX_DATA: usize = 128 * 1024 * 1024;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Zeroize, ZeroizeOnDrop)]
#[serde(deny_unknown_fields)]
pub struct Column {
    pub name: String,
    #[serde(rename = "type")]
    pub sql_type: String,
    pub identity: String,
    pub generated: String,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Zeroize, ZeroizeOnDrop)]
#[serde(deny_unknown_fields)]
pub struct TableSchema {
    pub schema: String,
    pub name: String,
    pub columns: Vec<Column>,
    pub rls: bool,
    pub force_rls: bool,
}
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(deny_unknown_fields)]
pub struct Table {
    pub schema: TableSchema,
    pub rows: u64,
    pub copy_base64: String,
}
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(deny_unknown_fields)]
pub struct DatabaseSnapshot {
    pub format: u32,
    pub server_major: i32,
    pub tables: Vec<Table>,
}

fn ident(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}
fn name(table: &TableSchema) -> String {
    format!("{}.{}", ident(&table.schema), ident(&table.name))
}
fn columns(table: &TableSchema) -> String {
    table
        .columns
        .iter()
        .map(|c| ident(&c.name))
        .collect::<Vec<_>>()
        .join(",")
}
fn copy_query(table: &TableSchema) -> String {
    format!("COPY (SELECT {} FROM ONLY {} v ORDER BY to_jsonb(v)::text COLLATE \"C\") TO STDOUT WITH (FORMAT text)",columns(table),name(table))
}
fn settings(tx: &mut Transaction<'_>) -> Result<()> {
    tx.batch_execute("SET LOCAL row_security=off; SET LOCAL TimeZone='UTC'; SET LOCAL DateStyle='ISO, YMD'; SET LOCAL IntervalStyle='postgres'; SET LOCAL extra_float_digits=3; SET LOCAL bytea_output='hex'; SET LOCAL client_encoding='UTF8'; SET LOCAL standard_conforming_strings=on; SET LOCAL search_path=public,pg_catalog; SET LOCAL lock_timeout='10s'; SET LOCAL statement_timeout='120s'").map_err(|_|Error::BackupDatabase)
}
fn expected() -> Result<Vec<TableSchema>> {
    serde_json::from_str(include_str!("../backup-tables-0.1.1.json").trim_start_matches('\u{feff}'))
        .map_err(|_| Error::BackupDatabase)
}
fn scope(tx: &mut Transaction<'_>) -> Result<Vec<TableSchema>> {
    // The signed app postcondition covers columns/defaults/constraints, routines,
    // triggers, policies and ACLs. Also refuse unsupported objects/native drift.
    let unsupported: bool = tx
        .query_one(include_str!("backup_scope.sql"), &[])
        .map_err(|_| Error::BackupDatabase)?
        .get(0);
    if unsupported {
        return Err(Error::BackupDatabase);
    }
    let rows=tx.query("SELECT n.nspname,c.relname,c.oid::bigint,c.relowner=current_user::regrole,c.relrowsecurity,c.relforcerowsecurity FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname IN ('public','villow_setup') AND c.relkind='r' AND NOT EXISTS(SELECT FROM pg_depend d WHERE d.classid='pg_class'::regclass AND d.objid=c.oid AND d.deptype='e') ORDER BY n.nspname,c.relname",&[]).map_err(|_|Error::BackupDatabase)?;
    if rows.len() != 62 {
        return Err(Error::BackupDatabase);
    }
    let mut tables = Vec::new();
    for row in rows {
        if !row.get::<_, bool>(3) {
            return Err(Error::BackupDatabase);
        }
        let oid: i64 = row.get(2);
        let cols=tx.query("SELECT attname,format_type(atttypid,atttypmod),attidentity::text,attgenerated::text FROM pg_attribute WHERE attrelid=$1::bigint::oid AND attnum>0 AND NOT attisdropped ORDER BY attnum",&[&oid]).map_err(|_|Error::BackupDatabase)?;
        tables.push(TableSchema {
            schema: row.get(0),
            name: row.get(1),
            rls: row.get(4),
            force_rls: row.get(5),
            columns: cols
                .iter()
                .map(|c| Column {
                    name: c.get(0),
                    sql_type: c.get(1),
                    identity: c.get(2),
                    generated: c.get(3),
                })
                .collect(),
        });
    }
    if tables != expected()? {
        return Err(Error::BackupDatabase);
    }
    let trigger_text:String=tx.query_one("SELECT coalesce(jsonb_agg(jsonb_build_object('table',c.relname,'name',t.tgname,'enabled',t.tgenabled,'definition',pg_get_triggerdef(t.oid)) ORDER BY c.relname,t.tgname),'[]')::text FROM pg_trigger t JOIN pg_class c ON c.oid=t.tgrelid WHERE c.relnamespace IN ('public'::regnamespace,'villow_setup'::regnamespace) AND NOT t.tgisinternal",&[]).map_err(|_|Error::BackupDatabase)?.get(0);
    let triggers: serde_json::Value =
        serde_json::from_str(&trigger_text).map_err(|_| Error::BackupDatabase)?;
    let expected_triggers: serde_json::Value = serde_json::from_str(
        include_str!("../backup-triggers-0.1.1.json").trim_start_matches('\u{feff}'),
    )
    .map_err(|_| Error::BackupDatabase)?;
    if triggers != expected_triggers {
        return Err(Error::BackupDatabase);
    }
    restore_order(tx, &tables)?;
    Ok(tables)
}
fn restore_order(tx: &mut Transaction<'_>, tables: &[TableSchema]) -> Result<Vec<usize>> {
    let indices = tables
        .iter()
        .enumerate()
        .map(|(i, t)| ((t.schema.clone(), t.name.clone()), i))
        .collect::<BTreeMap<_, _>>();
    let mut dependencies = vec![BTreeSet::new(); tables.len()];
    let rows=tx.query("SELECT ns.nspname,c.relname,nt.nspname,t.relname FROM pg_constraint f JOIN pg_class c ON c.oid=f.conrelid JOIN pg_namespace ns ON ns.oid=c.relnamespace JOIN pg_class t ON t.oid=f.confrelid JOIN pg_namespace nt ON nt.oid=t.relnamespace WHERE f.contype='f' AND (ns.nspname IN ('public','villow_setup') OR nt.nspname IN ('public','villow_setup'))",&[]).map_err(|_|Error::BackupDatabase)?;
    if rows.len() != 71 {
        return Err(Error::BackupDatabase);
    }
    for r in rows {
        let from = (r.get::<_, String>(0), r.get::<_, String>(1));
        let to = (r.get::<_, String>(2), r.get::<_, String>(3));
        let (Some(&from), Some(&to)) = (indices.get(&from), indices.get(&to)) else {
            return Err(Error::BackupDatabase);
        };
        dependencies[from].insert(to);
    }
    let mut done = BTreeSet::new();
    let mut order = Vec::new();
    while order.len() < tables.len() {
        let next = (0..tables.len())
            .find(|i| !done.contains(i) && dependencies[*i].is_subset(&done))
            .ok_or(Error::BackupDatabase)?;
        done.insert(next);
        order.push(next);
    }
    Ok(order)
}
fn read_table(
    tx: &mut Transaction<'_>,
    table: &TableSchema,
    remaining: usize,
) -> Result<Zeroizing<Vec<u8>>> {
    let mut data = Zeroizing::new(Vec::new());
    tx.copy_out(&copy_query(table))
        .map_err(|_| Error::BackupDatabase)?
        .take((remaining + 1) as u64)
        .read_to_end(&mut data)
        .map_err(|_| Error::BackupDatabase)?;
    if data.len() > remaining {
        return Err(Error::BackupTooLarge);
    }
    Ok(data)
}
pub fn capture(
    client: &mut Client,
    s: &Installation,
    old: &VerifiedRelease,
) -> Result<DatabaseSnapshot> {
    if old.digest != SOURCE || s.release_digest != SOURCE {
        return Err(Error::BackupDatabase);
    }
    let locked: bool = client
        .query_one("SELECT pg_try_advisory_lock($1)", &[&LOCK])
        .map_err(|_| Error::BackupDatabase)?
        .get(0);
    if !locked {
        return Err(Error::Busy);
    }
    let result = (|| {
        let mut tx = client
            .build_transaction()
            .isolation_level(IsolationLevel::RepeatableRead)
            .read_only(true)
            .start()
            .map_err(|_| Error::BackupDatabase)?;
        settings(&mut tx)?;
        let tables = expected()?;
        tx.batch_execute(&format!(
            "LOCK TABLE {} IN ACCESS SHARE MODE",
            tables.iter().map(name).collect::<Vec<_>>().join(",")
        ))
        .map_err(|_| Error::BackupDatabase)?;
        crate::repair_database::validate_backup_source(&mut tx, s, old)?;
        let tables = scope(&mut tx)?;
        let server: i32 = tx
            .query_one("SELECT current_setting('server_version_num')::int", &[])
            .map_err(|_| Error::BackupDatabase)?
            .get(0);
        let mut result = DatabaseSnapshot {
            format: 1,
            server_major: server / 10000,
            tables: Vec::new(),
        };
        let mut remaining = MAX_DATA;
        for table in tables {
            let rows: i64 = tx
                .query_one(&format!("SELECT count(*) FROM ONLY {}", name(&table)), &[])
                .map_err(|_| Error::BackupDatabase)?
                .get(0);
            let data = read_table(&mut tx, &table, remaining)?;
            remaining -= data.len();
            result.tables.push(Table {
                schema: table,
                rows: rows.try_into().map_err(|_| Error::BackupDatabase)?,
                copy_base64: STANDARD.encode(&data),
            });
        }
        tx.commit().map_err(|_| Error::BackupDatabase)?;
        Ok(result)
    })();
    let _ = client.execute("SELECT pg_advisory_unlock($1)", &[&LOCK]);
    result
}

/// SQL restore primitive only; no IPC or automatic target adoption calls this.
/// The caller must independently authorize a recovery target. Check namespace
/// emptiness and replay only authenticated original SQL in one transaction.
/// Existing app data can never be truncated, deleted or overwritten here.
pub fn restore_empty(
    client: &mut Client,
    s: &Installation,
    old: &VerifiedRelease,
    snapshot: &DatabaseSnapshot,
) -> Result<()> {
    if old.digest != SOURCE
        || s.release_digest != SOURCE
        || snapshot.format != 1
        || snapshot
            .tables
            .iter()
            .map(|t| t.schema.clone())
            .collect::<Vec<_>>()
            != expected()?
    {
        return Err(Error::BackupInvalid);
    }
    let locked: bool = client
        .query_one("SELECT pg_try_advisory_lock($1)", &[&LOCK])
        .map_err(|_| Error::BackupDatabase)?
        .get(0);
    if !locked {
        return Err(Error::Busy);
    }
    let result = (|| {
        let mut tx = client.transaction().map_err(|_| Error::BackupDatabase)?;
        settings(&mut tx)?;
        let existing:bool=tx.query_one("SELECT EXISTS(SELECT FROM pg_namespace WHERE nspname='villow_setup') OR EXISTS(SELECT FROM pg_class c WHERE c.relnamespace='public'::regnamespace AND NOT EXISTS(SELECT FROM pg_depend d WHERE d.classid='pg_class'::regclass AND d.objid=c.oid AND d.deptype='e')) OR EXISTS(SELECT FROM pg_proc p WHERE p.pronamespace='public'::regnamespace AND NOT EXISTS(SELECT FROM pg_depend d WHERE d.classid='pg_proc'::regclass AND d.objid=p.oid AND d.deptype='e'))",&[]).map_err(|_|Error::BackupDatabase)?.get(0);
        let server: i32 = tx
            .query_one("SELECT current_setting('server_version_num')::int", &[])
            .map_err(|_| Error::BackupDatabase)?
            .get(0);
        if existing || snapshot.server_major != server / 10000 {
            return Err(Error::BackupDatabase);
        }
        tx.batch_execute(crate::migration::NATIVE_DDL)
            .map_err(|_| Error::BackupDatabase)?;
        for unit in &old.manifest.schema.migrations {
            let sql = std::str::from_utf8(old.files.get(&unit.file).ok_or(Error::Release)?)
                .map_err(|_| Error::Release)?;
            tx.batch_execute(sql).map_err(|_| Error::BackupDatabase)?;
        }
        let tables = scope(&mut tx)?;
        for table in &tables {
            let count: i64 = tx
                .query_one(&format!("SELECT count(*) FROM ONLY {}", name(table)), &[])
                .map_err(|_| Error::BackupDatabase)?
                .get(0);
            if count != 0 {
                return Err(Error::BackupDatabase);
            }
            tx.batch_execute(&format!("ALTER TABLE {} DISABLE TRIGGER USER", name(table)))
                .map_err(|_| Error::BackupDatabase)?;
        }
        let mut remaining = MAX_DATA;
        for index in restore_order(&mut tx, &tables)? {
            let table = &snapshot.tables[index];
            if table.copy_base64.len() > remaining.div_ceil(3) * 4 {
                return Err(Error::BackupTooLarge);
            }
            let data = Zeroizing::new(
                STANDARD
                    .decode(&table.copy_base64)
                    .map_err(|_| Error::BackupInvalid)?,
            );
            remaining = remaining
                .checked_sub(data.len())
                .ok_or(Error::BackupTooLarge)?;
            let mut copy = tx
                .copy_in(&format!(
                    "COPY {} ({}) FROM STDIN WITH (FORMAT text)",
                    name(&table.schema),
                    columns(&table.schema)
                ))
                .map_err(|_| Error::BackupDatabase)?;
            copy.write_all(&data).map_err(|_| Error::BackupDatabase)?;
            if copy.finish().map_err(|_| Error::BackupDatabase)? != table.rows {
                return Err(Error::BackupInvalid);
            }
        }
        for table in &tables {
            tx.batch_execute(&format!("ALTER TABLE {} ENABLE TRIGGER USER", name(table)))
                .map_err(|_| Error::BackupDatabase)?;
        }
        for table in &snapshot.tables {
            let actual = read_table(&mut tx, &table.schema, MAX_DATA)?;
            if STANDARD.encode(&actual) != table.copy_base64 {
                return Err(Error::BackupInvalid);
            }
        }
        scope(&mut tx)?;
        crate::repair_database::validate_backup_source(&mut tx, s, old)?;
        tx.commit().map_err(|_| Error::Uncertain)
    })();
    let _ = client.execute("SELECT pg_advisory_unlock($1)", &[&LOCK]);
    result
}
