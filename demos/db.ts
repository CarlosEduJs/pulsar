// deno-lint-ignore-file
// Stub database client for demo purposes
import type { PgQueryResultHKT } from "drizzle-orm/pg-core";

type AnyDb = import("drizzle-orm/pg-core").PgDatabase<PgQueryResultHKT>;

export const db = {} as AnyDb;
