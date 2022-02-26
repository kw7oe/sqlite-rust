use crate::BigArray;
use serde::{Deserialize, Serialize};

const USERNAME_SIZE: usize = 32;
const EMAIL_SIZE: usize = 255;

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Row {
    id: u32,
    #[serde(with = "BigArray")]
    username: [u8; USERNAME_SIZE],
    #[serde(with = "BigArray")]
    email: [u8; EMAIL_SIZE],
}

impl std::fmt::Debug for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {}, {})",
            self.id,
            String::from_utf8_lossy(&self.username),
            String::from_utf8_lossy(&self.email)
        )
    }
}

impl Row {
    // Alternatively can move to prepare_insert instead.
    pub fn from_statement(statement: &str) -> Result<Row, &str> {
        let insert_statement: Vec<&str> = statement.split(" ").collect();
        match insert_statement[..] {
            ["insert", id, name, email] => {
                if let Ok(id) = id.parse::<u32>() {
                    Ok(Self::create(id, name, email))
                } else {
                    Err("invalid id")
                }
            }
            _ => Err("invalid insert statement"),
        }
    }

    pub fn create(id: u32, u: &str, m: &str) -> Row {
        let mut username: [u8; USERNAME_SIZE] = [0; USERNAME_SIZE];
        let mut email: [u8; EMAIL_SIZE] = [0; EMAIL_SIZE];

        let mut index = 0;
        for c in u.bytes() {
            username[index] = c;
            index += 1;
        }

        index = 0;
        for c in m.bytes() {
            email[index] = c;
            index += 1;
        }

        Row {
            id,
            username,
            email,
        }
    }
}

const ROW_SIZE: usize = USERNAME_SIZE + EMAIL_SIZE + 4; // u32 is 4 x u8;
const PAGE_SIZE: usize = 4096;
const TABLE_MAX_PAGE: usize = 100;
const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;

pub struct Table {
    num_rows: usize,
    pages: [[u8; PAGE_SIZE]; TABLE_MAX_PAGE],
}

impl Table {
    pub fn new() -> Table {
        Table {
            num_rows: 0,
            // This is not ideal as we are initializing
            // the memory we don't need.
            //
            // Ideally, we want to allocate the space
            // as we needed.
            pages: [[0; PAGE_SIZE]; TABLE_MAX_PAGE],
        }
    }

    pub fn select(&self) {
        for i in 0..self.num_rows {
            let page_num = i / ROWS_PER_PAGE;
            let row_offset = i % ROWS_PER_PAGE;
            let byte_offset = row_offset * ROW_SIZE;

            let bytes = &self.pages[page_num][byte_offset..byte_offset + ROW_SIZE];
            let row: Row = bincode::deserialize(&bytes).unwrap();

            println!("{:?}", row);
        }
    }

    pub fn insert(&mut self, row: &Row) {
        let row_in_bytes = bincode::serialize(row).unwrap();
        let page_num = self.num_rows / ROWS_PER_PAGE;
        let row_offset = self.num_rows % ROWS_PER_PAGE;
        let byte_offset = row_offset * ROW_SIZE;

        println!("inserting to page {page_num} with row offset {row_offset} and byte offset {byte_offset}...");

        // Copy each byte from row into our pages.
        let mut j = 0;
        for i in byte_offset..byte_offset + ROW_SIZE {
            self.pages[page_num][i] = row_in_bytes[j];
            j += 1;
        }
        self.num_rows += 1;
    }
}
