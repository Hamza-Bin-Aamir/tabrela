if ! sqlx migrate run; then
    echo "Failed to apply migrations" >&2
    exit 1
fi
echo "Migrations applied successfully"
# We tell cargo that these files were updated
touch ./auth/src/database.rs
touch ./attendance/src/database.rs
touch ./merit/src/database.rs
touch ./tabulation/src/database.rs
# We build the services to apply the new migrations (sqlx bakes migrations at compile time)
cd attendance
if ! cargo build; then
    echo "attendance: cargo build failed" >&2
    exit 1
fi
echo "attendance: build succeeded"
cd ../auth
if ! cargo build; then
    echo "auth: cargo build failed" >&2
    exit 1
fi
echo "auth: build succeeded"
cd ../merit
if ! cargo build; then
    echo "merit: cargo build failed" >&2
    exit 1
fi
echo "merit: build succeeded"
cd ../tabulation
if ! cargo build; then
    echo "tabulation: cargo build failed" >&2
    exit 1
fi
echo "tabulation: build succeeded"