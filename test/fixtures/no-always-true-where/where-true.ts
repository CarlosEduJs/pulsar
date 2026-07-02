import { db } from './db';
import { users } from './schema';

const result = await db.select().from(users).where(true);
