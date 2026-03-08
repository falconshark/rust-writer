// document.rs - Document management

use std::path::{Path, PathBuf};
use std::fs;
use chrono::{DateTime, Local};

/// A single text document
#[allow(dead_code)]
pub struct Document {
    pub path: Option<PathBuf>,
    pub text: String,
    modified: bool,
    pub created_at: DateTime<Local>,
    pub modified_at: DateTime<Local>,
    pub name: String, // Custom name if unsaved
}

impl Document {
    pub fn new() -> Self {
        Self {
            path: None,
            text: String::new(),
            modified: false,
            created_at: Local::now(),
            modified_at: Local::now(),
            name: "Untitled".to_string(),
        }
    }

    pub fn from_path(path: &Path) -> Result<Self, std::io::Error> {
        let text = fs::read_to_string(path)?;
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        Ok(Self {
            path: Some(path.to_path_buf()),
            text,
            modified: false,
            created_at: Local::now(),
            modified_at: Local::now(),
            name,
        })
    }

    pub fn title(&self) -> String {
        if let Some(ref p) = self.path {
            p.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        } else {
            self.name.clone()
        }
    }

    pub fn save(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref path) = self.path {
            fs::write(path, &self.text)?;
            self.modified = false;
            self.modified_at = Local::now();
        }
        Ok(())
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn is_empty_untitled(&self) -> bool {
        self.path.is_none() && self.text.is_empty() && !self.modified
    }

    pub fn mark_modified(&mut self) {
        self.modified = true;
        self.modified_at = Local::now();
    }

    pub fn word_count(&self) -> usize {
        self.text
            .split_whitespace()
            .filter(|w| !w.is_empty())
            .count()
    }

    pub fn char_count(&self) -> usize {
        self.text.chars().count()
    }

    #[allow(dead_code)]
    pub fn line_count(&self) -> usize {
        if self.text.is_empty() {
            0
        } else {
            self.text.lines().count()
        }
    }

    /// Returns paragraphs (double-newline separated)
    #[allow(dead_code)]
    pub fn paragraphs(&self) -> Vec<&str> {
        self.text.split("\n\n").collect()
    }
}

/// Manages multiple open documents
pub struct DocumentManager {
    documents: Vec<Document>,
    current: usize,
}

impl DocumentManager {
    pub fn new() -> Self {
        let mut mgr = Self {
            documents: Vec::new(),
            current: 0,
        };
        mgr.documents.push(Document::new());
        mgr
    }

    pub fn new_document(&mut self) {
        self.documents.push(Document::new());
        self.current = self.documents.len() - 1;
    }

    pub fn open_document(&mut self, path: &Path) -> Result<(), std::io::Error> {
        let doc = Document::from_path(path)?;
        // If the only open document is an empty untitled, replace it
        if self.documents.len() == 1 && self.documents[0].is_empty_untitled() {
            self.documents[0] = doc;
            self.current = 0;
        } else {
            self.documents.push(doc);
            self.current = self.documents.len() - 1;
        }
        Ok(())
    }

    pub fn close_document(&mut self, index: usize) {
        if self.documents.len() > 1 {
            self.documents.remove(index);
            if self.current >= self.documents.len() {
                self.current = self.documents.len() - 1;
            }
        }
    }

    pub fn current_document(&self) -> Option<&Document> {
        self.documents.get(self.current)
    }

    pub fn current_document_mut(&mut self) -> Option<&mut Document> {
        self.documents.get_mut(self.current)
    }

    pub fn current_text_mut(&mut self) -> &mut String {
        &mut self.documents[self.current].text
    }

    pub fn mark_modified(&mut self) {
        if let Some(doc) = self.documents.get_mut(self.current) {
            doc.mark_modified();
        }
    }

    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    pub fn current_index(&self) -> usize {
        self.current
    }

    pub fn set_current(&mut self, index: usize) {
        if index < self.documents.len() {
            self.current = index;
        }
    }

    pub fn next_document(&mut self) {
        if !self.documents.is_empty() {
            self.current = (self.current + 1) % self.documents.len();
        }
    }

    pub fn prev_document(&mut self) {
        if !self.documents.is_empty() {
            self.current = if self.current == 0 {
                self.documents.len() - 1
            } else {
                self.current - 1
            };
        }
    }

    pub fn document_title(&self, index: usize) -> String {
        self.documents
            .get(index)
            .map(|d| d.title())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    pub fn all_documents_mut(&mut self) -> &mut Vec<Document> {
        &mut self.documents
    }
}
