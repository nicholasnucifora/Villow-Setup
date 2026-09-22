-- Paired with the signed app postcondition and pinned column/trigger inventory.
SELECT
EXISTS (
 SELECT FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
 WHERE n.nspname IN ('public','villow_setup') AND c.relkind NOT IN ('r','i')
 AND NOT EXISTS (SELECT FROM pg_depend d WHERE d.classid='pg_class'::regclass AND d.objid=c.oid AND d.deptype='e')
) OR EXISTS (
 SELECT FROM pg_type t JOIN pg_namespace n ON n.oid=t.typnamespace
 WHERE n.nspname IN ('public','villow_setup')
 AND NOT EXISTS (SELECT FROM pg_depend d WHERE d.classid='pg_type'::regclass AND d.objid=t.oid AND d.deptype='e')
 AND NOT EXISTS (SELECT FROM pg_class c WHERE c.oid=t.typrelid AND c.relkind='r')
 AND NOT EXISTS (SELECT FROM pg_type e JOIN pg_class c ON c.oid=e.typrelid WHERE e.oid=t.typelem AND c.relkind='r' AND t.typcategory='A')
) OR EXISTS (
 SELECT FROM pg_rewrite r JOIN pg_class c ON c.oid=r.ev_class JOIN pg_namespace n ON n.oid=c.relnamespace
 WHERE n.nspname IN ('public','villow_setup')
 AND NOT EXISTS (SELECT FROM pg_depend d WHERE d.classid='pg_class'::regclass AND d.objid=c.oid AND d.deptype='e')
) OR EXISTS (
 SELECT FROM pg_depend d JOIN pg_class c ON d.refclassid='pg_class'::regclass AND d.refobjid=c.oid
 JOIN pg_namespace n ON n.oid=c.relnamespace
 JOIN pg_rewrite r ON d.classid='pg_rewrite'::regclass AND d.objid=r.oid
 JOIN pg_class v ON v.oid=r.ev_class JOIN pg_namespace vn ON vn.oid=v.relnamespace
 WHERE n.nspname IN ('public','villow_setup') AND vn.nspname NOT IN ('public','villow_setup')
) OR EXISTS (
 SELECT FROM pg_depend d JOIN pg_class c ON d.refclassid='pg_class'::regclass AND d.refobjid=c.oid
 JOIN pg_namespace n ON n.oid=c.relnamespace
 JOIN pg_proc p ON d.classid='pg_proc'::regclass AND d.objid=p.oid
 JOIN pg_namespace pn ON pn.oid=p.pronamespace
 WHERE n.nspname IN ('public','villow_setup') AND pn.nspname NOT IN ('public','villow_setup','pg_catalog')
) OR EXISTS (
 SELECT FROM pg_trigger t JOIN pg_class c ON c.oid=t.tgrelid JOIN pg_namespace n ON n.oid=c.relnamespace
 JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace pn ON pn.oid=p.pronamespace
 WHERE pn.nspname IN ('public','villow_setup') AND n.nspname NOT IN ('public','villow_setup')
) OR EXISTS (
 SELECT FROM pg_publication_rel pr JOIN pg_class c ON c.oid=pr.prrelid JOIN pg_namespace n ON n.oid=c.relnamespace
 WHERE n.nspname IN ('public','villow_setup')
) OR EXISTS (SELECT FROM pg_publication WHERE puballtables)
OR EXISTS (SELECT FROM pg_publication_namespace pn JOIN pg_namespace n ON n.oid=pn.pnnspid WHERE n.nspname IN ('public','villow_setup'))
OR EXISTS (SELECT FROM pg_largeobject_metadata)
OR EXISTS (
 SELECT FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
 WHERE n.nspname IN ('public','villow_setup') AND c.relkind='r'
 AND (c.relpersistence<>'p' OR c.reloftype<>0 OR c.reloptions IS NOT NULL
 OR EXISTS(SELECT FROM pg_inherits i WHERE i.inhrelid=c.oid OR i.inhparent=c.oid))
 AND NOT EXISTS (SELECT FROM pg_depend d WHERE d.classid='pg_class'::regclass AND d.objid=c.oid AND d.deptype='e')
)
OR EXISTS (SELECT FROM pg_proc WHERE pronamespace='villow_setup'::regnamespace)
OR EXISTS (SELECT FROM pg_namespace WHERE nspname='villow_setup' AND (nspowner<>current_user::regrole OR coalesce(nspacl,acldefault('n',nspowner))<>acldefault('n',nspowner)))
OR EXISTS (
 SELECT FROM pg_class c WHERE c.relnamespace='villow_setup'::regnamespace AND c.relkind='r'
 AND (c.relname NOT IN ('instance','migrations') OR c.relrowsecurity OR c.relforcerowsecurity
 OR coalesce(c.relacl,acldefault('r',c.relowner))<>acldefault('r',c.relowner)
 OR has_table_privilege('anon',c.oid,'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER')
 OR has_table_privilege('authenticated',c.oid,'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER')
 OR has_table_privilege('service_role',c.oid,'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER'))
) OR EXISTS (
 SELECT FROM pg_attribute a JOIN pg_class c ON c.oid=a.attrelid
 WHERE c.relnamespace='villow_setup'::regnamespace AND c.relkind='r' AND a.attnum>0
 AND (a.attisdropped OR NOT a.attnotnull OR a.attidentity<>'' OR a.attgenerated<>''
 OR a.attacl IS NOT NULL OR a.attoptions IS NOT NULL OR a.attcollation<>(SELECT typcollation FROM pg_type WHERE oid=a.atttypid))
)
OR (SELECT count(*) FROM pg_constraint WHERE connamespace='villow_setup'::regnamespace)<>3
OR (SELECT count(*) FROM pg_constraint WHERE connamespace='villow_setup'::regnamespace AND contype='p' AND convalidated)<>2
OR NOT EXISTS (SELECT FROM pg_constraint WHERE conrelid='villow_setup.instance'::regclass AND contype='p' AND NOT condeferrable AND conkey=ARRAY[(SELECT attnum FROM pg_attribute WHERE attrelid='villow_setup.instance'::regclass AND attname='singleton')]::smallint[])
OR NOT EXISTS (SELECT FROM pg_constraint WHERE conrelid='villow_setup.migrations'::regclass AND contype='p' AND NOT condeferrable AND conkey=ARRAY[(SELECT attnum FROM pg_attribute WHERE attrelid='villow_setup.migrations'::regclass AND attname='id')]::smallint[])
OR (SELECT count(*) FROM pg_class WHERE relnamespace='villow_setup'::regnamespace AND relkind='i')<>2
OR NOT EXISTS (SELECT FROM pg_constraint WHERE conrelid='villow_setup.instance'::regclass AND contype='c' AND pg_get_constraintdef(oid)='CHECK (singleton)' AND convalidated)
OR (SELECT count(*) FROM pg_attrdef a JOIN pg_class c ON c.oid=a.adrelid WHERE c.relnamespace='villow_setup'::regnamespace)<>2
OR NOT EXISTS (SELECT FROM pg_attrdef d JOIN pg_attribute a ON a.attrelid=d.adrelid AND a.attnum=d.adnum WHERE d.adrelid='villow_setup.instance'::regclass AND a.attname='singleton' AND pg_get_expr(d.adbin,d.adrelid)='true')
OR NOT EXISTS (SELECT FROM pg_attrdef d JOIN pg_attribute a ON a.attrelid=d.adrelid AND a.attnum=d.adnum WHERE d.adrelid='villow_setup.migrations'::regclass AND a.attname='applied_at' AND pg_get_expr(d.adbin,d.adrelid)='now()');
