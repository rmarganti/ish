# Creating Issues

Guidelines for writing effective issues in ish — for humans and AI agents alike.

## Principles

1. **Actionable** — Every issue should describe a clear, completable unit of work.
2. **Testable** — It should be possible to verify when the issue is done.
3. **Self-contained** — Between the issue's own fields and its ancestor context,
   all information needed to complete the work should be present.

## Issue hierarchy

Break work down only as far as necessary.

| Level      | Use for                                  | Example                               |
| ---------- | ---------------------------------------- | ------------------------------------- |
| Top-level  | Features, bug fixes, epics               | "Add user authentication"             |
| Child      | Tasks within a feature                   | "Implement JWT validation middleware" |
| Grandchild | Sub-tasks when a task is still too large | "Write token expiry unit tests"       |

A single top-level issue with no children is perfectly fine for small,
self-contained work. Only introduce children when the parent is too large
to complete in one pass or needs to be worked on incrementally.

### Leaf issues are the work

Parent issues that have children are **not directly worked on** — `next`
skips issues with incomplete children. The parent acts as a container;
the children are the actionable units.

## Fields

### `title`

A short, imperative description of the change.

- ✓ `Add rate limiting to /api/upload`
- ✗ `Rate limiting` (too vague)
- ✗ `We should probably add rate limiting to the upload endpoint` (not imperative)

### `body` — what to do

The body describes **the work itself**: what should change, acceptance criteria,
and expected behavior.

```bash
ish add "Add rate limiting to /api/upload" \
  --body "Limit to 10 req/min per API key. Return 429 with Retry-After header."
```

### `context` — reference material to do it

Context carries **all information needed to complete the work**: domain
decisions, relevant files and specific changes to make in them, architectural
constraints, API contracts, error handling expectations, and any other
reference material. Context can and should be thorough — include everything
the implementer needs so they don't have to go searching for it.

Context might include:

- **File paths and what to change in them**
- **Domain rules and business logic decisions**
- **API contracts and data shapes**
- **Error handling expectations**
- **Relationships to other parts of the system**
- **Examples of existing patterns to follow**

A short context is fine when the task is simple:

```bash
ish add "Fix typo in README" \
  --context "Line 12 of README.md: 'recieve' → 'receive'."
```

But don't hesitate to include substantial detail when the task demands it:

```bash
ish add "Add rate limiting to upload endpoint" \
  --body "Limit /api/upload to 10 req/min per API key. Return 429 with Retry-After header." \
  --context "Implementation:
- Add a RateLimiter struct in src/middleware/rate_limit.rs following the pattern
  in src/middleware/auth.rs (implement Tower Service trait).
- Config: add rate_limit_rpm field to ApiConfig in src/config.rs (default 10).
  This must be loaded from the RATE_LIMIT_RPM env var.
- Storage: use the existing Redis connection pool in src/db/redis.rs.
  Key format: rate:{api_key}:{endpoint}. Use INCR + EXPIRE (60s TTL).
- Error response: return 429 with JSON body { \"error\": \"rate_limit_exceeded\" }
  and Retry-After header (seconds until window resets). Follow the error
  format in src/errors.rs (ApiError enum).
- Wire it up in src/routes/upload.rs — apply after auth middleware so we
  have the API key available.
- Do NOT apply to /api/health or /api/status."
```

#### Context is inherited

When viewing an issue (`show`, `next`, `start`), ancestor context is
automatically included. This is a key design feature — put shared context
on the parent and only task-specific context on children.

```bash
# Parent: broad context shared by all children
ish add "Add user authentication" \
  --context "Architecture decisions:
- Using argon2 for password hashing (already in Cargo.toml).
- Auth module lives in src/auth/. See src/auth/mod.rs for the module structure.
- JWTs signed with ES256. Secret loaded from JWT_SECRET env var.
- Token lifetime: 15 min access, 7 day refresh.
- All auth errors return 401 with { \"error\": \"unauthorized\" } — do not
  leak whether a user exists.
- User model: see src/models/user.rs (id, email, password_hash, created_at)."

# Child: only what's specific to THIS task
ish add "Add password hashing utility" \
  --parent abc123 \
  --body "Create hash_password() and verify_password() functions." \
  --context "Put in src/auth/hash.rs and re-export from src/auth/mod.rs.
hash_password(plain: &str) -> Result<String> — returns PHC-format string.
verify_password(plain: &str, hash: &str) -> Result<bool>.
Use argon2::default config. See tests in src/auth/hash_test.rs for expected interface."
```

When an agent runs `ish show` or `ish start` on the child, it will see
both its own context and the parent's architecture decisions — no
repetition needed.

## Sizing and splitting

**Start with a single issue.** Split only when:

- The work spans multiple unrelated files or systems.
- Steps must be completed in a specific order.
- The issue is too large to hold in working memory (human or agent).

**When splitting**, ensure each child is independently actionable and testable.
Use `--sort` to control execution order when it matters.

```bash
ish add "Migrate database schema" --parent abc123 --sort 1 \
  --body "Run the migration in db/migrations/003.sql."

ish add "Update ORM models" --parent abc123 --sort 2 \
  --body "Regenerate models to match new schema."
```
