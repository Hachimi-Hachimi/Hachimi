use std::sync::Mutex;

use fnv::FnvHashMap;
use once_cell::sync::Lazy;
use sqlparser::{
    ast::BinaryOperator,
    dialect::SQLiteDialect,
    keywords::Keyword,
    parser::Parser
};

use crate::il2cpp::{ext::Il2CppStringExt, sql::{self, ExprExt, SelectExt, SelectItemExt}, symbols::get_method_addr, types::*};

pub static SELECT_QUERIES: Lazy<Mutex<FnvHashMap<usize, Box<dyn sql::SelectQueryState + Send + Sync>>>> =
    Lazy::new(|| Mutex::new(FnvHashMap::default()));

#[inline(never)]
fn parse_query(query: *mut Il2CppObject, sql: *const Il2CppString) {
    let sql_str = unsafe { (*sql).as_utf16str() }.to_string();

    // quick escape!!!11
    if !sql_str.starts_with("SELECT") {
        return;
    }

    // parse the sql string
    let dialect = SQLiteDialect {};
    let parser_res = Parser::new(&dialect).try_with_sql(&sql_str);

    if let Ok(mut parser) = parser_res {
        // only care about select statements
        if !parser.parse_keyword(Keyword::SELECT) {
            return;
        }
        let Ok(select) = parser.parse_select() else {
            return;
        };

        // and their first table name (SELECT FROM table_name)
        let Some(table_name) = select.get_first_table_name() else {
            debug!("no table name");
            return;
        };

        // Create the query state
        let mut query_state: Box<dyn sql::SelectQueryState + Send + Sync> = match table_name.as_ref() {
            "text_data" => Box::new(sql::TextDataQuery::default()),
            "character_system_text" => Box::new(sql::CharacterSystemTextQuery::default()),
            "race_jikkyo_comment" => Box::new(sql::RaceJikkyoCommentQuery::default()),
            "race_jikkyo_message" => Box::new(sql::RaceJikkyoMessageQuery::default()),
            _ => return
        };

        // Add columns
        let mut i = 0;
        for item in select.projection.iter() {
            if let Some(name) = item.get_unnamed_expr_ident() {
                query_state.add_column(i, name);
                i += 1;
            }
        }

        // Add params
        i = 1; // index starts at 1
        if let Some(selection) = select.selection {
            // this should visit them in order (column1 = ? AND column2 = ? ...)
            for expr in selection.binary_op_iter() {
                if *expr.op != BinaryOperator::Eq { continue; }

                if let Some(name) = expr.left.get_ident_value() {
                    if expr.right.is_placeholder_value() {
                        query_state.add_param(i, name);
                        i += 1;
                    }
                }
            }
        }

        // Add query state
        SELECT_QUERIES.lock().unwrap().insert(query as usize, query_state);
    }
}

type QueryFn = extern "C" fn(this: *mut Il2CppObject, sql: *const Il2CppString) -> *mut Il2CppObject;
extern "C" fn Query(this: *mut Il2CppObject, sql: *const Il2CppString) -> *mut Il2CppObject {
    trace!("Query");
    let query = get_orig_fn!(Query, QueryFn)(this, sql);
    parse_query(query, sql);
    query
}

type PreparedQueryFn = extern "C" fn(this: *mut Il2CppObject, sql: *const Il2CppString) -> *mut Il2CppObject;
extern "C" fn PreparedQuery(this: *mut Il2CppObject, sql: *const Il2CppString) -> *mut Il2CppObject {
    trace!("PreparedQuery");
    let query = get_orig_fn!(PreparedQuery, PreparedQueryFn)(this, sql);
    parse_query(query, sql);
    query
}

pub fn init(LibNative_Runtime: *const Il2CppImage) {
    get_class_or_return!(LibNative_Runtime, "LibNative.Sqlite3", Connection);

    let Query_addr = get_method_addr(Connection, c"Query", 1);
    let PreparedQuery_addr = get_method_addr(Connection, c"PreparedQuery", 1);

    new_hook!(Query_addr, Query);
    new_hook!(PreparedQuery_addr, PreparedQuery);
}