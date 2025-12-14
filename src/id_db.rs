use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::Path,
};

#[derive(Debug)]
struct IdDatabase {
    ids: HashSet<String>,
}

impl IdDatabase {
    pub fn new() -> Self {
        Self {
            ids: HashSet::new(),
        }
    }

    pub fn from_path(path: &Path) -> Result<Self, io::Error> {
        let mut db = Self::new();
        let f = File::open(path)?;
        let reader = BufReader::new(f);
        for line in reader.lines() {
            if let Ok(ref line) = line {
                db.add(line);
            }
        }

        Ok(db)
    }

    pub fn add(&mut self, id: &str) {
        self.ids.insert(id.to_owned());
    }

    fn serialize(&self, file: &mut File) -> io::Result<()> {
        for id in &self.ids {
            writeln!(file, "{}", id)?;
        }

        Ok(())
    }

    pub fn save_to_path(&self, path: &Path) -> io::Result<()> {
        let mut f = File::create(path)?;
        self.serialize(&mut f)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::id_db::IdDatabase;

    const ID: &'static str = "test_id";

    #[test]
    fn empty() {
        let db = IdDatabase::new();
        assert!(db.ids.is_empty(), "new db should be empty");
    }

    #[test]
    fn add() {
        let mut db = IdDatabase::new();
        db.add(ID);
        assert_eq!(db.ids.len(), 1, "db should contain 1 id");
        assert!(db.ids.contains(ID), "db should contain the added id");
    }

    #[test]
    fn serde() {
        let mut db = IdDatabase::new();
        db.add(ID);
        let path = Path::new("foo.txt");
        db.save_to_path(path).unwrap();
        let loaded_db = IdDatabase::from_path(path).expect("should be able to load a saved db");
        assert_eq!(loaded_db.ids.len(), 1, "db should contain 1 id");
        assert!(loaded_db.ids.contains(ID), "db should contain the added id");
    }
}
