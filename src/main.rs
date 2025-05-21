use chrono::{NaiveDate, Weekday};
use crossterm::cursor::MoveTo;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::stdout;
use std::path::PathBuf;

fn main() {
    let mut todo_list: HashMap<u32, ToDo> = HashMap::new();
    let mut id_list: Vec<u32> = Vec::new();
    let mut last_id: u32 = 0;
    let mut sort_type: SortType = SortType::ID;

    if let Ok(data) = load_data() {
        todo_list = data.todo_list;
        id_list = data.id_list;
        last_id = data.last_id;
        sort_type = data.sort_type;
    }

    let menu = "Select an option...";

    let add_todo = "Add Todo";
    let remove_todo = "Remove Todo";
    let list_todo = "List Todos";
    let sort_todo = "Sort Todos";
    let exit = "Exit";
    let options = vec![add_todo, remove_todo, list_todo, sort_todo, exit];

    let sort = "Select a sort option....";

    let sort_oldest = "Sort by Date (oldest)";
    let sort_newest = "Sort by Date (newest)";
    let sort_subject = "Sort by Subject";
    let sort_id = "Sort by ID";
    let back = "Back";
    let sort_options = vec![sort_oldest, sort_newest, sort_subject, sort_id, back];

    if let Err(e) = clear_terminal() {
        eprintln!("{}", e);
    }

    loop {
        match inquire::Select::new(menu, options.clone()).prompt() {
            Ok(choice) => {
                if choice == add_todo {
                    match inquire::DateSelect::new("Enter date:")
                        .with_week_start(Weekday::Mon)
                        .prompt()
                    {
                        Ok(date) => match inquire::Text::new("Enter subject:").prompt() {
                            Ok(subject) => match inquire::Text::new("Enter body:").prompt() {
                                Ok(body) => {
                                    todo_list
                                        .insert(last_id, ToDo::new(last_id, date, subject, body));
                                    id_list.push(last_id);
                                    last_id += 1;

                                    match save_data(Data::new(
                                        todo_list.clone(),
                                        id_list.clone(),
                                        last_id,
                                        sort_type.clone(),
                                    )) {
                                        Ok(_) => {}
                                        Err(e) => {
                                            eprintln!("Failed to save ToDos: {}", e);
                                        }
                                    }

                                    if let Err(e) = clear_terminal() {
                                        eprintln!("{}", e);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("{}", e);
                                    break;
                                }
                            },
                            Err(e) => {
                                eprintln!("{}", e);
                                break;
                            }
                        },
                        Err(e) => {
                            eprintln!("{}", e);
                            break;
                        }
                    }
                }
                if choice == remove_todo && !todo_list.is_empty() {
                    match inquire::Select::new("Select a ToDo by ID to delete:", id_list.clone())
                        .prompt()
                    {
                        Ok(selection) => {
                            todo_list.remove(&selection);
                            id_list.retain(|&id| id != selection);

                            match save_data(Data::new(
                                todo_list.clone(),
                                id_list.clone(),
                                last_id,
                                sort_type.clone(),
                            )) {
                                Ok(_) => {}
                                Err(e) => {
                                    eprintln!("Failed to save ToDos: {}", e);
                                }
                            }

                            if let Err(e) = clear_terminal() {
                                eprintln!("{}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                            break;
                        }
                    }
                }
                if choice == list_todo && !todo_list.is_empty() {
                    if let Err(e) = clear_terminal() {
                        eprintln!("{}", e);
                    }

                    let mut sorted_todos: Vec<_> = todo_list.values().collect();

                    match sort_type {
                        SortType::Oldest => sorted_todos.sort_by_key(|todo| todo.date),
                        SortType::Newest => sorted_todos.sort_by_key(|todo| Reverse(todo.date)),
                        SortType::Subject => sorted_todos.sort_by_key(|todo| &todo.subject),
                        SortType::ID => sorted_todos.sort_by_key(|todo| todo.id),
                    }

                    sorted_todos.iter().for_each(|todo| println!("{}", todo));
                }
                if choice == sort_todo && !todo_list.is_empty() {
                    match inquire::Select::new(sort, sort_options.clone()).prompt() {
                        Ok(choice) => {
                            if choice == sort_oldest {
                                sort_type = SortType::Oldest;
                            }
                            if choice == sort_newest {
                                sort_type = SortType::Newest;
                            }
                            if choice == sort_subject {
                                sort_type = SortType::Subject;
                            }
                            if choice == sort_id {
                                sort_type = SortType::ID;
                            }
                            if choice == back {
                                continue;
                            }

                            match save_data(Data::new(
                                todo_list.clone(),
                                id_list.clone(),
                                last_id,
                                sort_type.clone(),
                            )) {
                                Ok(_) => {}
                                Err(e) => {
                                    eprintln!("Failed to save ToDos: {}", e);
                                }
                            }

                            if let Err(e) = clear_terminal() {
                                eprintln!("{}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                            break;
                        }
                    }
                }
                if choice == exit {
                    break;
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                break;
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToDo {
    id: u32,
    date: NaiveDate,
    subject: String,
    body: String,
}

impl ToDo {
    fn new(id: u32, date: NaiveDate, subject: String, body: String) -> ToDo {
        Self {
            id,
            date,
            subject,
            body,
        }
    }
}

impl fmt::Display for ToDo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ID: {}\nDate: {}\nSubject: {}\nBody: {}\n",
            self.id, self.date, self.subject, self.body
        )
    }
}

#[derive(Serialize, Deserialize)]
struct Data {
    todo_list: HashMap<u32, ToDo>,
    id_list: Vec<u32>,
    last_id: u32,
    sort_type: SortType,
}

impl Data {
    fn new(
        todo_list: HashMap<u32, ToDo>,
        id_list: Vec<u32>,
        last_id: u32,
        sort_type: SortType,
    ) -> Self {
        Self {
            todo_list,
            id_list,
            last_id,
            sort_type,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
enum SortType {
    Oldest,
    Newest,
    Subject,
    ID,
}

fn clear_terminal() -> Result<(), std::io::Error> {
    execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0))
}

fn load_data() -> Result<Data, Box<dyn std::error::Error>> {
    let path = PathBuf::from("data.json");

    let data_string = fs::read_to_string(path)?;

    let data: Data = serde_json::from_str(&data_string)?;

    Ok(data)
}

fn save_data(data: Data) -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("data.json");

    let json = serde_json::to_string_pretty(&data)?;

    fs::write(path, json)?;

    Ok(())
}
