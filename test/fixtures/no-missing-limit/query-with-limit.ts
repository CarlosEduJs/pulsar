import { db } from './db';
import { users } from './schema';

const result = await db.select({ id: users.id }).from(users).limit(10);
