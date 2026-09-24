//! A destination-signed public inventory plus native-owned ledger contract.
//! Restore is empty-target only; this is never an in-place rollback.
use crate::{
    backup_database::{
        columns, name, read_table, settings, Column, DatabaseSnapshot, Table, TableSchema,
    },
    error::{Error, Result},
    model::Installation,
    release::VerifiedRelease,
    update_contract::{self, SourceBackup},
    update_database,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use postgres::{Client, IsolationLevel, Transaction};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
};
use zeroize::Zeroizing;
const MAX_DATA: usize = 128 * 1024 * 1024;
fn native_table(name: &str, cols: &[(&str, &str)]) -> TableSchema {
    TableSchema {
        schema: "villow_setup".into(),
        name: name.into(),
        columns: cols
            .iter()
            .map(|(name, t)| Column {
                name: (*name).into(),
                sql_type: (*t).into(),
                identity: String::new(),
                generated: String::new(),
            })
            .collect(),
        rls: false,
        force_rls: false,
    }
}
fn expected(s: &Installation, d: &SourceBackup) -> Result<Vec<TableSchema>> {
    let mut tables = d.tables.clone();
    tables.extend(native_expected(s)?);
    tables.sort_by(|a, b| (&a.schema, &a.name).cmp(&(&b.schema, &b.name)));
    Ok(tables)
}
fn native_expected(s: &Installation) -> Result<Vec<TableSchema>> {
    let lineage = s.update_lineage.as_ref().ok_or(Error::BackupDatabase)?;
    let mut tables = Vec::new();
    tables.push(native_table(
        "instance",
        &[
            ("singleton", "boolean"),
            ("installation_id", "text"),
            ("release_digest", "text"),
        ],
    ));
    tables.push(native_table(
        "migrations",
        &[
            ("id", "text"),
            ("checksum", "text"),
            ("postcondition_checksum", "text"),
            ("applied_at", "timestamp with time zone"),
        ],
    ));
    if lineage.base.repair.is_some() {
        tables.push(native_table(
            "repairs",
            &[
                ("repair_id", "text"),
                ("installation_id", "text"),
                ("from_digest", "text"),
                ("to_digest", "text"),
                ("sql_checksum", "text"),
                ("postcondition_checksum", "text"),
                ("operation_id", "text"),
                ("applied_at", "timestamp with time zone"),
            ],
        ));
    }
    if !lineage.receipts.is_empty() {
        tables.push(native_table(
            "app_updates",
            &[
                ("position", "integer"),
                ("receipt", "jsonb"),
                ("receipt_hash", "text"),
                ("applied_at", "timestamp with time zone"),
            ],
        ));
    }
    tables.sort_by(|a, b| (&a.schema, &a.name).cmp(&(&b.schema, &b.name)));
    Ok(tables)
}
pub(crate) fn scope(
    tx: &mut Transaction<'_>,
    s: &Installation,
    d: &SourceBackup,
) -> Result<Vec<TableSchema>> {
    let unsupported: bool = tx
        .query_one(include_str!("update_backup_scope.sql"), &[])
        .map_err(|_| Error::BackupDatabase)?
        .get(0);
    if unsupported {
        return Err(Error::BackupDatabase);
    }
    let expected = expected(s, d)?;
    let rows=tx.query("SELECT n.nspname,c.relname,c.oid::bigint,c.relowner=current_user::regrole,c.relrowsecurity,c.relforcerowsecurity FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname IN ('public','villow_setup') AND c.relkind='r' AND NOT EXISTS(SELECT FROM pg_depend d WHERE d.classid='pg_class'::regclass AND d.objid=c.oid AND d.deptype='e') ORDER BY n.nspname,c.relname",&[]).map_err(|_|Error::BackupDatabase)?;
    if rows.len() != expected.len() {
        return Err(Error::BackupDatabase);
    }
    for (row, t) in rows.iter().zip(&expected) {
        if !row.get::<_, bool>(3)
            || row.get::<_, String>(0) != t.schema
            || row.get::<_, String>(1) != t.name
            || row.get::<_, bool>(4) != t.rls
            || row.get::<_, bool>(5) != t.force_rls
        {
            return Err(Error::BackupDatabase);
        }
        let oid: i64 = row.get(2);
        let cols=tx.query("SELECT attname,format_type(atttypid,atttypmod),attidentity::text,attgenerated::text FROM pg_attribute WHERE attrelid=$1::bigint::oid AND attnum>0 AND NOT attisdropped ORDER BY attnum",&[&oid]).map_err(|_|Error::BackupDatabase)?;
        let actual = cols
            .iter()
            .map(|c| Column {
                name: c.get(0),
                sql_type: c.get(1),
                identity: c.get(2),
                generated: c.get(3),
            })
            .collect::<Vec<_>>();
        if actual != t.columns {
            return Err(Error::BackupDatabase);
        }
    }
    let triggers:String=tx.query_one("SELECT coalesce(jsonb_agg(jsonb_build_object('table',c.relname,'name',t.tgname,'enabled',t.tgenabled,'definition',pg_get_triggerdef(t.oid)) ORDER BY c.relname,t.tgname),'[]')::text FROM pg_trigger t JOIN pg_class c ON c.oid=t.tgrelid WHERE c.relnamespace IN ('public'::regnamespace,'villow_setup'::regnamespace) AND NOT t.tgisinternal",&[]).map_err(|_|Error::BackupDatabase)?.get(0);
    let actual: Vec<update_contract::Trigger> =
        serde_json::from_str(&triggers).map_err(|_| Error::BackupDatabase)?;
    if actual != d.triggers {
        return Err(Error::BackupDatabase);
    }
    // Native table structure is fixed by this executable, never the descriptor.
    native_scope(tx, s)?;
    restore_order(tx, &expected)?;
    Ok(expected)
}
pub(crate) fn native_scope(tx: &mut Transaction<'_>, s: &Installation) -> Result<()> {
    let unsupported: bool = tx
        .query_one(include_str!("update_backup_scope.sql"), &[])
        .map_err(|_| Error::BackupDatabase)?
        .get(0);
    if unsupported {
        return Err(Error::BackupDatabase);
    }
    let native = native_expected(s)?;
    let rows=tx.query("SELECT c.relname,c.oid::bigint,c.relowner=current_user::regrole FROM pg_class c WHERE c.relnamespace='villow_setup'::regnamespace AND c.relkind='r' ORDER BY c.relname",&[]).map_err(|_|Error::BackupDatabase)?;
    if rows.len() != native.len() {
        return Err(Error::BackupDatabase);
    }
    for (row, t) in rows.iter().zip(&native) {
        if row.get::<_, String>(0) != t.name || !row.get::<_, bool>(2) {
            return Err(Error::BackupDatabase);
        }
        let oid: i64 = row.get(1);
        let columns=tx.query("SELECT attname,format_type(atttypid,atttypmod) FROM pg_attribute WHERE attrelid=$1::bigint::oid AND attnum>0 AND NOT attisdropped ORDER BY attnum",&[&oid]).map_err(|_|Error::BackupDatabase)?;
        if columns.len() != t.columns.len()
            || columns.iter().zip(&t.columns).any(|(a, b)| {
                a.get::<_, String>(0) != b.name || a.get::<_, String>(1) != b.sql_type
            })
        {
            return Err(Error::BackupDatabase);
        }
    }
    let triggers:bool=tx.query_one("SELECT EXISTS(SELECT FROM pg_trigger t JOIN pg_class c ON c.oid=t.tgrelid WHERE c.relnamespace='villow_setup'::regnamespace AND NOT t.tgisinternal)",&[]).map_err(|_|Error::BackupDatabase)?.get(0);
    if triggers {
        return Err(Error::BackupDatabase);
    }
    let constraints=tx.query("SELECT c.relname,k.contype::text,pg_get_constraintdef(k.oid),k.convalidated,k.condeferrable,ARRAY(SELECT a.attname::text FROM unnest(k.conkey) WITH ORDINALITY ck(attnum,ord) JOIN pg_attribute a ON a.attrelid=k.conrelid AND a.attnum=ck.attnum ORDER BY ck.ord) FROM pg_constraint k JOIN pg_class c ON c.oid=k.conrelid WHERE k.connamespace='villow_setup'::regnamespace ORDER BY c.relname,k.contype",&[]).map_err(|_|Error::BackupDatabase)?;
    let mut required = vec![(
        "instance".to_string(),
        "c".to_string(),
        "CHECK (singleton)".to_string(),
    )];
    for t in &native {
        required.push((
            t.name.clone(),
            "p".into(),
            format!("PRIMARY KEY ({})", t.columns[0].name),
        ));
    }
    let actual = constraints
        .iter()
        .map(|r| {
            (
                r.get::<_, String>(0),
                r.get::<_, String>(1),
                if r.get::<_, String>(1) == "p" {
                    format!("PRIMARY KEY ({})", r.get::<_, Vec<String>>(5).join(", "))
                } else {
                    r.get::<_, String>(2)
                },
            )
        })
        .collect::<BTreeSet<_>>();
    if actual != required.into_iter().collect()
        || constraints.len() != native.len() + 1
        || constraints
            .iter()
            .any(|r| !r.get::<_, bool>(3) || r.get::<_, bool>(4))
    {
        return Err(Error::BackupDatabase);
    }
    let indexes:i64=tx.query_one("SELECT count(*) FROM pg_class WHERE relnamespace='villow_setup'::regnamespace AND relkind='i'",&[]).map_err(|_|Error::BackupDatabase)?.get(0);
    if indexes != native.len() as i64 {
        return Err(Error::BackupDatabase);
    }
    let defaults=tx.query("SELECT c.relname,a.attname,pg_get_expr(d.adbin,d.adrelid) FROM pg_attrdef d JOIN pg_class c ON c.oid=d.adrelid JOIN pg_attribute a ON a.attrelid=d.adrelid AND a.attnum=d.adnum WHERE c.relnamespace='villow_setup'::regnamespace",&[]).map_err(|_|Error::BackupDatabase)?;
    let actual = defaults
        .iter()
        .map(|r| {
            (
                r.get::<_, String>(0),
                r.get::<_, String>(1),
                r.get::<_, String>(2),
            )
        })
        .collect::<BTreeSet<_>>();
    let required = native
        .iter()
        .map(|t| {
            if t.name == "instance" {
                (t.name.clone(), "singleton".into(), "true".into())
            } else {
                (t.name.clone(), "applied_at".into(), "now()".into())
            }
        })
        .collect();
    if actual != required {
        return Err(Error::BackupDatabase);
    }
    Ok(())
}
fn restore_order(tx: &mut Transaction<'_>, tables: &[TableSchema]) -> Result<Vec<usize>> {
    let indices = tables
        .iter()
        .enumerate()
        .map(|(i, t)| ((t.schema.clone(), t.name.clone()), i))
        .collect::<BTreeMap<_, _>>();
    let mut dependencies = vec![BTreeSet::new(); tables.len()];
    let rows=tx.query("SELECT ns.nspname,c.relname,nt.nspname,t.relname FROM pg_constraint f JOIN pg_class c ON c.oid=f.conrelid JOIN pg_namespace ns ON ns.oid=c.relnamespace JOIN pg_class t ON t.oid=f.confrelid JOIN pg_namespace nt ON nt.oid=t.relnamespace WHERE f.contype='f' AND (ns.nspname IN ('public','villow_setup') OR nt.nspname IN ('public','villow_setup'))",&[]).map_err(|_|Error::BackupDatabase)?;
    for r in rows {
        let from = (r.get::<_, String>(0), r.get::<_, String>(1));
        let to = (r.get::<_, String>(2), r.get::<_, String>(3));
        let (Some(&a), Some(&b)) = (indices.get(&from), indices.get(&to)) else {
            return Err(Error::BackupDatabase);
        };
        dependencies[a].insert(b);
    }
    let mut done = BTreeSet::new();
    let mut order = vec![];
    while order.len() < tables.len() {
        let next = (0..tables.len())
            .find(|i| !done.contains(i) && dependencies[*i].is_subset(&done))
            .ok_or(Error::BackupDatabase)?;
        done.insert(next);
        order.push(next);
    }
    Ok(order)
}
pub fn capture(
    client: &mut Client,
    s: &Installation,
    old: &VerifiedRelease,
    new: &VerifiedRelease,
) -> Result<DatabaseSnapshot> {
    let plan = update_contract::transition(old, new)?;
    let d = update_contract::descriptor(plan, &new.files)?;
    locked(client, |client| {
        let mut tx = client
            .build_transaction()
            .isolation_level(IsolationLevel::RepeatableRead)
            .read_only(true)
            .start()
            .map_err(|_| Error::BackupDatabase)?;
        settings(&mut tx)?;
        let tables = expected(s, &d)?;
        tx.batch_execute(&format!(
            "LOCK TABLE {} IN ACCESS SHARE MODE",
            tables.iter().map(name).collect::<Vec<_>>().join(",")
        ))
        .map_err(|_| Error::BackupDatabase)?;
        update_database::source(&mut tx, s, old)?;
        let tables = scope(&mut tx, s, &d)?;
        let server: i32 = tx
            .query_one("SELECT current_setting('server_version_num')::int", &[])
            .map_err(|_| Error::BackupDatabase)?
            .get(0);
        let mut snapshot = DatabaseSnapshot {
            format: 1,
            server_major: server / 10000,
            tables: vec![],
        };
        let mut remaining = MAX_DATA;
        for table in tables {
            let rows: i64 = tx
                .query_one(&format!("SELECT count(*) FROM ONLY {}", name(&table)), &[])
                .map_err(|_| Error::BackupDatabase)?
                .get(0);
            let data = read_table(&mut tx, &table, remaining)?;
            remaining -= data.len();
            snapshot.tables.push(Table {
                schema: table,
                rows: rows.try_into().map_err(|_| Error::BackupDatabase)?,
                copy_base64: STANDARD.encode(&data),
            });
        }
        tx.commit().map_err(|_| Error::BackupDatabase)?;
        Ok(snapshot)
    })
}
fn locked<T>(client: &mut Client, action: impl FnOnce(&mut Client) -> Result<T>) -> Result<T> {
    let acquired: bool = client
        .query_one("SELECT pg_try_advisory_lock($1)", &[&update_database::LOCK])
        .map_err(|_| Error::BackupDatabase)?
        .get(0);
    if !acquired {
        return Err(Error::Busy);
    }
    let result = action(client);
    let _ = client.execute("SELECT pg_advisory_unlock($1)", &[&update_database::LOCK]);
    result
}
pub fn restore_empty(
    client: &mut Client,
    s: &Installation,
    old: &VerifiedRelease,
    new: &VerifiedRelease,
    snapshot: &DatabaseSnapshot,
) -> Result<()> {
    let plan = update_contract::transition(old, new)?;
    let d = update_contract::descriptor(plan, &new.files)?;
    if snapshot.format != 1
        || snapshot
            .tables
            .iter()
            .map(|t| t.schema.clone())
            .collect::<Vec<_>>()
            != expected(s, &d)?
    {
        return Err(Error::BackupInvalid);
    }
    locked(client, |client| {
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
        let lineage = s.update_lineage.as_ref().ok_or(Error::BackupDatabase)?;
        if lineage.base.repair.is_some() {
            tx.batch_execute(update_database::REPAIR_DDL)
                .map_err(|_| Error::BackupDatabase)?;
        }
        if !lineage.receipts.is_empty() {
            tx.batch_execute(update_database::UPDATE_DDL)
                .map_err(|_| Error::BackupDatabase)?;
        }
        tx.batch_execute(crate::repair_database::text(old, &d.restore_baseline)?)
            .map_err(|_| Error::BackupDatabase)?;
        let tables = scope(&mut tx, s, &d)?;
        for t in &tables {
            let count: i64 = tx
                .query_one(&format!("SELECT count(*) FROM ONLY {}", name(t)), &[])
                .map_err(|_| Error::BackupDatabase)?
                .get(0);
            if count != 0 {
                return Err(Error::BackupDatabase);
            }
            tx.batch_execute(&format!("ALTER TABLE {} DISABLE TRIGGER USER", name(t)))
                .map_err(|_| Error::BackupDatabase)?;
        }
        let mut remaining = MAX_DATA;
        for i in restore_order(&mut tx, &tables)? {
            let t = &snapshot.tables[i];
            if t.copy_base64.len() > remaining.div_ceil(3) * 4 {
                return Err(Error::BackupTooLarge);
            }
            let data = Zeroizing::new(
                STANDARD
                    .decode(&t.copy_base64)
                    .map_err(|_| Error::BackupInvalid)?,
            );
            remaining = remaining
                .checked_sub(data.len())
                .ok_or(Error::BackupTooLarge)?;
            let mut copy = tx
                .copy_in(&format!(
                    "COPY {} ({}) FROM STDIN WITH (FORMAT text)",
                    name(&t.schema),
                    columns(&t.schema)
                ))
                .map_err(|_| Error::BackupDatabase)?;
            copy.write_all(&data).map_err(|_| Error::BackupDatabase)?;
            if copy.finish().map_err(|_| Error::BackupDatabase)? != t.rows {
                return Err(Error::BackupInvalid);
            }
        }
        for t in &tables {
            tx.batch_execute(&format!("ALTER TABLE {} ENABLE TRIGGER USER", name(t)))
                .map_err(|_| Error::BackupDatabase)?;
        }
        for t in &snapshot.tables {
            if STANDARD.encode(read_table(&mut tx, &t.schema, MAX_DATA)?) != t.copy_base64 {
                return Err(Error::BackupInvalid);
            }
        }
        scope(&mut tx, s, &d)?;
        update_database::source(&mut tx, s, old)?;
        tx.commit().map_err(|_| Error::Uncertain)
    })
}
