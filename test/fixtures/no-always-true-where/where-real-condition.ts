import { db } from './db';
import { users } from './schema';

const result = await db.select({ id: users.id }).from(users).where(eq(users.id, 1)).limit(10);
