use crate::separator;
use crate::styles;

use eframe::App;
use fnv::FnvHashMap;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// This represents a file manager.
/// It shows a list of files (so it must store them).
/// It allows the selection of a file so that must be returned.
/// It should ideally also show an image thumbnail...
pub(crate) struct FileManager {
    filters: FileFilters,
    images: FnvHashMap<Option<i64>, egui_extras::RetainedImage>,
    current_dir: PathBuf,
    current_select: Option<PathBuf>,
    paths_in_dir: Vec<PathBuf>,
    pub(crate) selection: FileSelection,
}

#[derive(Debug, Clone)]
pub(crate) enum FileSelection {
    Selected(PathBuf),
    Cancelled,
    Undecided,
}

#[derive(Debug, Clone)]
pub(crate) enum FileFilters {
    Dir,
    Files(Vec<String>),
}

impl FileFilters {
    pub(crate) fn empty() -> Self {
        Self::Files(vec![])
    }
    pub(crate) fn dirs() -> Self {
        Self::Dir
    }
    pub(crate) fn files(files: Vec<String>) -> Self {
        Self::Files(files)
    }
    pub(crate) fn is_dirs(&self) -> bool {
        matches!(self, FileFilters::Dir)
    }
    pub(crate) fn allowed(&self, path: &std::path::Path) -> bool {
        match self {
            Self::Dir => path.is_dir(),
            Self::Files(ref f) => {
                let ext = path.extension();
                path.is_dir() || f.is_empty() || f.iter().any(|f| Some(OsStr::new(&f)) == ext)
            }
        }
    }
}

impl Default for FileSelection {
    fn default() -> Self {
        Self::Undecided
    }
}

impl App for FileManager {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.set(ctx, frame);
    }
}

impl FileManager {
    pub(crate) fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_default();
        let filters = FileFilters::empty();
        let paths_in_dir = get_entries(&current_dir, &filters);
        Self {
            filters,
            current_dir,
            paths_in_dir,
            current_select: None,
            images: FnvHashMap::default(),
            selection: FileSelection::Undecided,
        }
    }

    pub(crate) fn set_filters(&mut self, filters: FileFilters) {
        self.filters = filters;
    }

    pub(crate) fn set(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::Window::new("Select File").show(ctx, |ui| {
            ui.set_style(styles::style());
            ui.vertical(|ui| {
                if let Some(ref c) = self.current_select {
                    let _ = ui
                        .selectable_label(false, c.to_str().unwrap_or_default())
                        .clicked();
                    separator(ui);
                }
                // Display paths in dir + thumbnail.
                ui.horizontal(|ui| {
                    // Display paths in dir.
                    egui::ScrollArea::vertical()
                        .max_height(300.)
                        .min_scrolled_height(300.)
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                'paths_loop: for i in 0..self.paths_in_dir.len() {
                                    let entry = &self.paths_in_dir[i];
                                    if !self.filters.allowed(&entry) {
                                        continue 'paths_loop;
                                    }
                                    // Select the `current_dir` and `paths_in_dir` for the next go.
                                    let entry_str = match entry.file_name() {
                                        Some(n) => n.to_str().unwrap_or_default(),
                                        None => "..",
                                    };
                                    if ui.selectable_label(false, entry_str).clicked() {
                                        if entry.is_dir() {
                                            self.current_dir = entry.to_owned();
                                            let entries =
                                                get_entries(&self.current_dir, &self.filters);
                                            if self.filters.is_dirs() {
                                                self.current_select = Some(entry.to_owned());
                                            }
                                            self.paths_in_dir = entries;
                                            break 'paths_loop;
                                        } else {
                                            self.current_select = Some(entry.to_owned());
                                        }
                                    }
                                }
                            });
                        });
                    // End Display paths in dir.
                });
                // End Display paths in dir + thumbnail.
                // Buttons.
                separator(ui);
                ui.horizontal(|ui| {
                    // Go up a directory.
                    if ui.button("Up.").clicked() {
                        self.current_dir = match self.current_dir.parent() {
                            Some(p) => p.to_path_buf(),
                            None => self.current_dir.to_owned(),
                        };
                        let entries = get_entries(&self.current_dir, &self.filters);
                        self.paths_in_dir = entries;
                    }
                    // Select current entry.
                    let use_this = ui.button("Select current entry").clicked();
                    if let (true, Some(selected)) = (use_this, &self.current_select) {
                        self.selection = FileSelection::Selected(selected.to_path_buf());
                    }
                    if ui.button("Cancel").clicked() {
                        self.selection = FileSelection::Cancelled;
                    }
                })
                //End buttons.
            });
        });
    }
}

fn get_entries(current_dir: &Path, filters: &FileFilters) -> Vec<PathBuf> {
    match std::fs::read_dir(current_dir) {
        Ok(ent) => ent,
        Err(_) => return vec![],
    }
    .flatten()
    .filter(|x| filters.allowed(&x.path()))
    .map(|e| e.path().to_owned())
    .collect::<Vec<_>>()
}
