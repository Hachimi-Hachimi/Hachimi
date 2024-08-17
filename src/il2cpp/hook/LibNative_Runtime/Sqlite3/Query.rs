use std::{os::raw::c_void, sync::Mutex};

use fnv::FnvHashMap;
use once_cell::sync::Lazy;
use sqlparser::{
    ast::BinaryOperator,
    dialect::SQLiteDialect,
    keywords::Keyword,
    parser::Parser
};

use crate::{
    core::sql::{self, ExprExt, SelectExt, SelectItemExt},
    il2cpp::{symbols::get_method_addr, types::*}
};

pub static SELECT_QUERIES: Lazy<Mutex<FnvHashMap<usize, Box<dyn sql::SelectQueryState + Send + Sync>>>> =
    Lazy::new(|| Mutex::new(FnvHashMap::default()));

type _SetupFn = extern "C" fn(this: *mut Il2CppObject, conn: *mut c_void, sql: *const Il2CppString);
extern "C" fn _Setup(this: *mut Il2CppObject, conn: *mut c_void, sql: *const Il2CppString) {
    let res = get_orig_fn!(_Setup, _SetupFn)(this, conn, sql);
    let sql_str = unsafe { (*sql).to_utf16str() }.to_string();

    // quick escape!!!11
    if !sql_str.starts_with("SELECT") {
        return res;
    }

    // parse the sql string
    let dialect = SQLiteDialect {};
    let parser_res = Parser::new(&dialect).try_with_sql(&sql_str);

    if let Ok(mut parser) = parser_res {
        // only care about select statements
        if !parser.parse_keyword(Keyword::SELECT) {
            return res;
        }
        let Ok(select) = parser.parse_select() else {
            return res;
        };

        // and their first table name (SELECT FROM table_name)
        let Some(table_name) = select.get_first_table_name() else {
            return res;
        };

        // Create the query state
        let mut query_state: Box<dyn sql::SelectQueryState + Send + Sync> = match table_name.as_ref() {
            "text_data" => Box::new(sql::TextDataQuery::default()),
            "character_system_text" => Box::new(sql::CharacterSystemTextQuery::default()),
            "race_jikkyo_comment" => Box::new(sql::RaceJikkyoCommentQuery::default()),
            "race_jikkyo_message" => Box::new(sql::RaceJikkyoMessageQuery::default()),
            _ => return res
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
        SELECT_QUERIES.lock().unwrap().insert(this as usize, query_state);
    }

    res
}

type GetTextFn = extern "C" fn(this: *mut Il2CppObject, idx: i32) -> *mut Il2CppString;
extern "C" fn GetText(this: *mut Il2CppObject, idx: i32) -> *mut Il2CppString {
    if let Some(query) = SELECT_QUERIES.lock().unwrap().get(&(this as usize)) {
        return query.get_text(this, idx).unwrap_or_else(|| get_orig_fn!(GetText, GetTextFn)(this, idx));
    }
    get_orig_fn!(GetText, GetTextFn)(this, idx)
}

type DisposeFn = extern "C" fn(this: *mut Il2CppObject);
extern "C" fn Dispose(this: *mut Il2CppObject) {
    SELECT_QUERIES.lock().unwrap().remove(&(this as usize));
    get_orig_fn!(Dispose, DisposeFn)(this);
}

static mut GETINT_ADDR: usize = 0;
impl_addr_wrapper_fn!(GetInt, GETINT_ADDR, i32, this: *mut Il2CppObject, index: i32);

pub fn init(LibNative_Runtime: *const Il2CppImage) {
    get_class_or_return!(LibNative_Runtime, "LibNative.Sqlite3", Query);

    let _Setup_addr = get_method_addr(Query, c"_Setup", 2);
    let GetText_addr = get_method_addr(Query, c"GetText", 1);
    let Dispose_addr = get_method_addr(Query, c"Dispose", 0);

    new_hook!(_Setup_addr, _Setup);
    new_hook!(GetText_addr, GetText);
    new_hook!(Dispose_addr, Dispose);

    unsafe {
        GETINT_ADDR = get_method_addr(Query, c"GetInt", 1);
    }
}