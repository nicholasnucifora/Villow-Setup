#[test]
fn sql_functions_remain_whole_but_transaction_escapes_are_refused() {
    use villow_setup::sql_guard::validate_transactional as check;
    check("-- begin comment\nCREATE FUNCTION public.f() RETURNS void LANGUAGE plpgsql AS $body$ BEGIN PERFORM 'a;b'; END $body$;").unwrap();
    check("/* outer /* COMMIT */ inner */ CREATE TABLE x (s text default 'COMMIT;');").unwrap();
    check(r"SELECT E'escaped \\' quote';").unwrap_err();
    check(r"SELECT E'escaped \' quote';").unwrap();
    assert!(check(r"SELECT '\'; COMMIT; SELECT 'x';").is_err());
    assert!(check(r#"UPDATE "villow_setup".migrations SET checksum='x';"#).is_err());
    check("SELECT CASE WHEN true THEN CASE WHEN false THEN 1 ELSE 2 END ELSE 3 END;").unwrap();
    for sql in [
        "END;",
        "SELECT CASE WHEN true THEN 1 END; END;",
        "SELECT CASE WHEN true THEN 1;",
        "SELECT \"CASE\"; END;",
        "COMMIT; DROP TABLE users; BEGIN;",
        "START TRANSACTION;",
        "CREATE INDEX CONCURRENTLY x ON y(z);",
        "VACUUM;",
        "ROLLBACK;",
        "ALTER SYSTEM SET x=1;",
        "CREATE DATABASE x;",
        "UPDATE villow_setup.migrations SET checksum='x';",
        "SELECT 'unclosed",
        "/* unclosed",
        "DO $$unclosed",
    ] {
        assert!(check(sql).is_err(), "{sql}")
    }
}
