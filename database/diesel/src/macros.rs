/// Runs a given query on a given connection with a given function.
/// Returns `DbResult<T>`
#[macro_export]
macro_rules! exec {
    ($query:expr, $conn:expr, $func:ident) => {{
        $query
            .$func(&$conn)
            .map_err(DieselError::DieselError)
            .map_err(DatabaseError::Custom)
    }};
}

/// Runs a given query on a given connection with a given function.
/// Uses the optional function to return `DbResult<Option<T>>`
#[macro_export]
macro_rules! exec_opt {
    ($query:expr, $conn:expr, $func:ident) => {{
        $query
            .$func(&$conn)
            .optional()
            .map_err(DieselError::DieselError)
            .map_err(DatabaseError::Custom)
    }};
}

/// Runs a given query on a given connection with a given function.
/// Uses the unique function to return `DbResult<Option<T>>`
#[macro_export]
macro_rules! exec_unique {
    ($query:expr, $conn:expr, $func:ident) => {{
        $query.$func(&$conn).unique()
    }};
}
