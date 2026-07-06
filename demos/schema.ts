import {
  boolean,
  integer,
  pgEnum,
  pgTable,
  serial,
  text,
  uniqueIndex,
  varchar,
} from "drizzle-orm/pg-core";

export const roleEnum = pgEnum("role", ["USER", "ADMIN", "MODERATOR"]);

export const users = pgTable(
  "users",
  {
    id: serial("id").primaryKey(),
    email: varchar("email", { length: 255 }).notNull().unique(),
    name: text("name"),
    role: roleEnum("role").default("USER").notNull(),
    isActive: boolean("isActive").default(true).notNull(),
  },
  (table) => [uniqueIndex("email_idx").on(table.email)],
);

export const profile = pgTable("profile", {
  id: serial("id").primaryKey(),
  bio: text("bio"),
  userId: integer("userId")
    .notNull()
    .references(() => users.id)
    .unique(),
});

export const posts = pgTable(
  "posts",
  {
    id: serial("id").primaryKey(),
    title: varchar("title", { length: 255 }).notNull(),
    content: text("content"),
    published: boolean("published").default(false).notNull(),
    authorId: integer("authorId")
      .notNull()
      .references(() => users.id),
    categoryId: integer("categoryId").references(() => categories.id),
  },
  (table) => [uniqueIndex("author_id_idx").on(table.authorId)],
);

export const categories = pgTable("categories", {
  id: serial("id").primaryKey(),
  name: varchar("name", { length: 255 }).notNull().unique(),
  isActive: boolean("isActive").default(true).notNull(),
});
