use crate::row::Row;
use crate::table::*;

#[derive(Debug)]
pub enum MetaCommand {
    // Success,
    Unrecognized,
    Exit,
    PrintTree,
}

#[derive(Debug, PartialEq)]
pub enum StatementType {
    Select,
    Insert,
    Delete,
}

#[derive(Debug)]
pub struct Statement {
    statement_type: StatementType,
    pub row: Option<Row>,
}

pub fn handle_meta_command(command: &str) -> MetaCommand {
    if command.eq(".exit") {
        MetaCommand::Exit
    } else if command.eq(".tree") {
        MetaCommand::PrintTree
    } else {
        MetaCommand::Unrecognized
    }
}

pub fn parse_action_with_id(
    input: &str,
    statement_type: StatementType,
) -> Result<Statement, String> {
    match input.split_once(' ') {
        None => Ok(Statement {
            statement_type,
            row: None,
        }),
        Some((_, id)) => {
            if let Ok(id) = id.parse::<u32>() {
                Ok(Statement {
                    statement_type,
                    row: Some(Row::create(id, "", "")),
                })
            } else {
                Err("invalid id provided".to_string())
            }
        }
    }
}

pub fn prepare_statement(input: &str) -> Result<Statement, String> {
    if input.starts_with("select") {
        return parse_action_with_id(input, StatementType::Select);
    }

    if input.starts_with("delete") {
        return parse_action_with_id(input, StatementType::Delete);
    }

    if input.starts_with("insert") {
        match Row::from_statement(input) {
            Ok(row) => {
                return Ok(Statement {
                    statement_type: StatementType::Insert,
                    row: Some(row),
                })
            }
            Err(e) => return Err(e),
        }
    }

    Err("unrecognized statement".to_string())
}

pub fn execute_statement(table: &mut Table, statement: &Statement) -> String {
    match statement.statement_type {
        StatementType::Select => table.select(statement),
        StatementType::Insert => table.insert(statement.row.as_ref().unwrap()),
        StatementType::Delete => table.delete(statement.row.as_ref().unwrap()),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_select_without_id() {
        let result = prepare_statement("select");
        assert!(result.is_ok());

        let statement = result.unwrap();
        assert_eq!(statement.statement_type, StatementType::Select);
        assert_eq!(statement.row, None);
    }

    #[test]
    fn parse_select_with_id() {
        let result = prepare_statement("select 1");
        assert!(result.is_ok());

        let statement = result.unwrap();
        assert_eq!(statement.statement_type, StatementType::Select);
        assert_eq!(statement.row, Some(Row::create(1, "", "")));
    }

    #[test]
    fn parse_delete_with_id() {
        let result = prepare_statement("delete 1");
        assert!(result.is_ok());

        let statement = result.unwrap();
        assert_eq!(statement.statement_type, StatementType::Delete);
        assert_eq!(statement.row, Some(Row::create(1, "", "")));
    }

    #[test]
    fn error_when_parse_action_with_non_u32_id() {
        let result = prepare_statement("select apple");
        assert!(result.is_err());

        let message = result.unwrap_err();
        assert_eq!(message, "invalid id provided");

        let result = prepare_statement("delete apple");
        assert!(result.is_err());

        let message = result.unwrap_err();
        assert_eq!(message, "invalid id provided");
    }
}
