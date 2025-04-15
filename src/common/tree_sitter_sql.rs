use tree_sitter::{Tree, TreeCursor};

use rmcp::{Error as McpError, ServerHandler, model::*, tool};

#[derive(Clone)]
pub struct ParseSqlTool {}

#[tool(tool_box)]
impl ServerHandler for ParseSqlTool {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides tools to parse SQL statements into a tree structure using future-architect/tree-sitter-sql. Use the 'parse_sql' tool to strictly parse SQL statements, or 'parse_sql_with_error_recovery' to parse and return the tree including ERROR nodes for error recovery.".to_string()),
        }
    }
}

#[tool(tool_box)]
impl ParseSqlTool {
    pub fn new() -> Self {
        Self {}
    }

    #[tool(description = "Parse sql")]
    /// SQL をパースしてツリーを表現した文字列を返す
    /// パースに失敗した場合はエラーを返す
    pub fn parse_sql(
        #[tool(param)]
        #[schemars(description = "sql text to parse")]
        sql: String,
    ) -> Result<CallToolResult, McpError> {
        let tree = parse(&sql);

        if tree.root_node().has_error() {
            Err(McpError::invalid_params(
                "Failed to parse sql".to_string(),
                None,
            ))
        } else {
            let result = write_tree(&tree, &sql);

            Ok(CallToolResult::success(vec![Content::text(result)]))
        }
    }

    #[tool(description = "Parse sql with error recovery (including ERROR node)")]
    /// SQL をパースしてツリーを表現した文字列を返す
    /// パースは失敗せず、ERROR ノードを含めてツリーを表現した文字列を返す
    pub fn parse_sql_with_error_recovery(
        #[tool(param)]
        #[schemars(description = "sql text to parse")]
        sql: String,
    ) -> Result<CallToolResult, McpError> {
        let tree = parse(&sql);

        let result = write_tree(&tree, &sql);

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
}

fn parse(sql: &str) -> Tree {
    let language = tree_sitter_sql::language();
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(language).unwrap();

    let tree = parser.parse(&sql, None).unwrap();

    tree
}

fn write_tree(tree: &Tree, src: &str) -> String {
    let mut cursor = tree.walk();
    let mut result = String::new();
    visit(&mut cursor, 0, &src, &mut result);

    result
}

const UNIT: usize = 2;

fn visit(cursor: &mut TreeCursor, depth: usize, src: &str, result: &mut String) {
    // インデント
    for _ in 0..(depth * UNIT) {
        result.push_str("-");
    }

    result.push_str(&format!("{}", cursor.node().kind()));

    if cursor.node().child_count() == 0 {
        result.push_str(&format!(" \"{}\"", cursor.node().utf8_text(src.as_bytes()).unwrap()));
    }

    result.push_str(&format!(
        " [{}-{}]\n",
        cursor.node().start_position(),
        cursor.node().end_position()
    ));

    // 子供を走査
    if cursor.goto_first_child() {
        visit(cursor, depth + 1, src, result);
        while cursor.goto_next_sibling() {
            visit(cursor, depth + 1, src, result);
        }
        cursor.goto_parent();
    }
}
