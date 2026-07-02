import { db } from './db';
import { users } from './schema';

for (let i = 0; i < 10; i++) {
  const result = await db.select().from(users);
}
