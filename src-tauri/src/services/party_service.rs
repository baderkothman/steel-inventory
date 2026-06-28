use rusqlite::{params, Connection};

use crate::{
    models::{DateRangeFilters, PartyFilters, PartyPayload, PartyRow, StatementRow},
    utils::{
        audit::insert_audit_log,
        dates::{now_iso, validate_date},
        errors::AppError,
        validation::{non_negative_i64, required},
    },
};

#[derive(Debug, Clone, Copy)]
pub enum PartyKind {
    Supplier,
    Customer,
}

impl PartyKind {
    fn table(self) -> &'static str {
        match self {
            PartyKind::Supplier => "suppliers",
            PartyKind::Customer => "customers",
        }
    }

    fn invoice_table(self) -> &'static str {
        match self {
            PartyKind::Supplier => "purchase_invoices",
            PartyKind::Customer => "sales_invoices",
        }
    }

    fn invoice_party_column(self) -> &'static str {
        match self {
            PartyKind::Supplier => "supplier_id",
            PartyKind::Customer => "customer_id",
        }
    }

    fn invoice_date_column(self) -> &'static str {
        match self {
            PartyKind::Supplier => "invoice_date",
            PartyKind::Customer => "invoice_date",
        }
    }

    fn invoice_status_filter(self) -> &'static str {
        match self {
            PartyKind::Supplier => "status = 'active'",
            PartyKind::Customer => "sales_status = 'completed'",
        }
    }

    fn payment_party_type(self) -> &'static str {
        match self {
            PartyKind::Supplier => "supplier",
            PartyKind::Customer => "customer",
        }
    }

    fn payment_direction(self) -> &'static str {
        match self {
            PartyKind::Supplier => "out",
            PartyKind::Customer => "in",
        }
    }

    fn invoice_type_label(self) -> &'static str {
        match self {
            PartyKind::Supplier => "Purchase Invoice",
            PartyKind::Customer => "Sales Invoice",
        }
    }
}

pub fn list_parties(
    conn: &Connection,
    kind: PartyKind,
    filters: PartyFilters,
) -> Result<Vec<PartyRow>, AppError> {
    let search = filters
        .search
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let active_only = filters.active_only.unwrap_or(false);
    let sql = format!(
        "SELECT id, name, company_name, phone, email, address, tax_number,
                opening_balance_cents, notes, is_active, created_at, updated_at
         FROM {}
         WHERE (?1 IS NULL OR (
             name LIKE '%' || ?1 || '%' OR
             company_name LIKE '%' || ?1 || '%' OR
             phone LIKE '%' || ?1 || '%' OR
             email LIKE '%' || ?1 || '%'
         ))
           AND (?2 = 0 OR is_active = 1)
         ORDER BY name",
        kind.table()
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params![search, if active_only { 1 } else { 0 }], base_party_from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    rows.into_iter()
        .map(|party_result| {
            let mut party = party_result;
            party.balance_cents = party_balance(conn, kind, party.id)?;
            Ok(party)
        })
        .collect()
}

pub fn get_party(conn: &Connection, kind: PartyKind, id: i64) -> Result<PartyRow, AppError> {
    let sql = format!(
        "SELECT id, name, company_name, phone, email, address, tax_number,
                opening_balance_cents, notes, is_active, created_at, updated_at
         FROM {}
         WHERE id = ?1",
        kind.table()
    );
    let mut party = conn
        .query_row(&sql, [id], base_party_from_row)
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => AppError::not_found("Record not found."),
            other => other.into(),
        })?;
    party.balance_cents = party_balance(conn, kind, id)?;
    Ok(party)
}

pub fn create_party(
    conn: &Connection,
    user_id: i64,
    kind: PartyKind,
    payload: PartyPayload,
) -> Result<PartyRow, AppError> {
    validate_party_payload(&payload)?;
    let now = now_iso();
    let sql = format!(
        "INSERT INTO {}
         (name, company_name, phone, email, address, tax_number, opening_balance_cents,
          notes, is_active, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9, ?9)",
        kind.table()
    );
    conn.execute(
        &sql,
        params![
            payload.name.trim(),
            payload.company_name,
            payload.phone,
            payload.email,
            payload.address,
            payload.tax_number,
            payload.opening_balance_cents,
            payload.notes,
            now
        ],
    )?;
    let id = conn.last_insert_rowid();
    let party = get_party(conn, kind, id)?;
    insert_audit_log(
        conn,
        user_id,
        "create",
        kind.table(),
        id,
        None,
        Some(serde_json::to_value(&party).unwrap_or_default()),
    )?;
    Ok(party)
}

pub fn update_party(
    conn: &Connection,
    user_id: i64,
    kind: PartyKind,
    id: i64,
    payload: PartyPayload,
) -> Result<PartyRow, AppError> {
    validate_party_payload(&payload)?;
    ensure_party_exists(conn, kind, id)?;
    let now = now_iso();
    let sql = format!(
        "UPDATE {}
         SET name = ?1, company_name = ?2, phone = ?3, email = ?4, address = ?5,
             tax_number = ?6, opening_balance_cents = ?7, notes = ?8, updated_at = ?9
         WHERE id = ?10",
        kind.table()
    );
    conn.execute(
        &sql,
        params![
            payload.name.trim(),
            payload.company_name,
            payload.phone,
            payload.email,
            payload.address,
            payload.tax_number,
            payload.opening_balance_cents,
            payload.notes,
            now,
            id
        ],
    )?;
    let party = get_party(conn, kind, id)?;
    insert_audit_log(
        conn,
        user_id,
        "update",
        kind.table(),
        id,
        None,
        Some(serde_json::to_value(&party).unwrap_or_default()),
    )?;
    Ok(party)
}

pub fn archive_party(
    conn: &Connection,
    user_id: i64,
    kind: PartyKind,
    id: i64,
) -> Result<(), AppError> {
    ensure_party_exists(conn, kind, id)?;
    let sql = format!("UPDATE {} SET is_active = 0, updated_at = ?1 WHERE id = ?2", kind.table());
    conn.execute(&sql, params![now_iso(), id])?;
    insert_audit_log(conn, user_id, "archive", kind.table(), id, None, None)?;
    Ok(())
}

pub fn party_balance(conn: &Connection, kind: PartyKind, id: i64) -> Result<i64, AppError> {
    let invoice_sql = format!(
        "SELECT COALESCE(SUM(total_cents), 0) FROM {} WHERE {} = ?1 AND {}",
        kind.invoice_table(),
        kind.invoice_party_column(),
        kind.invoice_status_filter()
    );
    let payment_sql = "SELECT COALESCE(SUM(amount_cents), 0)
                       FROM payments
                       WHERE party_type = ?1 AND party_id = ?2 AND payment_direction = ?3";
    let opening_sql = format!("SELECT opening_balance_cents FROM {} WHERE id = ?1", kind.table());

    let opening: i64 = conn.query_row(&opening_sql, [id], |row| row.get(0))?;
    let invoice_total: i64 = conn.query_row(&invoice_sql, [id], |row| row.get(0))?;
    let payment_total: i64 = conn.query_row(
        payment_sql,
        params![kind.payment_party_type(), id, kind.payment_direction()],
        |row| row.get(0),
    )?;
    Ok(opening + invoice_total - payment_total)
}

pub fn statement(
    conn: &Connection,
    kind: PartyKind,
    id: i64,
    filters: DateRangeFilters,
) -> Result<Vec<StatementRow>, AppError> {
    ensure_party_exists(conn, kind, id)?;
    if let Some(date) = filters.date_from.as_deref() {
        validate_date(date, "Start date")?;
    }
    if let Some(date) = filters.date_to.as_deref() {
        validate_date(date, "End date")?;
    }

    let opening_sql = format!("SELECT opening_balance_cents FROM {} WHERE id = ?1", kind.table());
    let mut running_balance: i64 = conn.query_row(&opening_sql, [id], |row| row.get(0))?;

    if let Some(date_from) = filters.date_from.as_deref() {
        running_balance += prior_invoice_total(conn, kind, id, date_from)?;
        running_balance -= prior_payment_total(conn, kind, id, date_from)?;
    }

    let mut rows = vec![StatementRow {
        date: filters.date_from.clone().unwrap_or_default(),
        entry_type: "Opening Balance".to_string(),
        reference: "Opening".to_string(),
        debit_cents: running_balance.max(0),
        credit_cents: if running_balance < 0 { running_balance.abs() } else { 0 },
        balance_cents: running_balance,
    }];

    let invoice_sql = format!(
        "SELECT {date_col}, invoice_number, total_cents
         FROM {invoice_table}
         WHERE {party_col} = ?1
           AND {status_filter}
           AND (?2 IS NULL OR date({date_col}) >= date(?2))
           AND (?3 IS NULL OR date({date_col}) <= date(?3))
         ORDER BY {date_col}, id",
        date_col = kind.invoice_date_column(),
        invoice_table = kind.invoice_table(),
        party_col = kind.invoice_party_column(),
        status_filter = kind.invoice_status_filter()
    );
    let mut invoice_stmt = conn.prepare(&invoice_sql)?;
    let invoice_rows = invoice_stmt
        .query_map(params![id, filters.date_from, filters.date_to], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut payment_stmt = conn.prepare(
        "SELECT payment_date, id, amount_cents
         FROM payments
         WHERE party_type = ?1 AND party_id = ?2 AND payment_direction = ?3
           AND (?4 IS NULL OR date(payment_date) >= date(?4))
           AND (?5 IS NULL OR date(payment_date) <= date(?5))
         ORDER BY payment_date, id",
    )?;
    let payment_rows = payment_stmt
        .query_map(
            params![
                kind.payment_party_type(),
                id,
                kind.payment_direction(),
                filters.date_from,
                filters.date_to
            ],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;

    let mut entries: Vec<(String, String, String, i64, i64)> = Vec::new();
    entries.extend(invoice_rows.into_iter().map(|(date, number, total)| {
        (
            date,
            kind.invoice_type_label().to_string(),
            number,
            total,
            0,
        )
    }));
    entries.extend(payment_rows.into_iter().map(|(date, id, amount)| {
        (
            date,
            "Payment".to_string(),
            format!("PAY-{id:05}"),
            0,
            amount,
        )
    }));
    entries.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    for (date, entry_type, reference, debit, credit) in entries {
        running_balance += debit - credit;
        rows.push(StatementRow {
            date,
            entry_type,
            reference,
            debit_cents: debit,
            credit_cents: credit,
            balance_cents: running_balance,
        });
    }
    Ok(rows)
}

fn prior_invoice_total(
    conn: &Connection,
    kind: PartyKind,
    id: i64,
    date_from: &str,
) -> Result<i64, AppError> {
    let sql = format!(
        "SELECT COALESCE(SUM(total_cents), 0)
         FROM {}
         WHERE {} = ?1 AND {} AND date({}) < date(?2)",
        kind.invoice_table(),
        kind.invoice_party_column(),
        kind.invoice_status_filter(),
        kind.invoice_date_column()
    );
    conn.query_row(&sql, params![id, date_from], |row| row.get(0))
        .map_err(Into::into)
}

fn prior_payment_total(
    conn: &Connection,
    kind: PartyKind,
    id: i64,
    date_from: &str,
) -> Result<i64, AppError> {
    conn.query_row(
        "SELECT COALESCE(SUM(amount_cents), 0)
         FROM payments
         WHERE party_type = ?1 AND party_id = ?2 AND payment_direction = ?3
           AND date(payment_date) < date(?4)",
        params![kind.payment_party_type(), id, kind.payment_direction(), date_from],
        |row| row.get(0),
    )
    .map_err(Into::into)
}

fn validate_party_payload(payload: &PartyPayload) -> Result<(), AppError> {
    required(&payload.name, "Name")?;
    non_negative_i64(payload.opening_balance_cents, "Opening balance")?;
    Ok(())
}

fn ensure_party_exists(conn: &Connection, kind: PartyKind, id: i64) -> Result<(), AppError> {
    let sql = format!("SELECT COUNT(*) FROM {} WHERE id = ?1", kind.table());
    let count: i64 = conn.query_row(&sql, [id], |row| row.get(0))?;
    if count == 0 {
        Err(AppError::not_found("Record not found."))
    } else {
        Ok(())
    }
}

fn base_party_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PartyRow> {
    Ok(PartyRow {
        id: row.get(0)?,
        name: row.get(1)?,
        company_name: row.get(2)?,
        phone: row.get(3)?,
        email: row.get(4)?,
        address: row.get(5)?,
        tax_number: row.get(6)?,
        opening_balance_cents: row.get(7)?,
        notes: row.get(8)?,
        is_active: row.get::<_, i64>(9)? == 1,
        balance_cents: 0,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}
