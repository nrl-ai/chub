---
name: distributed-db
description: "CockroachDB with node-postgres (pg) - JavaScript guide for connecting to CockroachDB from Node.js applications"
metadata:
  languages: "javascript"
  versions: "8.16.3"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "cockroachdb,distributed-db,sql,postgres,database"
---

# CockroachDB with node-postgres (pg) - JavaScript Guide

## Golden Rule

**ALWAYS use the `pg` (node-postgres) package version 8.16.3 or later** to connect to CockroachDB from Node.js applications.

```bash
npm install pg
```

CockroachDB is PostgreSQL wire-compatible, meaning it uses the PostgreSQL protocol. The official recommendation is to use the standard PostgreSQL `pg` driver (node-postgres) for JavaScript/Node.js applications.

**DO NOT use:**
- Unofficial or deprecated CockroachDB-specific packages
- Old versions of `pg` that may not support modern features
- Random third-party wrappers without proper maintenance

**ALWAYS use `pg` (node-postgres)** - it is the officially supported and recommended driver.

---

## Installation

### Basic Installation

```bash
npm install pg
```

### With TypeScript Support

```bash
npm install pg @types/pg
```

### Additional Dependencies (Optional)

For SSL connections with custom certificates:

```bash
npm install pg fs
```

---

## Environment Variables

### Basic Configuration

Create a `.env` file:

```bash
# Database connection
DATABASE_URL=postgresql://root@localhost:26257/defaultdb?sslmode=disable

# Or separate variables
DB_USER=root
DB_HOST=localhost
DB_PORT=26257
DB_NAME=defaultdb
DB_PASSWORD=
```

### Secure Production Configuration

```bash
# CockroachDB Serverless/Cloud
DATABASE_URL=postgresql://username:password@host.cockroachlabs.cloud:26257/database?sslmode=verify-full

# With certificate paths
DB_USER=maxroach
DB_HOST=blue-dog-147.6wr.cockroachlabs.cloud
DB_PORT=26257
DB_NAME=defaultdb
DB_PASSWORD=YourSecurePassword
DB_SSL_MODE=verify-full
DB_SSL_ROOT_CERT=/path/to/root.crt
DB_SSL_CERT=/path/to/client.crt
DB_SSL_KEY=/path/to/client.key
```

### Loading Environment Variables

```javascript
require('dotenv').config();

const config = {
  user: process.env.DB_USER || 'root',
  host: process.env.DB_HOST || 'localhost',
  database: process.env.DB_NAME || 'defaultdb',
  password: process.env.DB_PASSWORD || '',
  port: parseInt(process.env.DB_PORT) || 26257,
};
```

---

## Initialization

### Basic Client Connection

```javascript
const { Client } = require('pg');

const client = new Client({
  user: 'root',
  host: 'localhost',
  database: 'defaultdb',
  port: 26257,
  ssl: false
});

client.connect()
  .then(() => console.log('Connected to CockroachDB'))
  .catch(err => console.error('Connection error', err.stack));
```

### Connection Pool (Recommended)

```javascript
const { Pool } = require('pg');

const pool = new Pool({
  user: 'root',
  host: 'localhost',
  database: 'defaultdb',
  port: 26257,
  max: 10,                    // Maximum pool size
  min: 2,                     // Minimum pool size
  idleTimeoutMillis: 30000,   // Close idle clients after 30 seconds
  connectionTimeoutMillis: 2000, // Return error after 2 seconds if connection unavailable
});

// Test the connection
pool.query('SELECT NOW()', (err, res) => {
  if (err) {
    console.error('Error executing query', err.stack);
  } else {
    console.log('Connected to CockroachDB:', res.rows[0]);
  }
});
```

### Async/Await Pool Connection

```javascript
const { Pool } = require('pg');

const pool = new Pool({
  user: 'root',
  host: 'localhost',
  database: 'defaultdb',
  port: 26257,
});

async function testConnection() {
  try {
    const client = await pool.connect();
    const res = await client.query('SELECT version()');
    console.log('CockroachDB version:', res.rows[0].version);
    client.release();
  } catch (err) {
    console.error('Error:', err);
  }
}

testConnection();
```

### SSL Configuration for Production

```javascript
const { Pool } = require('pg');
const fs = require('fs');

const pool = new Pool({
  user: 'maxroach',
  host: 'blue-dog-147.cockroachlabs.cloud',
  database: 'defaultdb',
  password: 'YourPassword',
  port: 26257,
  ssl: {
    rejectUnauthorized: true,
    ca: fs.readFileSync('/path/to/root.crt').toString(),
    cert: fs.readFileSync('/path/to/client.crt').toString(),
    key: fs.readFileSync('/path/to/client.key').toString(),
  },
});
```

### Connection String Format

```javascript
const { Pool } = require('pg');

// Local insecure cluster
const pool = new Pool({
  connectionString: 'postgresql://root@localhost:26257/defaultdb?sslmode=disable'
});

// Production secure cluster
const pool = new Pool({
  connectionString: 'postgresql://user:password@host.cockroachlabs.cloud:26257/database?sslmode=verify-full'
});

// With environment variable
const pool = new Pool({
  connectionString: process.env.DATABASE_URL
});
```

### Complete Initialization with Error Handling

```javascript
const { Pool } = require('pg');

const pool = new Pool({
  user: process.env.DB_USER || 'root',
  host: process.env.DB_HOST || 'localhost',
  database: process.env.DB_NAME || 'defaultdb',
  password: process.env.DB_PASSWORD || '',
  port: parseInt(process.env.DB_PORT) || 26257,
  max: 20,
  min: 5,
  idleTimeoutMillis: 300000,  // 5 minutes
  connectionTimeoutMillis: 5000,
});

pool.on('error', (err, client) => {
  console.error('Unexpected error on idle client', err);
  process.exit(-1);
});

pool.on('connect', () => {
  console.log('New client connected to CockroachDB pool');
});

pool.on('remove', () => {
  console.log('Client removed from pool');
});

async function initializeDatabase() {
  const client = await pool.connect();
  try {
    await client.query('SELECT 1');
    console.log('Database connection established');
  } catch (err) {
    console.error('Failed to connect to database:', err);
    throw err;
  } finally {
    client.release();
  }
}

initializeDatabase();
```

---

## Core API Operations

### Basic Queries

#### Simple SELECT Query

```javascript
const { Pool } = require('pg');
const pool = new Pool({ connectionString: process.env.DATABASE_URL });

async function getUsers() {
  const client = await pool.connect();
  try {
    const result = await client.query('SELECT * FROM users');
    console.log('Users:', result.rows);
    return result.rows;
  } catch (err) {
    console.error('Query error:', err);
    throw err;
  } finally {
    client.release();
  }
}

getUsers();
```

#### Parameterized Query

```javascript
async function getUserById(id) {
  const client = await pool.connect();
  try {
    const query = 'SELECT * FROM users WHERE id = $1';
    const result = await client.query(query, [id]);
    return result.rows[0];
  } catch (err) {
    console.error('Error fetching user:', err);
    throw err;
  } finally {
    client.release();
  }
}

getUserById('123e4567-e89b-12d3-a456-426614174000');
```

#### Multiple Parameters

```javascript
async function searchUsers(city, minAge) {
  const client = await pool.connect();
  try {
    const query = 'SELECT * FROM users WHERE city = $1 AND age >= $2';
    const result = await client.query(query, [city, minAge]);
    return result.rows;
  } finally {
    client.release();
  }
}

searchUsers('Seattle', 25);
```

### INSERT Operations

#### Single Row Insert

```javascript
async function createUser(name, email, city) {
  const client = await pool.connect();
  try {
    const query = `
      INSERT INTO users (name, email, city)
      VALUES ($1, $2, $3)
      RETURNING *
    `;
    const result = await client.query(query, [name, email, city]);
    console.log('Created user:', result.rows[0]);
    return result.rows[0];
  } finally {
    client.release();
  }
}

createUser('John Doe', 'john@example.com', 'New York');
```

#### Multiple Row Insert

```javascript
async function createMultipleUsers(users) {
  const client = await pool.connect();
  try {
    const query = `
      INSERT INTO users (name, email, city)
      VALUES
        ($1, $2, $3),
        ($4, $5, $6),
        ($7, $8, $9)
      RETURNING id, name
    `;
    const values = users.flatMap(u => [u.name, u.email, u.city]);
    const result = await client.query(query, values);
    return result.rows;
  } finally {
    client.release();
  }
}

createMultipleUsers([
  { name: 'Alice', email: 'alice@example.com', city: 'Boston' },
  { name: 'Bob', email: 'bob@example.com', city: 'Chicago' },
  { name: 'Carol', email: 'carol@example.com', city: 'Denver' }
]);
```

#### Insert with Default Values

```javascript
async function createDriver(city) {
  const client = await pool.connect();
  try {
    const query = `
      INSERT INTO drivers (id, city, created_at)
      VALUES (gen_random_uuid(), $1, now())
      RETURNING *
    `;
    const result = await client.query(query, [city]);
    return result.rows[0];
  } finally {
    client.release();
  }
}

createDriver('Seattle');
```

#### Insert with ON CONFLICT

```javascript
async function upsertUser(email, name, city) {
  const client = await pool.connect();
  try {
    const query = `
      INSERT INTO users (email, name, city)
      VALUES ($1, $2, $3)
      ON CONFLICT (email)
      DO UPDATE SET
        name = EXCLUDED.name,
        city = EXCLUDED.city,
        updated_at = now()
      RETURNING *
    `;
    const result = await client.query(query, [email, name, city]);
    return result.rows[0];
  } finally {
    client.release();
  }
}

upsertUser('john@example.com', 'John Smith', 'Los Angeles');
```

### UPDATE Operations

#### Simple Update

```javascript
async function updateUserCity(userId, newCity) {
  const client = await pool.connect();
  try {
    const query = `
      UPDATE users
      SET city = $1, updated_at = now()
      WHERE id = $2
      RETURNING *
    `;
    const result = await client.query(query, [newCity, userId]);
    return result.rows[0];
  } finally {
    client.release();
  }
}

updateUserCity('123e4567-e89b-12d3-a456-426614174000', 'Portland');
```

#### Conditional Update

```javascript
async function activateUser(email) {
  const client = await pool.connect();
  try {
    const query = `
      UPDATE users
      SET status = 'active', activated_at = now()
      WHERE email = $1 AND status = 'pending'
      RETURNING id, email, status
    `;
    const result = await client.query(query, [email]);
    if (result.rowCount === 0) {
      throw new Error('User not found or already active');
    }
    return result.rows[0];
  } finally {
    client.release();
  }
}

activateUser('john@example.com');
```

#### Bulk Update

```javascript
async function updateVehicleStatus(city, newStatus) {
  const client = await pool.connect();
  try {
    const query = `
      UPDATE vehicles
      SET status = $1
      WHERE city = $2
      RETURNING id, status
    `;
    const result = await client.query(query, [newStatus, city]);
    console.log(`Updated ${result.rowCount} vehicles`);
    return result.rows;
  } finally {
    client.release();
  }
}

updateVehicleStatus('New York', 'available');
```

### DELETE Operations

#### Simple Delete

```javascript
async function deleteUser(userId) {
  const client = await pool.connect();
  try {
    const query = 'DELETE FROM users WHERE id = $1 RETURNING *';
    const result = await client.query(query, [userId]);
    return result.rows[0];
  } finally {
    client.release();
  }
}

deleteUser('123e4567-e89b-12d3-a456-426614174000');
```

#### Conditional Delete

```javascript
async function deleteInactiveUsers(daysInactive) {
  const client = await pool.connect();
  try {
    const result = await client.query(
      `DELETE FROM users
       WHERE last_login < now() - make_interval(days => $1)
       RETURNING id, email`,
      [daysInactive]
    );
    console.log(`Deleted ${result.rowCount} inactive users`);
    return result.rows;
  } finally {
    client.release();
  }
}

deleteInactiveUsers(90);
```

#### Delete with JOIN

```javascript
async function deleteUserOrders(userId) {
  const client = await pool.connect();
  try {
    const query = `
      DELETE FROM orders
      WHERE user_id = $1
      RETURNING id, total_amount
    `;
    const result = await client.query(query, [userId]);
    return result.rows;
  } finally {
    client.release();
  }
}

deleteUserOrders('123e4567-e89b-12d3-a456-426614174000');
```

---

## Transactions

### Basic Transaction

```javascript
async function transferFunds(fromAccount, toAccount, amount) {
  const client = await pool.connect();
  try {
    await client.query('BEGIN');

    // Deduct from sender
    await client.query(
      'UPDATE accounts SET balance = balance - $1 WHERE id = $2',
      [amount, fromAccount]
    );

    // Add to receiver
    await client.query(
      'UPDATE accounts SET balance = balance + $1 WHERE id = $2',
      [amount, toAccount]
    );

    await client.query('COMMIT');
    console.log('Transfer completed successfully');
  } catch (err) {
    await client.query('ROLLBACK');
    console.error('Transfer failed, rolled back:', err);
    throw err;
  } finally {
    client.release();
  }
}

transferFunds('account-1', 'account-2', 100.50);
```

### Transaction with Savepoints

```javascript
async function complexTransaction() {
  const client = await pool.connect();
  try {
    await client.query('BEGIN');

    // First operation
    await client.query('INSERT INTO logs (message) VALUES ($1)', ['Started']);

    // Create savepoint
    await client.query('SAVEPOINT sp1');

    try {
      await client.query('INSERT INTO users (email) VALUES ($1)', ['test@example.com']);
    } catch (err) {
      // Rollback to savepoint on error
      await client.query('ROLLBACK TO SAVEPOINT sp1');
      console.log('User insert failed, rolled back to savepoint');
    }

    // Continue with transaction
    await client.query('INSERT INTO logs (message) VALUES ($1)', ['Completed']);

    await client.query('COMMIT');
  } catch (err) {
    await client.query('ROLLBACK');
    throw err;
  } finally {
    client.release();
  }
}

complexTransaction();
```

### Transaction with Retry Logic (CockroachDB Specific)

```javascript
async function transferWithRetry(fromAccount, toAccount, amount, maxRetries = 3) {
  let retries = 0;

  while (retries < maxRetries) {
    const client = await pool.connect();
    try {
      await client.query('BEGIN');

      const fromResult = await client.query(
        'UPDATE accounts SET balance = balance - $1 WHERE id = $2 RETURNING balance',
        [amount, fromAccount]
      );

      if (fromResult.rows[0].balance < 0) {
        throw new Error('Insufficient funds');
      }

      await client.query(
        'UPDATE accounts SET balance = balance + $1 WHERE id = $2',
        [amount, toAccount]
      );

      await client.query('COMMIT');
      client.release();
      return { success: true, retries };

    } catch (err) {
      await client.query('ROLLBACK');
      client.release();

      // Check if it's a serialization error (40001)
      if (err.code === '40001' && retries < maxRetries - 1) {
        retries++;
        console.log(`Retrying transaction (attempt ${retries + 1})`);
        await new Promise(resolve => setTimeout(resolve, Math.random() * 100));
        continue;
      }

      throw err;
    }
  }

  throw new Error('Transaction failed after maximum retries');
}

transferWithRetry('account-1', 'account-2', 100.50);
```

### Read-Only Transaction

```javascript
async function getAccountSummary(userId) {
  const client = await pool.connect();
  try {
    await client.query('BEGIN TRANSACTION READ ONLY');

    const userResult = await client.query(
      'SELECT * FROM users WHERE id = $1',
      [userId]
    );

    const accountsResult = await client.query(
      'SELECT * FROM accounts WHERE user_id = $1',
      [userId]
    );

    const transactionsResult = await client.query(
      'SELECT * FROM transactions WHERE user_id = $1 ORDER BY created_at DESC LIMIT 10',
      [userId]
    );

    await client.query('COMMIT');

    return {
      user: userResult.rows[0],
      accounts: accountsResult.rows,
      recentTransactions: transactionsResult.rows
    };
  } catch (err) {
    await client.query('ROLLBACK');
    throw err;
  } finally {
    client.release();
  }
}

getAccountSummary('user-123');
```

---

## Advanced Features

### JSON/JSONB Operations

#### Inserting JSON Data

```javascript
async function createProduct(name, metadata) {
  const client = await pool.connect();
  try {
    const query = `
      INSERT INTO products (name, metadata)
      VALUES ($1, $2::jsonb)
      RETURNING *
    `;
    const result = await client.query(query, [name, JSON.stringify(metadata)]);
    return result.rows[0];
  } finally {
    client.release();
  }
}

createProduct('Laptop', {
  brand: 'Dell',
  specs: { ram: '16GB', storage: '512GB SSD' },
  tags: ['electronics', 'computers']
});
```

#### Querying JSON Fields

```javascript
async function findProductsByBrand(brand) {
  const client = await pool.connect();
  try {
    const query = `
      SELECT * FROM products
      WHERE metadata->>'brand' = $1
    `;
    const result = await client.query(query, [brand]);
    return result.rows;
  } finally {
    client.release();
  }
}

findProductsByBrand('Dell');
```

#### Nested JSON Queries

```javascript
async function findProductsByRAM(ram) {
  const client = await pool.connect();
  try {
    const query = `
      SELECT * FROM products
      WHERE metadata->'specs'->>'ram' = $1
    `;
    const result = await client.query(query, [ram]);
    return result.rows;
  } finally {
    client.release();
  }
}

findProductsByRAM('16GB');
```

#### Updating JSON Fields

```javascript
async function updateProductPrice(productId, newPrice) {
  const client = await pool.connect();
  try {
    const query = `
      UPDATE products
      SET metadata = jsonb_set(metadata, '{price}', $1::jsonb)
      WHERE id = $2
      RETURNING *
    `;
    const result = await client.query(query, [JSON.stringify(newPrice), productId]);
    return result.rows[0];
  } finally {
    client.release();
  }
}

updateProductPrice('product-123', 999.99);
```

#### JSON Array Operations

```javascript
async function addProductTag(productId, tag) {
  const client = await pool.connect();
  try {
    const query = `
      UPDATE products
      SET metadata = jsonb_set(
        metadata,
        '{tags}',
        (metadata->'tags')::jsonb || $1::jsonb
      )
      WHERE id = $2
      RETURNING *
    `;
    const result = await client.query(query, [JSON.stringify([tag]), productId]);
    return result.rows[0];
  } finally {
    client.release();
  }
}

addProductTag('product-123', 'featured');
```

### Array Operations

#### Working with Arrays

```javascript
async function createUserWithTags(email, tags) {
  const client = await pool.connect();
  try {
    const query = `
      INSERT INTO users (email, tags)
      VALUES ($1, $2)
      RETURNING *
    `;
    const result = await client.query(query, [email, tags]);
    return result.rows[0];
  } finally {
    client.release();
  }
}

createUserWithTags('john@example.com', ['premium', 'verified']);
```

#### Querying Arrays

```javascript
async function findUsersByTag(tag) {
  const client = await pool.connect();
  try {
    const query = `
      SELECT * FROM users
      WHERE $1 = ANY(tags)
    `;
    const result = await client.query(query, [tag]);
    return result.rows;
  } finally {
    client.release();
  }
}

findUsersByTag('premium');
```

#### Array Contains Query

```javascript
async function findUsersWithAllTags(requiredTags) {
  const client = await pool.connect();
  try {
    const query = `
      SELECT * FROM users
      WHERE tags @> $1::text[]
    `;
    const result = await client.query(query, [requiredTags]);
    return result.rows;
  } finally {
    client.release();
  }
}

findUsersWithAllTags(['premium', 'verified']);
```

### Full-Text Search

#### Creating a Text Search Query

```javascript
async function searchArticles(searchTerm) {
  const client = await pool.connect();
  try {
    const query = `
      SELECT id, title, content,
             ts_rank(to_tsvector('english', title || ' ' || content), query) AS rank
      FROM articles,
           to_tsquery('english', $1) query
      WHERE to_tsvector('english', title || ' ' || content) @@ query
      ORDER BY rank DESC
      LIMIT 20
    `;
    const result = await client.query(query, [searchTerm]);
    return result.rows;
  } finally {
    client.release();
  }
}

searchArticles('database performance');
```

### Aggregations and Analytics

#### Group By and Aggregation

```javascript
async function getUserStatsByCity() {
  const client = await pool.connect();
  try {
    const query = `
      SELECT
        city,
        COUNT(*) as user_count,
        AVG(age) as avg_age,
        MIN(created_at) as first_user,
        MAX(created_at) as latest_user
      FROM users
      GROUP BY city
      ORDER BY user_count DESC
    `;
    const result = await client.query(query);
    return result.rows;
  } finally {
    client.release();
  }
}

getUserStatsByCity();
```

#### Window Functions

```javascript
async function getRankedProducts() {
  const client = await pool.connect();
  try {
    const query = `
      SELECT
        name,
        category,
        price,
        RANK() OVER (PARTITION BY category ORDER BY price DESC) as price_rank,
        AVG(price) OVER (PARTITION BY category) as category_avg_price
      FROM products
    `;
    const result = await client.query(query);
    return result.rows;
  } finally {
    client.release();
  }
}

getRankedProducts();
```

### Common Table Expressions (CTEs)

```javascript
async function getTopSpenders(limit = 10) {
  const client = await pool.connect();
  try {
    const query = `
      WITH user_totals AS (
        SELECT
          user_id,
          SUM(amount) as total_spent,
          COUNT(*) as order_count
        FROM orders
        WHERE status = 'completed'
        GROUP BY user_id
      )
      SELECT
        u.id,
        u.email,
        ut.total_spent,
        ut.order_count
      FROM users u
      JOIN user_totals ut ON u.id = ut.user_id
      ORDER BY ut.total_spent DESC
      LIMIT $1
    `;
    const result = await client.query(query, [limit]);
    return result.rows;
  } finally {
    client.release();
  }
}

getTopSpenders(20);
```

### Batch Operations

#### Batch Insert

```javascript
async function batchInsertUsers(users) {
  const client = await pool.connect();
  try {
    await client.query('BEGIN');

    for (const user of users) {
      await client.query(
        'INSERT INTO users (name, email, city) VALUES ($1, $2, $3)',
        [user.name, user.email, user.city]
      );
    }

    await client.query('COMMIT');
    console.log(`Inserted ${users.length} users`);
  } catch (err) {
    await client.query('ROLLBACK');
    throw err;
  } finally {
    client.release();
  }
}

batchInsertUsers([
  { name: 'Alice', email: 'alice@example.com', city: 'NYC' },
  { name: 'Bob', email: 'bob@example.com', city: 'LA' }
]);
```

---

## Schema Management

### Creating Tables

```javascript
async function createUsersTable() {
  const client = await pool.connect();
  try {
    const query = `
      CREATE TABLE IF NOT EXISTS users (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        email STRING UNIQUE NOT NULL,
        name STRING NOT NULL,
        city STRING,
        age INT,
        tags TEXT[],
        metadata JSONB,
        created_at TIMESTAMP DEFAULT now(),
        updated_at TIMESTAMP DEFAULT now()
      )
    `;
    await client.query(query);
    console.log('Users table created');
  } finally {
    client.release();
  }
}

createUsersTable();
```

### Creating Indexes

```javascript
async function createIndexes() {
  const client = await pool.connect();
  try {
    // Standard index
    await client.query('CREATE INDEX IF NOT EXISTS idx_users_city ON users (city)');

    // Multi-column index
    await client.query('CREATE INDEX IF NOT EXISTS idx_users_city_age ON users (city, age)');

    // Inverted index for JSONB
    await client.query('CREATE INVERTED INDEX IF NOT EXISTS idx_users_metadata ON users (metadata)');

    // Inverted index for arrays
    await client.query('CREATE INVERTED INDEX IF NOT EXISTS idx_users_tags ON users (tags)');

    console.log('Indexes created');
  } finally {
    client.release();
  }
}

createIndexes();
```

### Altering Tables

```javascript
async function alterUsersTable() {
  const client = await pool.connect();
  try {
    // Add column
    await client.query('ALTER TABLE users ADD COLUMN IF NOT EXISTS status STRING DEFAULT \'active\'');

    // Add constraint
    await client.query('ALTER TABLE users ADD CONSTRAINT check_age CHECK (age >= 0 AND age <= 150)');

    console.log('Table altered successfully');
  } finally {
    client.release();
  }
}

alterUsersTable();
```

---

## Error Handling

### Handling Specific Errors

```javascript
async function createUserWithErrorHandling(email, name) {
  const client = await pool.connect();
  try {
    const query = 'INSERT INTO users (email, name) VALUES ($1, $2) RETURNING *';
    const result = await client.query(query, [email, name]);
    return result.rows[0];
  } catch (err) {
    if (err.code === '23505') {
      // Unique violation
      throw new Error(`User with email ${email} already exists`);
    } else if (err.code === '23502') {
      // Not null violation
      throw new Error('Required field is missing');
    } else if (err.code === '23503') {
      // Foreign key violation
      throw new Error('Referenced record does not exist');
    } else if (err.code === '40001') {
      // Serialization failure
      throw new Error('Transaction conflict, please retry');
    } else {
      throw err;
    }
  } finally {
    client.release();
  }
}

createUserWithErrorHandling('john@example.com', 'John Doe');
```

### Connection Error Handling

```javascript
async function queryWithRetry(query, params, maxRetries = 3) {
  let retries = 0;

  while (retries < maxRetries) {
    let client;
    try {
      client = await pool.connect();
      const result = await client.query(query, params);
      client.release();
      return result;
    } catch (err) {
      if (client) client.release();

      if (err.code === 'ECONNREFUSED' || err.code === 'ETIMEDOUT') {
        retries++;
        if (retries >= maxRetries) {
          throw new Error('Database connection failed after retries');
        }
        await new Promise(resolve => setTimeout(resolve, 1000 * retries));
      } else {
        throw err;
      }
    }
  }
}

queryWithRetry('SELECT * FROM users WHERE id = $1', ['user-123']);
```

---

## Closing Connections

### Graceful Shutdown

```javascript
async function gracefulShutdown() {
  console.log('Closing database connections...');
  await pool.end();
  console.log('Database connections closed');
  process.exit(0);
}

process.on('SIGTERM', gracefulShutdown);
process.on('SIGINT', gracefulShutdown);
```

### Complete Application Example

```javascript
const { Pool } = require('pg');

const pool = new Pool({
  user: process.env.DB_USER || 'root',
  host: process.env.DB_HOST || 'localhost',
  database: process.env.DB_NAME || 'defaultdb',
  port: parseInt(process.env.DB_PORT) || 26257,
  max: 20,
});

// Query functions
async function getAllUsers() {
  const client = await pool.connect();
  try {
    const result = await client.query('SELECT * FROM users');
    return result.rows;
  } finally {
    client.release();
  }
}

async function createUser(name, email) {
  const client = await pool.connect();
  try {
    const query = 'INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *';
    const result = await client.query(query, [name, email]);
    return result.rows[0];
  } finally {
    client.release();
  }
}

// Graceful shutdown
async function shutdown() {
  await pool.end();
  process.exit(0);
}

process.on('SIGTERM', shutdown);
process.on('SIGINT', shutdown);

// Export for use in application
module.exports = {
  pool,
  getAllUsers,
  createUser,
};
```

---

## Common Patterns

### Repository Pattern

```javascript
class UserRepository {
  constructor(pool) {
    this.pool = pool;
  }

  async findAll() {
    const client = await this.pool.connect();
    try {
      const result = await client.query('SELECT * FROM users');
      return result.rows;
    } finally {
      client.release();
    }
  }

  async findById(id) {
    const client = await this.pool.connect();
    try {
      const result = await client.query('SELECT * FROM users WHERE id = $1', [id]);
      return result.rows[0];
    } finally {
      client.release();
    }
  }

  async create(user) {
    const client = await this.pool.connect();
    try {
      const query = `
        INSERT INTO users (name, email, city)
        VALUES ($1, $2, $3)
        RETURNING *
      `;
      const result = await client.query(query, [user.name, user.email, user.city]);
      return result.rows[0];
    } finally {
      client.release();
    }
  }

  async update(id, updates) {
    const client = await this.pool.connect();
    try {
      const query = `
        UPDATE users
        SET name = $1, city = $2, updated_at = now()
        WHERE id = $3
        RETURNING *
      `;
      const result = await client.query(query, [updates.name, updates.city, id]);
      return result.rows[0];
    } finally {
      client.release();
    }
  }

  async delete(id) {
    const client = await this.pool.connect();
    try {
      const result = await client.query('DELETE FROM users WHERE id = $1 RETURNING *', [id]);
      return result.rows[0];
    } finally {
      client.release();
    }
  }
}

const userRepo = new UserRepository(pool);
```

### Query Builder Pattern

```javascript
class QueryBuilder {
  constructor(pool, table) {
    this.pool = pool;
    this.table = table;
    this.whereConditions = [];
    this.parameters = [];
    this.orderByClause = '';
    this.limitClause = '';
  }

  where(column, operator, value) {
    this.parameters.push(value);
    this.whereConditions.push(`${column} ${operator} $${this.parameters.length}`);
    return this;
  }

  orderBy(column, direction = 'ASC') {
    this.orderByClause = `ORDER BY ${column} ${direction}`;
    return this;
  }

  limit(count) {
    this.limitClause = `LIMIT ${count}`;
    return this;
  }

  async execute() {
    const client = await this.pool.connect();
    try {
      let query = `SELECT * FROM ${this.table}`;

      if (this.whereConditions.length > 0) {
        query += ` WHERE ${this.whereConditions.join(' AND ')}`;
      }

      if (this.orderByClause) {
        query += ` ${this.orderByClause}`;
      }

      if (this.limitClause) {
        query += ` ${this.limitClause}`;
      }

      const result = await client.query(query, this.parameters);
      return result.rows;
    } finally {
      client.release();
    }
  }
}

// Usage
const users = await new QueryBuilder(pool, 'users')
  .where('city', '=', 'Seattle')
  .where('age', '>=', 25)
  .orderBy('created_at', 'DESC')
  .limit(10)
  .execute();
```

---

## Performance Optimization

### Connection Pooling Configuration

```javascript
const { Pool } = require('pg');

const pool = new Pool({
  user: 'root',
  host: 'localhost',
  database: 'defaultdb',
  port: 26257,
  max: 20,                      // Maximum number of clients in the pool
  min: 5,                       // Minimum number of clients in the pool
  idleTimeoutMillis: 300000,    // Close idle clients after 5 minutes (CockroachDB recommendation)
  connectionTimeoutMillis: 5000,
  allowExitOnIdle: false,
});
```

### Prepared Statements

```javascript
async function findUsersPrepared(city) {
  const client = await pool.connect();
  try {
    // Named prepared statement
    const queryName = 'find-users-by-city';
    const queryText = 'SELECT * FROM users WHERE city = $1';

    const result = await client.query({
      name: queryName,
      text: queryText,
      values: [city]
    });

    return result.rows;
  } finally {
    client.release();
  }
}

findUsersPrepared('Seattle');
```

### Cursor-Based Pagination

```javascript
async function paginateUsers(cursorId, pageSize = 20) {
  const client = await pool.connect();
  try {
    const query = `
      SELECT * FROM users
      WHERE id > $1
      ORDER BY id
      LIMIT $2
    `;
    const result = await client.query(query, [cursorId || '00000000-0000-0000-0000-000000000000', pageSize]);

    return {
      data: result.rows,
      nextCursor: result.rows.length > 0 ? result.rows[result.rows.length - 1].id : null,
      hasMore: result.rows.length === pageSize
    };
  } finally {
    client.release();
  }
}

paginateUsers(null, 20);
```
